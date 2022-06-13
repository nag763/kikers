use std::pin::Pin;

use ffb_auth::JwtUser;

use crate::error::ApplicationError;
use crate::ApplicationData;
use actix_service::{Service, Transform};
use actix_web::ResponseError;
use actix_web::{dev::ServiceRequest, dev::ServiceResponse, Error};
use ffb_structs::navaccess::Model as Navaccess;
use futures::future::{ok, Ready};
use futures::Future;

#[derive(Default)]
pub struct JwtRefresher;

impl<S> Transform<S, ServiceRequest> for JwtRefresher
where
    S: Service<ServiceRequest, Response = ServiceResponse, Error = Error>,
    S::Future: 'static,
{
    type Response = ServiceResponse;
    type Error = Error;
    type InitError = ();
    type Transform = JwtRefresherMiddleware<S>;
    type Future = Ready<Result<Self::Transform, Self::InitError>>;

    fn new_transform(&self, service: S) -> Self::Future {
        ok(JwtRefresherMiddleware { service })
    }
}

#[derive(Default)]
pub struct JwtRefresherMiddleware<S> {
    service: S,
}

impl<S> Service<ServiceRequest> for JwtRefresherMiddleware<S>
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
        let token: &str = req.cookie(app_data.get_jwt_path()) {
            Some(token) => match JwtUser::from_token(token.value()) {
                    token
                }
                _ => {
                    return Box::pin(async move {
                        Ok(req.into_response(ApplicationError::IllegalToken.error_response()))
                    });
                }
            },
            None => _,
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
            let res = fut.await?;
            Ok(res)
        })
    }
}
