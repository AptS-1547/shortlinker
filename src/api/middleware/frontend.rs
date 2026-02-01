use actix_service::{Service, Transform};
use actix_web::{
    Error, HttpResponse,
    body::EitherBody,
    dev::{ServiceRequest, ServiceResponse},
    http::StatusCode,
    http::header::CONTENT_TYPE,
};
use futures_util::future::{LocalBoxFuture, Ready, ready};
use std::rc::Rc;
use tracing::trace;

use crate::config::{get_runtime_config, keys};

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
            let rt = get_runtime_config();
            let enable_frontend_routes = rt.get_bool_or(keys::FEATURES_ENABLE_ADMIN_PANEL, false);
            let admin_token = rt.get_or(keys::API_ADMIN_TOKEN, "");

            if !enable_frontend_routes || admin_token.is_empty() {
                return Ok(req.into_response(
                    HttpResponse::build(StatusCode::NOT_FOUND)
                        .insert_header((CONTENT_TYPE, "text/plain; charset=utf-8"))
                        .body("Not Found")
                        .map_into_right_body(),
                ));
            }

            trace!("Processing frontend request: {}", req.path());
            let res = srv.call(req).await?.map_into_left_body();
            Ok(res)
        })
    }
}
