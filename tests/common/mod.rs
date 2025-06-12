use sqlx::{PgPool, postgres::PgPoolOptions};
use std::sync::Once;
use std::time::Duration;

static INIT: Once = Once::new();

pub async fn setup_test_db() -> PgPool {
    INIT.call_once(|| {
        dotenvy::dotenv().ok();
        std::env::set_var("RUST_LOG", "info");
        tracing_subscriber::fmt::init();
    });

    let database_url = std::env::var("DATABASE_URL")
        .expect("DATABASE_URL must be set for tests")
        .replace("apidb", "apidb_test"); // Utiliser une base de données distincte pour les tests

    let pool = PgPoolOptions::new()
        .max_connections(5)
        .acquire_timeout(Duration::from_secs(3))
        .connect(&database_url)
        .await
        .expect("Failed to connect to test database");

    // Exécuter les migrations pour la base de données de test
    sqlx::migrate!("./migrations")
        .run(&pool)
        .await
        .expect("Failed to run migrations");

    pool
}
