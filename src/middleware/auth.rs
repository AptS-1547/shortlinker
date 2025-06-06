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
use tracing::{debug, info};

static ADMIN_TOKEN: OnceLock<String> = OnceLock::new();

#[derive(Clone)]
pub struct AdminAuth;

impl<S, B> Transform<S, ServiceRequest> for AdminAuth
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error> + 'static,
    B: 'static,
{
    type Response = ServiceResponse<EitherBody<B>>;
    type Error = Error;
    type InitError = ();
    type Transform = AdminAuthMiddleware<S>;
    type Future = Ready<Result<Self::Transform, Self::InitError>>;

    fn new_transform(&self, service: S) -> Self::Future {
        ready(Ok(AdminAuthMiddleware {
            service: Rc::new(service),
        }))
    }
}

pub struct AdminAuthMiddleware<S> {
    service: Rc<S>,
}

impl<S, B> Service<ServiceRequest> for AdminAuthMiddleware<S>
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
            let method = req.method().clone();

            if method == actix_web::http::Method::OPTIONS {
                return Ok(req.into_response(
                    HttpResponse::build(StatusCode::NO_CONTENT)
                        .insert_header((CONTENT_TYPE, "text/plain; charset=utf-8"))
                        .finish()
                        .map_into_right_body(),
                ));
            }

            let token = ADMIN_TOKEN.get_or_init(|| env::var("ADMIN_TOKEN").unwrap_or_default());

            if token.is_empty() {
                return Ok(req.into_response(
                    HttpResponse::build(StatusCode::NOT_FOUND)
                        .insert_header((CONTENT_TYPE, "text/plain; charset=utf-8"))
                        .body("Not Found")
                        .map_into_right_body(),
                ));
            }

            let auth_ok = req
                .headers()
                .get("Authorization")
                .and_then(|h| h.to_str().ok())
                .and_then(|s| s.strip_prefix("Bearer "))
                .map(|val| val == token)
                .unwrap_or(false);

            if !auth_ok {
                info!("Admin auth failed");
                return Ok(req.into_response(
                    HttpResponse::build(StatusCode::UNAUTHORIZED)
                        .insert_header((CONTENT_TYPE, "application/json; charset=utf-8"))
                        .json(serde_json::json!({
                            "code": 401,
                            "data": { "error": "Unauthorized: Invalid or missing token" }
                        }))
                        .map_into_right_body(),
                ));
            }

            debug!("Admin auth passed");
            let res = srv.call(req).await?.map_into_left_body();
            Ok(res)
        })
    }
}
