use std::pin::Pin;
use std::task::{Context, Poll};

use actix_service::{Service, Transform};
use actix_web::HttpMessage;
use actix_web::HttpResponse;
use actix_web::ResponseError;
use actix_web::{dev::ServiceRequest, dev::ServiceResponse, Error};
use futures::future::{ok, Ready};
use futures::Future;

use crate::error::ApplicationError;

#[derive(Default)]
pub struct CookieChecker;

impl<S, B> Transform<S> for CookieChecker
where
    S: Service<Request = ServiceRequest, Response = ServiceResponse<B>, Error = Error>,
    S::Future: 'static,
    B: 'static,
{
    type Request = ServiceRequest;
    type Response = ServiceResponse<B>;
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

impl<S, B> Service for CookieCheckerMiddleware<S>
where
    S: Service<Request = ServiceRequest, Response = ServiceResponse<B>, Error = Error>,
    S::Future: 'static,
    B: 'static,
{
    type Request = ServiceRequest;
    type Response = ServiceResponse<B>;
    type Error = Error;
    type Future = Pin<Box<dyn Future<Output = Result<Self::Response, Self::Error>>>>;

    fn poll_ready(&mut self, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        self.service.poll_ready(cx)
    }

    fn call(&mut self, req: ServiceRequest) -> Self::Future {
        let cookie_approval_path: String =
            std::env::var("COOKIE_APPROVAL_PATH").expect("No COOKIE_APPROVAL_PATH in .env");
        if req.cookie(cookie_approval_path.as_str()).is_none() {
            match req.path() {
                "/" => {
                    return Box::pin(async move {
                        Ok(req.into_response(
                            HttpResponse::Found()
                                .header("Location", "/cookies")
                                .finish()
                                .into_body(),
                        ))
                    })
                }
                "/cookies" => {}
                _ => {
                    return Box::pin(async move {
                        Ok(req.into_response(
                            ApplicationError::CookiesUnapproved
                                .error_response()
                                .into_body(),
                        ))
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
