use actix_service::{Service, Transform};
use actix_web::{
    body::EitherBody,
    dev::{ServiceRequest, ServiceResponse},
    http::header::CONTENT_TYPE,
    http::StatusCode,
    Error, HttpResponse,
};
use futures_util::future::{ready, LocalBoxFuture, Ready};
use std::rc::Rc;
use std::sync::OnceLock;
use tracing::debug;

static ENABLE_ADMIN_PANEL: OnceLock<bool> = OnceLock::new();
static ADMIN_TOKEN: OnceLock<String> = OnceLock::new();

#[derive(Clone)]
pub struct FrontendGuard;

impl<S, B> Transform<S, ServiceRequest> for FrontendGuard
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error> + 'static,
    B: 'static,
{
    type Response = ServiceResponse<EitherBody<B>>;
    type Error = Error;
    type InitError = ();
    type Transform = FrontendGuardMiddleware<S>;
    type Future = Ready<Result<Self::Transform, Self::InitError>>;

    fn new_transform(&self, service: S) -> Self::Future {
        ready(Ok(FrontendGuardMiddleware {
            service: Rc::new(service),
        }))
    }
}

pub struct FrontendGuardMiddleware<S> {
    service: Rc<S>,
}

impl<S, B> Service<ServiceRequest> for FrontendGuardMiddleware<S>
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error> + 'static,
    B: 'static,
{
    type Response = ServiceResponse<EitherBody<B>>;
    type Error = Error;
    type Future = LocalBoxFuture<'static, Result<Self::Response, Self::Error>>;

    fn poll_ready(
        &self,
        ctx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Result<(), Self::Error>> {
        self.service.poll_ready(ctx)
    }

    fn call(&self, req: ServiceRequest) -> Self::Future {
        let srv = self.service.clone();
        Box::pin(async move {
            let config = crate::system::app_config::get_config();
            let enable_frontend_routes =
                ENABLE_ADMIN_PANEL.get_or_init(|| config.features.enable_admin_panel);
            let admin_token = ADMIN_TOKEN.get_or_init(|| config.api.admin_token.clone());

            if !enable_frontend_routes || admin_token.is_empty() {
                return Ok(req.into_response(
                    HttpResponse::build(StatusCode::NOT_FOUND)
                        .insert_header((CONTENT_TYPE, "text/plain; charset=utf-8"))
                        .body("Not Found")
                        .map_into_right_body(),
                ));
            }

            debug!("Processing frontend request: {}", req.path());
            let res = srv.call(req).await?.map_into_left_body();
            Ok(res)
        })
    }
}
