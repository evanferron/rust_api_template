mod api;
mod config;
mod core;
mod db;
mod modules;

use config::{config::Config, server::Server};
use dotenv::dotenv;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    dotenv().ok();

    // Création de la configuration du serveur à partir des variables d'environnement
    let config = Config::from_env().expect("Configuration du serveur");

    // création du serveur
    let server = Server::new(config);

    // Démarrage du serveur
    server.run().await
}
