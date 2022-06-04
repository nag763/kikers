use std::pin::Pin;

use actix_service::{Service, Transform};
use actix_web::body::BoxBody;
use actix_web::ResponseError;
use actix_web::{dev::ServiceRequest, dev::ServiceResponse, Error};
use ffb_structs::ddos::Entity as DDosEntity;
use futures::future::{ok, Ready};
use futures::Future;

use crate::error::ApplicationError;

#[derive(Default)]
pub struct DDosLimiter;

impl<S> Transform<S, ServiceRequest> for DDosLimiter
where
    S: Service<ServiceRequest, Response = ServiceResponse, Error = Error>,
    S::Future: 'static,
{
    type Response = ServiceResponse;
    type Error = Error;
    type InitError = ();
    type Transform = DDosLimiterMiddleware<S>;
    type Future = Ready<Result<Self::Transform, Self::InitError>>;

    fn new_transform(&self, service: S) -> Self::Future {
        ok(DDosLimiterMiddleware { service })
    }
}

#[derive(Default)]
pub struct DDosLimiterMiddleware<S> {
    service: S,
}

impl<S> Service<ServiceRequest> for DDosLimiterMiddleware<S>
where
    S: Service<ServiceRequest, Response = ServiceResponse, Error = Error>,
    S::Future: 'static,
{
    type Response = ServiceResponse<BoxBody>;
    type Error = Error;
    type Future = Pin<Box<dyn Future<Output = Result<Self::Response, Self::Error>>>>;

    actix_web::dev::forward_ready!(service);

    fn call(&self, req: ServiceRequest) -> Self::Future {
        let (is_banned, real_ip): (Option<bool>, Option<String>) =
            if let Some(peer_addr) = req.connection_info().realip_remote_addr() {
                let real_ip: String = peer_addr.to_string();
                (
                    Some(DDosEntity::is_ip_banned(&real_ip).unwrap()),
                    Some(real_ip),
                )
            } else {
                (None, None)
            };
        if let (Some(is_banned), Some(real_ip)) = (is_banned, real_ip) {
            if is_banned {
                return Box::pin(async move {
                    Ok(req.into_response(ApplicationError::PeerBanned(real_ip).error_response()))
                });
            }
        }
        let fut = self.service.call(req);

        Box::pin(async move {
            let res = fut.await?;
            if res.status().is_client_error() {
                if let Some(peer_addr) = res.request().connection_info().realip_remote_addr() {
                    DDosEntity::register_client_error(peer_addr).unwrap();
                }
            }
            Ok(res)
        })
    }
}
