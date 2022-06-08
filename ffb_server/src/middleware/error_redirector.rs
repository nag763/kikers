use std::pin::Pin;

use crate::ApplicationData;
use actix_service::{Service, Transform};
use actix_web::body::BoxBody;
use actix_web::HttpResponse;
use actix_web::ResponseError;
use actix_web::{dev::ServiceRequest, dev::ServiceResponse, Error};
use actix_web::http::header::{HeaderName, HeaderValue};
use futures::future::{ok, Ready};
use futures::Future;

use crate::error::ApplicationError;

#[derive(Default)]
pub struct ErrorRedirector;

impl<S> Transform<S, ServiceRequest> for ErrorRedirector
where
    S: Service<ServiceRequest, Response = ServiceResponse, Error = Error>,
    S::Future: 'static,
{
    type Response = ServiceResponse;
    type Error = Error;
    type InitError = ();
    type Transform = ErrorRedirectorMiddleware<S>;
    type Future = Ready<Result<Self::Transform, Self::InitError>>;

    fn new_transform(&self, service: S) -> Self::Future {
        ok(ErrorRedirectorMiddleware { service })
    }
}

#[derive(Default)]
pub struct ErrorRedirectorMiddleware<S> {
    service: S,
}

impl<S> Service<ServiceRequest> for ErrorRedirectorMiddleware<S>
where
    S: Service<ServiceRequest, Response = ServiceResponse, Error = Error>,
    S::Future: 'static,
{
    type Response = ServiceResponse<BoxBody>;
    type Error = Error;
    type Future = Pin<Box<dyn Future<Output = Result<Self::Response, Self::Error>>>>;

    actix_web::dev::forward_ready!(service);

    fn call(&self, req: ServiceRequest) -> Self::Future {

        let fut = self.service.call(req);

        Box::pin(async move {
            let mut res = fut.await?;
            if res.status().is_client_error() {
               let mut headers = res.headers_mut();
                    headers.insert(HeaderName::from_static("Refresh"), HeaderValue::from_static("google.com"));
            }
            Ok(res)
        })
    }
}
