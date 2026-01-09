use actix_service::{Service, Transform};
use actix_web::{
    Error, HttpResponse,
    body::EitherBody,
    dev::{ServiceRequest, ServiceResponse},
    http::{Method, header::CONTENT_TYPE},
};
use futures_util::future::{LocalBoxFuture, Ready, ready};
use std::rc::Rc;
use tracing::{trace, warn};

use crate::api::jwt::JwtService;

#[derive(Clone)]
pub struct HealthAuth;

impl<S, B> Transform<S, ServiceRequest> for HealthAuth
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error> + 'static,
    B: 'static,
{
    type Response = ServiceResponse<EitherBody<B>>;
    type Error = Error;
    type InitError = ();
    type Transform = HealthAuthMiddleware<S>;
    type Future = Ready<Result<Self::Transform, Self::InitError>>;

    fn new_transform(&self, service: S) -> Self::Future {
        let config = crate::config::get_config();
        ready(Ok(HealthAuthMiddleware {
            service: Rc::new(service),
            admin_token: config.api.admin_token.clone(),
            access_cookie_name: config.api.access_cookie_name.clone(),
            jwt_secret: config.api.jwt_secret.clone(),
            access_token_minutes: config.api.access_token_minutes,
            refresh_token_days: config.api.refresh_token_days,
        }))
    }
}

pub struct HealthAuthMiddleware<S> {
    service: Rc<S>,
    admin_token: String,
    access_cookie_name: String,
    jwt_secret: String,
    access_token_minutes: u64,
    refresh_token_days: u64,
}

impl<S, B> HealthAuthMiddleware<S>
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error> + 'static,
    B: 'static,
{
    fn validate_jwt_cookie(
        req: &ServiceRequest,
        cookie_name: &str,
        jwt_secret: &str,
        access_token_minutes: u64,
        refresh_token_days: u64,
    ) -> bool {
        let cookie_token = req.cookie(cookie_name).map(|c| c.value().to_string());
        if let Some(token) = cookie_token {
            let jwt_service = JwtService::new(jwt_secret, access_token_minutes, refresh_token_days);
            if jwt_service.validate_access_token(&token).is_ok() {
                trace!("Health JWT validation successful");
                return true;
            }
        }
        false
    }
}

impl<S, B> Service<ServiceRequest> for HealthAuthMiddleware<S>
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
        let admin_token = self.admin_token.clone();
        let access_cookie_name = self.access_cookie_name.clone();
        let jwt_secret = self.jwt_secret.clone();
        let access_token_minutes = self.access_token_minutes;
        let refresh_token_days = self.refresh_token_days;

        Box::pin(async move {
            // Check if admin token is configured (health requires admin access)
            if admin_token.is_empty() {
                warn!("Admin token not configured - health endpoint disabled");
                return Ok(req.into_response(
                    HttpResponse::NotFound()
                        .insert_header((CONTENT_TYPE, "text/plain; charset=utf-8"))
                        .body("Not Found")
                        .map_into_right_body(),
                ));
            }

            // Handle CORS preflight
            if req.method() == Method::OPTIONS {
                return Ok(req.into_response(
                    HttpResponse::NoContent()
                        .insert_header((CONTENT_TYPE, "text/plain; charset=utf-8"))
                        .finish()
                        .map_into_right_body(),
                ));
            }

            // Validate JWT Cookie
            if !Self::validate_jwt_cookie(
                &req,
                &access_cookie_name,
                &jwt_secret,
                access_token_minutes,
                refresh_token_days,
            ) {
                warn!("Health authentication failed - invalid or missing JWT");
                return Ok(req.into_response(
                    HttpResponse::Unauthorized()
                        .insert_header((CONTENT_TYPE, "application/json; charset=utf-8"))
                        .json(serde_json::json!({
                            "code": 401,
                            "data": { "error": "Unauthorized: Invalid or missing token" }
                        }))
                        .map_into_right_body(),
                ));
            }

            trace!("Health authentication successful via JWT Cookie");
            let response = srv.call(req).await?.map_into_left_body();
            Ok(response)
        })
    }
}
