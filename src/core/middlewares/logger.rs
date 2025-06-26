use actix_web::{
    body::MessageBody,
    dev::{ServiceRequest, ServiceResponse},
    middleware::Next,
};

use actix_web::Error;

pub async fn logger_middleware(
    req: ServiceRequest,
    next: Next<impl MessageBody>,
) -> Result<ServiceResponse<impl MessageBody>, Error> {
    let method = req.method().clone();
    let path = req.path().to_owned();
    let start = std::time::Instant::now();
    println!("Requête: {} {}", method, path);

    let res = next.call(req).await;

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
}
