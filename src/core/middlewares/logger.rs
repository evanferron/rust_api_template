use actix_web::{
    Error,
    body::MessageBody,
    dev::{ServiceRequest, ServiceResponse},
    middleware::Next,
};
use tracing::{error, info, warn};

pub async fn logger_middleware(
    req: ServiceRequest,
    next: Next<impl MessageBody>,
) -> Result<ServiceResponse<impl MessageBody>, Error> {
    let method = req.method().clone();
    let path = req.path().to_owned();
    let user_agent = req
        .headers()
        .get("user-agent")
        .and_then(|h| h.to_str().ok())
        .unwrap_or("unknown");
    let remote_addr = req
        .connection_info()
        .peer_addr()
        .unwrap_or("unknown")
        .to_string();

    let start = std::time::Instant::now();

    info!(
        method = %method,
        path = %path,
        remote_addr = %remote_addr,
        user_agent = %user_agent,
        "Incoming_request"
    );

    let res = next.call(req).await;

    match &res {
        Ok(response) => {
            let status = response.status();
            let duration = start.elapsed();

            if status.is_success() {
                info!(
                    method = %method,
                    path = %path,
                    status = %status.as_u16(),
                    duration_ms = %duration.as_millis(),
                    "Success_Response"
                );
            } else if status.is_client_error() {
                warn!(
                    method = %method,
                    path = %path,
                    status = %status.as_u16(),
                    duration_ms = %duration.as_millis(),
                    "Client_Error"
                );
            } else if status.is_server_error() {
                error!(
                    method = %method,
                    path = %path,
                    status = %status.as_u16(),
                    duration_ms = %duration.as_millis(),
                    "Server_Error"
                );
            } else {
                info!(
                    method = %method,
                    path = %path,
                    status = %status.as_u16(),
                    duration_ms = %duration.as_millis(),
                    "Completed"
                );
            }
        }
        Err(err) => {
            let duration = start.elapsed();
            error!(
                method = %method,
                path = %path,
                duration_ms = %duration.as_millis(),
                error = %err,
                "Error"
            );
        }
    }

    res
}
