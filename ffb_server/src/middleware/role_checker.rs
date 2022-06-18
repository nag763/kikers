use std::pin::Pin;

use ffb_auth::JwtUser;

use crate::error::ApplicationError;
use crate::ApplicationData;
use actix_service::{Service, Transform};
use actix_web::ResponseError;
use actix_web::{
    cookie::{time::Duration, Cookie},
    dev::ServiceRequest,
    dev::ServiceResponse,
    Error,
};
use ffb_structs::navaccess::Model as Navaccess;
use futures::future::{ok, Ready};
use futures::Future;

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
        let app_data = req
            .app_data::<actix_web::web::Data<ApplicationData>>()
            .unwrap();
        let (role_id, token, jwt_user): (u32, Cookie, JwtUser) = match req
            .cookie(app_data.get_jwt_path())
        {
            Some(token) => match JwtUser::from_token(token.value()) {
                Ok(jwt_user) if !jwt_user.has_session_expired() => {
                    if JwtUser::check_token_of_login(token.value(), &jwt_user.login).is_err() {
                        return Box::pin(async move {
                            Ok(req.into_response(ApplicationError::IllegalToken.error_response()))
                        });
                    }
                    (jwt_user.role, token, jwt_user)
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

        let navaccess: Vec<Navaccess> = app_data.get_navaccess_for_role(&role_id);
        let req_path: &str = req.path();
        if !req_path.contains(app_data.get_assets_base_path())
            && !navaccess.iter().any(|nav| nav.href == req_path)
        {
            return Box::pin(async move {
                Ok(req.into_response(ApplicationError::BadRequest.error_response()))
            });
        }

        let fut = self.service.call(req);

        Box::pin(async move {
            let mut res = fut.await?;
            if jwt_user.has_to_be_refreshed() {
                let refreshed_token: String = match JwtUser::refresh_token(token.value()).await {
                    Ok(v) => v,
                    Err(e) => {
                        let response = ApplicationError::from(e).error_response();
                        return Ok(res.into_response(response));
                    }
                };

                let new_auth_token: Cookie =
                    Cookie::build(std::env::var("JWT_TOKEN_PATH").unwrap(), &refreshed_token)
                        .path("/")
                        .http_only(true)
                        .secure(true)
                        .max_age(Duration::days(7))
                        .finish();
                res.response_mut().add_cookie(&new_auth_token).unwrap();
            }
            Ok(res)
        })
    }
}
