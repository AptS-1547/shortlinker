use actix_service::{Service, Transform};
use actix_web::{
    Error, HttpRequest, HttpResponse,
    body::EitherBody,
    dev::{ServiceRequest, ServiceResponse},
    http::{Method, header::CONTENT_TYPE},
};
use futures_util::future::{LocalBoxFuture, Ready, ready};
use std::rc::Rc;
use tracing::{trace, warn};

/// Authentication strategy trait
pub trait AuthStrategy: Clone + 'static {
    /// Check if authentication is required for this request
    fn requires_auth(&self, req: &ServiceRequest) -> bool;

    /// Extract and validate the token from the request
    fn validate_request(&self, req: &ServiceRequest) -> bool;

    /// Get the name of this auth strategy (for logging)
    fn name(&self) -> &'static str;

    /// Build unauthorized response
    fn unauthorized_response<B: 'static>(
        &self,
        req: ServiceRequest,
    ) -> ServiceResponse<EitherBody<B>> {
        warn!(
            "{} authentication failed - invalid or missing token",
            self.name()
        );
        req.into_response(
            HttpResponse::Unauthorized()
                .insert_header((CONTENT_TYPE, "application/json; charset=utf-8"))
                .json(serde_json::json!({
                    "code": 401,
                    "data": { "error": "Unauthorized: Invalid or missing token" }
                }))
                .map_into_right_body(),
        )
    }

    /// Build not found response (when feature is disabled)
    fn not_found_response<B: 'static>(
        &self,
        req: ServiceRequest,
    ) -> ServiceResponse<EitherBody<B>> {
        warn!("{} is disabled - returning 404", self.name());
        req.into_response(
            HttpResponse::NotFound()
                .insert_header((CONTENT_TYPE, "text/plain; charset=utf-8"))
                .body("Not Found")
                .map_into_right_body(),
        )
    }
}

/// Generic authentication middleware
#[derive(Clone)]
pub struct GenericAuth<S: AuthStrategy> {
    strategy: S,
}

impl<S: AuthStrategy> GenericAuth<S> {
    pub fn new(strategy: S) -> Self {
        Self { strategy }
    }
}

impl<S, St, B> Transform<S, ServiceRequest> for GenericAuth<St>
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error> + 'static,
    St: AuthStrategy,
    B: 'static,
{
    type Response = ServiceResponse<EitherBody<B>>;
    type Error = Error;
    type InitError = ();
    type Transform = GenericAuthMiddleware<S, St>;
    type Future = Ready<Result<Self::Transform, Self::InitError>>;

    fn new_transform(&self, service: S) -> Self::Future {
        ready(Ok(GenericAuthMiddleware {
            service: Rc::new(service),
            strategy: self.strategy.clone(),
        }))
    }
}

pub struct GenericAuthMiddleware<S, St: AuthStrategy> {
    pub service: Rc<S>,
    pub strategy: St,
}

impl<S, St, B> GenericAuthMiddleware<S, St>
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error> + 'static,
    St: AuthStrategy,
    B: 'static,
{
    /// Handle OPTIONS requests for CORS preflight
    fn handle_options_request(req: ServiceRequest) -> ServiceResponse<EitherBody<B>> {
        req.into_response(
            HttpResponse::NoContent()
                .insert_header((CONTENT_TYPE, "text/plain; charset=utf-8"))
                .finish()
                .map_into_right_body(),
        )
    }
}

impl<S, St, B> Service<ServiceRequest> for GenericAuthMiddleware<S, St>
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error> + 'static,
    St: AuthStrategy,
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
        let strategy = self.strategy.clone();

        Box::pin(async move {
            // Handle CORS preflight requests
            if req.method() == Method::OPTIONS {
                return Ok(Self::handle_options_request(req));
            }

            // Check if this request requires authentication
            if !strategy.requires_auth(&req) {
                trace!("{} - request does not require auth", strategy.name());
                let response = srv.call(req).await?.map_into_left_body();
                return Ok(response);
            }

            // Validate the request
            if !strategy.validate_request(&req) {
                return Ok(strategy.unauthorized_response(req));
            }

            trace!("{} authentication successful", strategy.name());

            // Process the request with the next service
            let response = srv.call(req).await?.map_into_left_body();
            Ok(response)
        })
    }
}

/// Helper function to extract token from Cookie or Bearer header
pub fn extract_token_from_request(req: &HttpRequest, cookie_name: &str) -> Option<String> {
    // First, try to get token from cookie
    if let Some(cookie) = req.cookie(cookie_name) {
        trace!("Token found in cookie: {}", cookie_name);
        return Some(cookie.value().to_string());
    }

    // Fallback to Bearer token
    trace!("Cookie not found, trying Bearer token");
    req.headers()
        .get("Authorization")
        .and_then(|header_value| header_value.to_str().ok())
        .and_then(|auth_str| auth_str.strip_prefix("Bearer "))
        .map(|token| token.to_string())
}

/// Helper function to extract token from ServiceRequest
pub fn extract_token_from_service_request(
    req: &ServiceRequest,
    cookie_name: &str,
) -> Option<String> {
    extract_token_from_request(req.request(), cookie_name)
}
