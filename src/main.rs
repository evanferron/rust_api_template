mod api;
mod config;
mod core;
mod db;
mod modules;

use config::{config::Config, server::Server};
use dotenv::dotenv;

#[tokio::main(flavor = "multi_thread", worker_threads = 4)]
async fn main() -> std::io::Result<()> {
    dotenv().ok();

    // Server configuration from environment variables
    let config = Config::from_env().expect("Server configuration failed");

    // Create server instance
    let server = Server::new(config);

    // Start the server
    server.run().await
}
