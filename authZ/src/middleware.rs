use actix_utils::future::{ready, Ready};
use actix_web::{
    dev::{forward_ready, Service, ServiceRequest, ServiceResponse, Transform},
    error::ErrorForbidden,
    Error, HttpMessage,
};
use futures_util::future::LocalBoxFuture;
use std::rc::Rc;

use crate::{CheckPermission, ParsedPath};

#[derive(Clone, Default)]
pub struct AuthZ<P> {
    permission: P,
}

impl<P> AuthZ<P>
where
    P: CheckPermission,
{
    /// Construct `TokenAuth` middleware.
    pub fn new(permission: P) -> Self {
        Self { permission }
    }
}

// Middleware factory is `Transform` trait from actix-service crate
// `S` - type of the next service
// `B` - type of response's body
impl<S, B, F> Transform<S, ServiceRequest> for AuthZ<F>
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error> + 'static,
    S::Future: 'static,
    B: 'static,
    F: CheckPermission + Clone + 'static,
{
    type Response = ServiceResponse<B>;
    type Error = Error;
    type Transform = AuthZMiddleware<S, F>;
    type InitError = ();
    type Future = Ready<Result<Self::Transform, Self::InitError>>;

    fn new_transform(&self, service: S) -> Self::Future {
        ready(Ok(AuthZMiddleware {
            service: Rc::new(service),
            permission: self.permission.clone(),
        }))
    }
}

pub struct AuthZMiddleware<S, P> {
    service: Rc<S>,
    permission: P,
}

impl<S, B, F> Service<ServiceRequest> for AuthZMiddleware<S, F>
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error> + 'static,
    S::Future: 'static,
    F: CheckPermission + Clone + 'static,
{
    type Response = ServiceResponse<B>;
    type Error = Error;
    type Future = LocalBoxFuture<'static, Result<Self::Response, Self::Error>>;

    forward_ready!(service);

    fn call(&self, req: ServiceRequest) -> Self::Future {
        let service = Rc::clone(&self.service);
        let permission = self.permission.clone();
        let subject = match req.extensions().get::<u32>() {
            Some(id) => Some(id.clone()),
            None => None,
        };

        let url_as_str = req.uri().path();

        let path = ParsedPath::from(url_as_str);

        Box::pin(async move {
            if permission
                .check(subject, path, req.method().to_string())
                .await
            {
                let res = service.call(req).await?;
                return Ok(res);
            }

            return Err(ErrorForbidden("You don't have access to this resource!"));
        })
    }
}
