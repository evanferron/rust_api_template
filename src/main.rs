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

    // Création de la configuration du serveur à partir des variables d'environnement
    let config = Config::from_env().expect("Configuration du serveur");

    // création du serveur
    let server = Server::new(config);

    // Démarrage du serveur
    server.run().await
}
// todo : voir pour le content encodding : https://actix.rs/docs/response#content-encoding
// todo : revoir la gestion des erreurs(utiliser actix.web::Error) : https://actix.rs/docs/errors/#recommended-practices-in-error-handling
// todo : revoir la gestion des logs : https://docs.rs/log/latest/log/ faire en sorte d'avoir un logger pouvant afficher en AEG ou json
