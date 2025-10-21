use actix_service::{Service, Transform};
use actix_web::{Error, body::EitherBody, dev::ServiceRequest, dev::ServiceResponse};
use futures_util::future::{Ready, ready};
use std::rc::Rc;
use tracing::trace;

use super::common::{AuthStrategy, GenericAuthMiddleware, extract_token_from_service_request};

/// Admin authentication strategy
#[derive(Clone)]
pub struct AdminAuthStrategy {
    admin_prefix: String,
    admin_token: String,
}

impl AdminAuthStrategy {
    pub fn new(admin_prefix: String, admin_token: String) -> Self {
        Self {
            admin_prefix,
            admin_token,
        }
    }

    /// Check if the request path is the login endpoint
    fn is_login_endpoint(&self, path: &str) -> bool {
        let login_path = format!("{}/auth/login", self.admin_prefix);
        path == login_path
    }
}

impl AuthStrategy for AdminAuthStrategy {
    fn requires_auth(&self, req: &ServiceRequest) -> bool {
        // Admin token must be configured
        if self.admin_token.is_empty() {
            return false;
        }

        // Login endpoint doesn't require authentication
        !self.is_login_endpoint(req.path())
    }

    fn validate_request(&self, req: &ServiceRequest) -> bool {
        // Extract token from cookie or bearer header
        let token = extract_token_from_service_request(req, "sl_admin");

        match token {
            Some(t) if t == self.admin_token => {
                trace!("Admin token validation successful");
                true
            }
            _ => {
                trace!("Admin token validation failed");
                false
            }
        }
    }

    fn name(&self) -> &'static str {
        "AdminAuth"
    }
}

/// Admin authentication middleware
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
    type Transform = GenericAuthMiddleware<S, AdminAuthStrategy>;
    type Future = Ready<Result<Self::Transform, Self::InitError>>;

    fn new_transform(&self, service: S) -> Self::Future {
        let config = crate::system::app_config::get_config();
        let strategy = AdminAuthStrategy::new(
            config.routes.admin_prefix.clone(),
            config.api.admin_token.clone(),
        );

        ready(Ok(GenericAuthMiddleware {
            service: Rc::new(service),
            strategy,
        }))
    }
}
