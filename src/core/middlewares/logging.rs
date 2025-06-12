use actix_web::dev::{Service, ServiceRequest, ServiceResponse, Transform};
use futures::future::{LocalBoxFuture, Ready, ready};
use std::rc::Rc;
use std::task::{Context, Poll};

pub struct Logger;

impl<S, B> Transform<S, ServiceRequest> for Logger
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = actix_web::Error> + 'static,
    S::Future: 'static,
    B: 'static,
{
    type Response = ServiceResponse<B>;
    type Error = actix_web::Error;
    type Transform = LoggerMiddleware<S>;
    type InitError = ();
    type Future = Ready<Result<Self::Transform, Self::InitError>>;

    fn new_transform(&self, service: S) -> Self::Future {
        ready(Ok(LoggerMiddleware {
            service: Rc::new(service),
        }))
    }
}

pub struct LoggerMiddleware<S> {
    service: Rc<S>,
}

impl<S, B> Service<ServiceRequest> for LoggerMiddleware<S>
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = actix_web::Error> + 'static,
    S::Future: 'static,
    B: 'static,
{
    type Response = ServiceResponse<B>;
    type Error = actix_web::Error;
    type Future = LocalBoxFuture<'static, Result<Self::Response, Self::Error>>;

    fn poll_ready(&self, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        self.service.poll_ready(cx)
    }

    fn call(&self, req: ServiceRequest) -> Self::Future {
        let service = Rc::clone(&self.service);
        let method = req.method().clone();
        let path = req.path().to_owned();

        Box::pin(async move {
            let start = std::time::Instant::now();
            println!("Requête: {} {}", method, path);

            let res = service.call(req).await;

            match &res {
                Ok(response) => {
                    let status = response.status();
                    let duration = start.elapsed();

                    println!(
                        "Réponse: {} {} - {} en {:.2?}",
                        method, path, status, duration
                    );
                }
                Err(err) => {
                    println!("Erreur: {} {} - {}", method, path, err);
                }
            }

            res
        })
    }
}
