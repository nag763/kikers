use std::pin::Pin;

use ffb_auth::JwtUser;

use actix_service::{Service, Transform};
use actix_web::ResponseError;
use actix_web::{dev::ServiceRequest, dev::ServiceResponse, Error};
use ffb_structs::navaccess::Model as Navaccess;
use futures::future::{ok, Ready};
use futures::Future;

use crate::error::ApplicationError;

#[derive(Default)]
pub struct RoleChecker;

impl<S> Transform<S, ServiceRequest> for RoleChecker
where
    S: Service<ServiceRequest, Response = ServiceResponse, Error = Error>,
    S::Future: 'static,
{
    type Response = ServiceResponse;
    type Error = Error;
    type InitError = ();
    type Transform = RoleCheckerMiddleware<S>;
    type Future = Ready<Result<Self::Transform, Self::InitError>>;

    fn new_transform(&self, service: S) -> Self::Future {
        ok(RoleCheckerMiddleware { service })
    }
}

#[derive(Default)]
pub struct RoleCheckerMiddleware<S> {
    service: S,
}

impl<S> Service<ServiceRequest> for RoleCheckerMiddleware<S>
where
    S: Service<ServiceRequest, Response = ServiceResponse, Error = Error>,
    S::Future: 'static,
{
    type Response = ServiceResponse;
    type Error = Error;
    type Future = Pin<Box<dyn Future<Output = Result<Self::Response, Self::Error>>>>;

    actix_web::dev::forward_ready!(service);

    fn call(&self, req: ServiceRequest) -> Self::Future {
        let jwt_path: String =
            std::env::var("JWT_TOKEN_PATH").unwrap_or_else(|_| "jwt-token".to_string());
        let navaccess: Vec<Navaccess> = match req.cookie(jwt_path.as_str()) {
            Some(token) => match JwtUser::from_token(token.value()) {
                Ok(jwt_user) => {
                    if JwtUser::check_token(token.value()).is_err() {
                        return Box::pin(async move {
                            Ok(req.into_response(ApplicationError::IllegalToken.error_response()))
                        });
                    }
                    jwt_user.nav
                }
                _ => {
                    return Box::pin(async move {
                        Ok(req.into_response(ApplicationError::IllegalToken.error_response()))
                    });
                }
            },
            None => {
                return Box::pin(async move {
                    Ok(req.into_response(ApplicationError::BadRequest.error_response()))
                });
            }
        };

        let req_path: &str = req.path();
        if !navaccess.iter().any(|nav| nav.href == req_path) {
            return Box::pin(async move {
                Ok(req.into_response(ApplicationError::BadRequest.error_response()))
            });
        }

        let fut = self.service.call(req);

        Box::pin(async move {
            let res = fut.await?;
            Ok(res)
        })
    }
}
