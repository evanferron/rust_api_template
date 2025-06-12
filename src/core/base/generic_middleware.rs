use actix_web::Error;
use actix_web::dev::{Service, ServiceRequest, ServiceResponse, Transform, forward_ready};
use futures::future::{LocalBoxFuture, Ready, ready};
use std::future::Future;
use std::rc::Rc;

/// Middleware générique permettant d'injecter une logique personnalisée via une closure ou fonction.
pub struct GenericMiddleware<F> {
    logic: Rc<F>,
}

impl<F> GenericMiddleware<F> {
    /// Crée un nouveau middleware générique avec la logique fournie.
    pub fn new(logic: F) -> Self {
        Self {
            logic: Rc::new(logic),
        }
    }
}

impl<S, B, F, Fut> Transform<S, ServiceRequest> for GenericMiddleware<F>
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error> + 'static,
    S::Future: 'static,
    B: 'static,
    F: Fn(ServiceRequest, Rc<S>) -> Fut + 'static,
    Fut: Future<Output = Result<ServiceResponse<B>, Error>> + 'static,
{
    type Response = ServiceResponse<B>;
    type Error = Error;
    type Transform = GenericMiddlewareImpl<S, F>;
    type InitError = ();
    type Future = Ready<Result<Self::Transform, Self::InitError>>;

    fn new_transform(&self, service: S) -> Self::Future {
        ready(Ok(GenericMiddlewareImpl {
            service: Rc::new(service),
            logic: Rc::clone(&self.logic),
        }))
    }
}

pub struct GenericMiddlewareImpl<S, F> {
    service: Rc<S>,
    logic: Rc<F>,
}

impl<S, B, F, Fut> Service<ServiceRequest> for GenericMiddlewareImpl<S, F>
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error> + 'static,
    S::Future: 'static,
    B: 'static,
    F: Fn(ServiceRequest, Rc<S>) -> Fut + 'static,
    Fut: Future<Output = Result<ServiceResponse<B>, Error>> + 'static,
{
    type Response = ServiceResponse<B>;
    type Error = Error;
    type Future = LocalBoxFuture<'static, Result<Self::Response, Self::Error>>;

    forward_ready!(service);

    fn call(&self, req: ServiceRequest) -> Self::Future {
        let logic = Rc::clone(&self.logic);
        let service = Rc::clone(&self.service);
        Box::pin(async move { (logic)(req, service).await })
    }
}

/*
Exemple d'utilisation :

use crate::modules::middlewares::generic_middleware::GenericMiddleware;

let my_middleware = GenericMiddleware::new(|req, srv| async move {
    // ... logique personnalisée ...
    srv.call(req).await
});
*/
