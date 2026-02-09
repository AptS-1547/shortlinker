//! Request ID middleware
//!
//! 为每个请求生成唯一的 UUID，注入到 tracing span 中，方便日志关联追踪。

use actix_service::{Service, Transform};
use actix_web::{
    Error, HttpMessage,
    dev::{ServiceRequest, ServiceResponse},
    http::header::HeaderValue,
};
use futures_util::future::{LocalBoxFuture, Ready, ready};
use std::rc::Rc;
use tracing::{Instrument, info_span};
use uuid::Uuid;

/// 请求 ID 类型，可从 request extensions 中提取
#[derive(Clone, Debug)]
pub struct RequestId(pub String);

/// Request ID 中间件工厂
#[derive(Clone, Default)]
pub struct RequestIdMiddleware;

impl<S, B> Transform<S, ServiceRequest> for RequestIdMiddleware
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error> + 'static,
    B: 'static,
{
    type Response = ServiceResponse<B>;
    type Error = Error;
    type InitError = ();
    type Transform = RequestIdService<S>;
    type Future = Ready<Result<Self::Transform, Self::InitError>>;

    fn new_transform(&self, service: S) -> Self::Future {
        ready(Ok(RequestIdService {
            service: Rc::new(service),
        }))
    }
}

pub struct RequestIdService<S> {
    service: Rc<S>,
}

impl<S, B> Service<ServiceRequest> for RequestIdService<S>
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error> + 'static,
    B: 'static,
{
    type Response = ServiceResponse<B>;
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

        // 生成 UUID v4
        let request_id = Uuid::new_v4().to_string();

        // 存入 request extensions，handler 可以通过 req.extensions().get::<RequestId>() 获取
        req.extensions_mut().insert(RequestId(request_id.clone()));

        // 创建带有 request_id 的 tracing span
        let span = info_span!(
            "request",
            request_id = %request_id,
            method = %req.method(),
            path = %req.path(),
        );

        let request_id_for_header = request_id;

        Box::pin(
            async move {
                let mut response = srv.call(req).await?;

                // 在响应头中添加 X-Request-ID，方便调试
                if let Ok(header_value) = HeaderValue::from_str(&request_id_for_header) {
                    response.headers_mut().insert(
                        actix_web::http::header::HeaderName::from_static("x-request-id"),
                        header_value,
                    );
                }

                Ok(response)
            }
            .instrument(span),
        )
    }
}
