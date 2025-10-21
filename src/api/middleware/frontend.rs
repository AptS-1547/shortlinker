use actix_service::{Service, Transform};
use actix_web::{Error, body::EitherBody, dev::ServiceRequest, dev::ServiceResponse};
use futures_util::future::{Ready, ready};
use std::rc::Rc;

use super::common::{AuthStrategy, GenericAuthMiddleware};

/// Frontend guard strategy
/// This is not an authentication middleware, but a feature toggle
#[derive(Clone)]
pub struct FrontendGuardStrategy {
    enable_admin_panel: bool,
    admin_token: String,
}

impl FrontendGuardStrategy {
    pub fn new(enable_admin_panel: bool, admin_token: String) -> Self {
        Self {
            enable_admin_panel,
            admin_token,
        }
    }
}

impl AuthStrategy for FrontendGuardStrategy {
    fn requires_auth(&self, _req: &ServiceRequest) -> bool {
        // Frontend routes require the feature to be enabled
        // If not enabled, return 404
        self.enable_admin_panel && !self.admin_token.is_empty()
    }

    fn validate_request(&self, _req: &ServiceRequest) -> bool {
        // FrontendGuard doesn't validate anything
        // It only checks if the feature is enabled
        true
    }

    fn name(&self) -> &'static str {
        "FrontendGuard"
    }
}

/// Frontend guard middleware
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
    type Transform = GenericAuthMiddleware<S, FrontendGuardStrategy>;
    type Future = Ready<Result<Self::Transform, Self::InitError>>;

    fn new_transform(&self, service: S) -> Self::Future {
        let config = crate::system::app_config::get_config();
        let strategy = FrontendGuardStrategy::new(
            config.features.enable_admin_panel,
            config.api.admin_token.clone(),
        );

        ready(Ok(GenericAuthMiddleware {
            service: Rc::new(service),
            strategy,
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
            let config = crate::config::get_config();
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

            trace!("Processing frontend request: {}", req.path());
            let res = srv.call(req).await?.map_into_left_body();
            Ok(res)
        })
    }
}
