use std::pin::Pin;
use std::task::{Context, Poll};

use crate::auth::JwtUser;
use crate::database::Database;
use ffb_structs::entities::navaccess::Model as Navaccess;
use actix_service::{Service, Transform};
use actix_web::HttpMessage;
use actix_web::ResponseError;
use actix_web::{dev::ServiceRequest, dev::ServiceResponse, Error};
use futures::future::{ok, Ready};
use futures::Future;

use crate::error::ApplicationError;

#[derive(Default)]
pub struct RoleChecker;

impl<S, B> Transform<S> for RoleChecker
where
    S: Service<Request = ServiceRequest, Response = ServiceResponse<B>, Error = Error>,
    S::Future: 'static,
    B: 'static,
{
    type Request = ServiceRequest;
    type Response = ServiceResponse<B>;
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

impl<S, B> Service for RoleCheckerMiddleware<S>
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
        let jwt_path: String =
            std::env::var("JWT_TOKEN_PATH").unwrap_or_else(|_| "jwt-token".to_string());
        let navaccess: Vec<Navaccess> = match req.cookie(jwt_path.as_str()) {
            Some(token) => match JwtUser::check_token(token.value()) {
                Ok(jwt_user) => {
                    let mut redis_conn = Database::acquire_redis_connection().unwrap();
                    let is_token_valid: bool = redis::cmd("SISMEMBER")
                        .arg(format!("token:{}", jwt_user.login))
                        .arg(token.value())
                        .query(&mut redis_conn)
                        .unwrap();

                    if !is_token_valid {
                        return Box::pin(async move {
                            Ok(req.into_response(
                                ApplicationError::IllegalToken.error_response().into_body(),
                            ))
                        });
                    }

                    jwt_user.nav
                }
                Err(_) => {
                    return Box::pin(async move {
                        Ok(req.into_response(
                            ApplicationError::IllegalToken.error_response().into_body(),
                        ))
                    });
                }
            },
            None => {
                return Box::pin(async move {
                    Ok(req.into_response(
                        ApplicationError::InternalError.error_response().into_body(),
                    ))
                });
            }
        };

        let req_path: &str = req.path();
        if !navaccess.iter().any(|nav| nav.href == req_path) {
            return Box::pin(async move {
                Ok(req.into_response(ApplicationError::BadRequest.error_response().into_body()))
            });
        }

        let fut = self.service.call(req);

        Box::pin(async move {
            let res = fut.await?;
            Ok(res)
        })
    }
}
