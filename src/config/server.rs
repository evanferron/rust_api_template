use super::config::Config;
use crate::api::swagger::ApiDoc;
use crate::config::models::{Repositories, Services};
use crate::core::middlewares::logger::logger_middleware;
use crate::modules::auth::auth_service::AuthService;
use crate::modules::user::user_service::UserService;
use crate::{api, db::repositories::user_repository::UserRepository};
use actix_cors::Cors;
use actix_web::{App, HttpServer, middleware, web};
use sqlx::postgres::PgPoolOptions;
use std::sync::Arc;
use std::time::Duration;
use utoipa::OpenApi;
use utoipa_swagger_ui::SwaggerUi;

#[derive(Clone)]
pub struct Server {
    pub config: Config,
}

impl Server {
    pub fn new(config: Config) -> Self {
        Server { config }
    }

    pub async fn run(&self) -> std::io::Result<()> {
        let config = self.config.clone();
        println!("Starting server with configuration: {:#?}", config);

        // Configuration de la pool de connexions à la base de données
        let pool = PgPoolOptions::new()
            .max_connections(config.database.max_connections)
            .acquire_timeout(Duration::from_secs(config.database.acquire_timeout))
            .idle_timeout(Duration::from_secs(config.database.idle_timeout))
            .max_lifetime(Duration::from_secs(config.database.max_lifetime))
            .connect(&config.database.url)
            .await
            .expect("Cannot connect to the database");

        // Exécution des migrations
        sqlx::migrate!("./migrations")
            .run(&pool)
            .await
            .expect("Échec lors de l'exécution des migrations");

        // Initialize Logger
        tracing_subscriber::fmt()
            .with_env_filter(
                tracing_subscriber::EnvFilter::try_from_default_env()
                    .unwrap_or_else(|_| "info".into()),
            )
            .with_target(false)
            .with_thread_ids(false)
            .with_file(false)
            .with_line_number(false)
            .init();

        // Démarrage du serveur HTTP
        println!(
            "Le serveur démarre sur http://{}:{} en mode {} 🚀",
            config.server.host, config.server.port, config.server.environment
        );

        // On stock les informations de configuration dans des variables avant que la config soit moved dans le closure
        let host = config.server.host.clone();
        let port = config.server.port;

        // Création de repositories
        let repositories = Arc::new(Repositories {
            user_repository: UserRepository::new(pool.clone()),
        });

        // Création de services
        let services = Services {
            user_service: UserService::new(Arc::clone(&repositories)),
            auth_service: AuthService::new(Arc::clone(&repositories)),
        };

        HttpServer::new(move || {
            // todo : ajouter les origines autorisées dynamiquement
            // Configuration CORS
            let cors = Cors::default()
                .allowed_origin("http://localhost:3000")
                .allowed_methods(vec!["GET", "POST", "PUT", "DELETE"])
                .allowed_headers(vec!["Content-Type", "Authorization"])
                .supports_credentials()
                .max_age(3600);

            App::new()
                .wrap(cors)
                .wrap(middleware::from_fn(logger_middleware))
                .app_data(web::Data::new(pool.clone()))
                .app_data(web::Data::new(config.clone()))
                .app_data(web::Data::new(Arc::clone(&repositories)))
                .app_data(web::Data::new(services.clone()))
                .configure(api::routes_config)
                .service(
                    SwaggerUi::new("/swagger-ui/{_:.*}")
                        .url("/api-docs/openapi.json", ApiDoc::openapi()),
                )
        })
        .bind(format!("{}:{}", host, port))?
        .run()
        .await

        // todo : ajouter un default service pour les routes non définies
    }
}
