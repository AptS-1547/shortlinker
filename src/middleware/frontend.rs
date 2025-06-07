use actix_service::{Service, Transform};
use actix_web::{
    body::EitherBody,
    dev::{ServiceRequest, ServiceResponse},
    http::header::CONTENT_TYPE,
    http::StatusCode,
    Error, HttpResponse,
};
use futures_util::future::{ready, LocalBoxFuture, Ready};
use std::env;
use std::rc::Rc;
use std::sync::OnceLock;
use tracing::debug;

static ENABLE_FRONTEND_ROUTES: OnceLock<bool> = OnceLock::new();
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
            let enable_frontend_routes = ENABLE_FRONTEND_ROUTES.get_or_init(|| {
                env::var("ENABLE_FRONTEND_ROUTES")
                    .map(|v| v == "true")
                    .unwrap_or(false)
            });
            let admin_token =
                ADMIN_TOKEN.get_or_init(|| env::var("ADMIN_TOKEN").unwrap_or_default());

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
