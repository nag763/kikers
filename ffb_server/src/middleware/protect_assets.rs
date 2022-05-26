use std::pin::Pin;

use ffb_auth::JwtUser;

use crate::error::ApplicationError;
use crate::ApplicationData;
use actix_service::{Service, Transform};
use actix_web::ResponseError;
use actix_web::{dev::ServiceRequest, dev::ServiceResponse, Error};
use futures::future::{ok, Ready};
use futures::Future;

#[derive(Default)]
pub struct AssetsProtector;

impl<S> Transform<S, ServiceRequest> for AssetsProtector
where
    S: Service<ServiceRequest, Response = ServiceResponse, Error = Error>,
    S::Future: 'static,
{
    type Response = ServiceResponse;
    type Error = Error;
    type InitError = ();
    type Transform = AssetsProtectorMiddleware<S>;
    type Future = Ready<Result<Self::Transform, Self::InitError>>;

    fn new_transform(&self, service: S) -> Self::Future {
        ok(AssetsProtectorMiddleware { service })
    }
}

#[derive(Default)]
pub struct AssetsProtectorMiddleware<S> {
    service: S,
}

impl<S> Service<ServiceRequest> for AssetsProtectorMiddleware<S>
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
        match req.headers().get("Host") {
            Some(v) if app_data.is_host_trusted(v.to_str().unwrap()) => (),
            _ => {
                return Box::pin(async move {
                    Ok(req.into_response(ApplicationError::BadRequest.error_response()))
                });
            }
        }
        let req_path: &str = req.path();
        if req_path.contains(app_data.get_assets_base_path())
        {
            match req.cookie(app_data.get_jwt_path()) {
                Some(token) => match JwtUser::from_token(token.value()) {
                    Err(_) => {
                        return Box::pin(async move {
                            Ok(req.into_response(ApplicationError::IllegalToken.error_response()))
                        });
                    }
                    _ => (),
                },
                None => {
                    return Box::pin(async move {
                        Ok(req.into_response(ApplicationError::BadRequest.error_response()))
                    });
                }
            };
        }
        let fut = self.service.call(req);

        Box::pin(async move {
            let res = fut.await?;
            Ok(res)
        })
    }
}
