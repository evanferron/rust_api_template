use actix_web::dev::{Service, ServiceRequest, ServiceResponse};
use futures::future::LocalBoxFuture;
use std::rc::Rc;

use crate::core::base::generic_middleware::GenericMiddleware;
use actix_web::Error;

pub fn logger_middleware<S, B>() -> GenericMiddleware<
    impl Fn(ServiceRequest, Rc<S>) -> LocalBoxFuture<'static, Result<ServiceResponse<B>, Error>> + Clone,
>
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error> + 'static,
    S::Future: 'static,
    B: 'static,
{
    GenericMiddleware::new(
        |req: ServiceRequest,
         service: Rc<S>|
         -> LocalBoxFuture<'static, Result<ServiceResponse<B>, Error>> {
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
        },
    )
}
