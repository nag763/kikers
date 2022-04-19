use std::pin::Pin;

use actix_service::{Service, Transform};
use actix_web::body::BoxBody;
use actix_web::HttpResponse;
use actix_web::ResponseError;
use actix_web::{dev::ServiceRequest, dev::ServiceResponse, Error};
use futures::future::{ok, Ready};
use futures::Future;

use crate::error::ApplicationError;

#[derive(Default)]
pub struct CookieChecker;

impl<S> Transform<S, ServiceRequest> for CookieChecker
where
    S: Service<ServiceRequest, Response = ServiceResponse, Error = Error>,
    S::Future: 'static,
{
    type Response = ServiceResponse;
    type Error = Error;
    type InitError = ();
    type Transform = CookieCheckerMiddleware<S>;
    type Future = Ready<Result<Self::Transform, Self::InitError>>;

    fn new_transform(&self, service: S) -> Self::Future {
        ok(CookieCheckerMiddleware { service })
    }
}

#[derive(Default)]
pub struct CookieCheckerMiddleware<S> {
    service: S,
}

impl<S> Service<ServiceRequest> for CookieCheckerMiddleware<S>
where
    S: Service<ServiceRequest, Response = ServiceResponse, Error = Error>,
    S::Future: 'static,
{
    type Response = ServiceResponse<BoxBody>;
    type Error = Error;
    type Future = Pin<Box<dyn Future<Output = Result<Self::Response, Self::Error>>>>;

    actix_web::dev::forward_ready!(service);

    fn call(&self, req: ServiceRequest) -> Self::Future {
        let cookie_approval_path: String =
            std::env::var("COOKIE_APPROVAL_PATH").expect("No COOKIE_APPROVAL_PATH in .env");
        if req.cookie(cookie_approval_path.as_str()).is_none() {
            match req.path() {
                "/" => {
                    return Box::pin(async move {
                        Ok(req.into_response(
                            HttpResponse::Found()
                                .append_header(("Location", "/cookies"))
                                .finish(),
                        ))
                    })
                }
                "/cookies" => {}
                _ => {
                    return Box::pin(async move {
                        Ok(req.into_response(ApplicationError::CookiesUnapproved.error_response()))
                    })
                }
            }
        }

        let fut = self.service.call(req);

        Box::pin(async move {
            let res = fut.await?;
            Ok(res)
        })
    }
}
