# QueryBuilder Générique Multi-Database

Un QueryBuilder simple et générique pour Rust qui supporte PostgreSQL, MySQL et SQLite via SQLx.

## 🎯 Objectif

Ce QueryBuilder vous permet d'écrire vos requêtes SQL en dur tout en gérant automatiquement les différences de syntaxe entre les bases de données (placeholders, quoting des identifiants).

## 📦 Installation

Ajoutez les dépendances suivantes à votre `Cargo.toml` :

```toml
[dependencies]
sqlx = { version = "0.7", features = ["runtime-tokio-native-tls"] }
actix-web = "4"

[features]
postgres = ["sqlx/postgres"]
mysql = ["sqlx/mysql"]
sqlite = ["sqlx/sqlite"]
```

## 🚀 Utilisation de base

### 1. Créer un QueryBuilder

```rust
use sqlx::PgPool;

let pool = PgPool::connect("postgresql://localhost/mydb").await?;
let mut qb = QueryBuilder::new(pool, DbType::Postgres);
```

### 2. SELECT avec paramètres

```rust
use sqlx::FromRow;

#[derive(FromRow, Debug)]
struct User {
    id: i32,
    name: String,
    email: String,
}

// Avec prepare() et ParameterizedQuery (Recommandé)
let mut qb = QueryBuilder::new(pool.clone(), DbType::Postgres);

let sql = format!(
    "SELECT id, name, email FROM users WHERE age > {} AND active = {}",
    qb.placeholder(),  // $1 pour Postgres
    qb.placeholder()   // $2 pour Postgres
);

qb = qb.sql(sql);

let users: Vec<User> = qb.prepare()
    .bind(18)
    .bind(true)
    .fetch_all()
    .await?;

println!("{:?}", users);

// Alternative: Manuelle avec SQLx (pour plus de contrôle)
let mut qb = QueryBuilder::new(pool.clone(), DbType::Postgres);
let sql = format!("SELECT * FROM users WHERE age > {}", qb.placeholder());
qb = qb.sql(sql);

let users: Vec<User> = sqlx::query_as(qb.get_sql())
    .bind(18)
    .fetch_all(qb.pool())
    .await?;
```

### 3. INSERT

```rust
// Avec prepare()
let mut qb = QueryBuilder::new(pool, DbType::Postgres);

let sql = format!(
    "INSERT INTO users (name, email, age) VALUES ({}, {}, {})",
    qb.placeholder(),
    qb.placeholder(),
    qb.placeholder()
);

let result = qb.sql(sql)
    .prepare()
    .bind("Alice")
    .bind("alice@example.com")
    .bind(25)
    .execute()
    .await?;

println!("Lignes insérées: {}", result.rows_affected());
```

### 4. UPDATE

```rust
// Avec prepare()
let mut qb = QueryBuilder::new(pool, DbType::MySQL);

let sql = format!(
    "UPDATE users SET email = {} WHERE id = {}",
    qb.placeholder(),  // ? pour MySQL
    qb.placeholder()   // ? pour MySQL
);

let result = qb.sql(sql)
    .prepare()
    .bind("newemail@example.com")
    .bind(42)
    .execute()
    .await?;

println!("Lignes modifiées: {}", result.rows_affected());
```

### 5. DELETE

```rust
// Avec prepare()
let mut qb = QueryBuilder::new(pool, DbType::SQLite);

let sql = format!(
    "DELETE FROM users WHERE created_at < {}",
    qb.placeholder()
);

let result = qb.sql(sql)
    .prepare()
    .bind("2020-01-01")
    .execute()
    .await?;

println!("Lignes supprimées: {}", result.rows_affected());
```

## 🔧 Méthodes disponibles

### Construction de requête

| Méthode | Description | Exemple |
|---------|-------------|---------|
| `new(pool, db_type)` | Crée un nouveau QueryBuilder | `QueryBuilder::new(pool, DbType::Postgres)` |
| `sql(sql)` | Définit le SQL complet | `qb.sql("SELECT * FROM users")` |
| `append(sql)` | Ajoute du SQL à la suite | `qb.append("WHERE active = true")` |
| `placeholder()` | Retourne le prochain placeholder | `qb.placeholder()` → `$1` ou `?` |

### Informations

| Méthode | Description |
|---------|-------------|
| `get_sql()` | Récupère le SQL généré |
| `param_count()` | Nombre de paramètres |
| `pool()` | Référence au pool |
| `pool_clone()` | Clone le pool |
| `build()` | Décompose en `(String, Pool<DB>)` |

### Exécution sans paramètres

| Méthode | Description |
|---------|-------------|
| `execute_simple()` | Execute INSERT/UPDATE/DELETE |
| `fetch_all_simple()` | SELECT multiple lignes |
| `fetch_one_simple()` | SELECT une ligne |
| `fetch_optional_simple()` | SELECT optionnel |

### Exécution avec paramètres

| Méthode | Description |
|---------|-------------|
| `prepare()` | Crée une ParameterizedQuery pour binder et exécuter avec des paramètres |

## 📚 Exemples avancés

### Requête complexe avec JOIN

```rust
let mut qb = QueryBuilder::new(pool, DbType::Postgres);

qb = qb.append("SELECT u.id, u.name, COUNT(o.id) as order_count");
qb = qb.append("FROM users u");
qb = qb.append("LEFT JOIN orders o ON u.id = o.user_id");
qb = qb.append(&format!("WHERE u.created_at > {}", qb.placeholder()));
qb = qb.append("GROUP BY u.id, u.name");
qb = qb.append(&format!("HAVING COUNT(o.id) > {}", qb.placeholder()));
qb = qb.append("ORDER BY order_count DESC");

#[derive(FromRow)]
struct UserStats {
    id: i32,
    name: String,
    order_count: i64,
}

// Avec prepare()
let stats: Vec<UserStats> = qb.prepare()
    .bind("2024-01-01")
    .bind(5)
    .fetch_all()
    .await?;
```

### Requête conditionnelle dynamique

```rust
let mut qb = QueryBuilder::new(pool, DbType::Postgres);
qb = qb.sql("SELECT * FROM users WHERE 1=1");

let mut params: Vec<Box<dyn std::any::Any>> = Vec::new();

if let Some(min_age) = min_age {
    qb = qb.append(&format!("AND age > {}", qb.placeholder()));
    params.push(Box::new(min_age));
}

if let Some(status) = status {
    qb = qb.append(&format!("AND status = {}", qb.placeholder()));
    params.push(Box::new(status));
}

// Avec prepare() et binding dynamique
let mut bound = qb.prepare();

if let Some(min_age) = min_age {
    bound = bound.bind(min_age);
}

if let Some(status) = status {
    bound = bound.bind(status);
}

let users: Vec<User> = bound.fetch_all().await?;
```

### Utilisation avec Actix-web

```rust
use actix_web::{web, HttpResponse};

#[derive(Clone)]
pub struct AppState {
    pub pool: PgPool,
}

async fn get_users(
    state: web::Data<AppState>,
    query: web::Query<UserFilter>,
) -> Result<HttpResponse, Error> {
    let mut qb = QueryBuilder::new(state.pool.clone(), DbType::Postgres);
    
    let sql = format!(
        "SELECT * FROM users WHERE age > {} LIMIT {}",
        qb.placeholder(),
        qb.placeholder()
    );
    
    // Avec prepare()
    let users: Vec<User> = qb.sql(sql)
        .prepare()
        .bind(query.min_age)
        .bind(query.limit)
        .fetch_all()
        .await?;
    
    Ok(HttpResponse::Ok().json(users))
}

async fn create_user(
    state: web::Data<AppState>,
    user: web::Json<CreateUser>,
) -> Result<HttpResponse, Error> {
    let mut qb = QueryBuilder::new(state.pool.clone(), DbType::Postgres);
    
    let sql = format!(
        "INSERT INTO users (name, email) VALUES ({}, {}) RETURNING id",
        qb.placeholder(),
        qb.placeholder()
    );
    
    #[derive(FromRow)]
    struct NewUser {
        id: i32,
    }
    
    let new_user: NewUser = qb.sql(sql)
        .prepare()
        .bind(&user.name)
        .bind(&user.email)
        .fetch_one()
        .await?;
    
    Ok(HttpResponse::Created().json(new_user))
}
```

## 🔄 Support multi-database

Le QueryBuilder gère automatiquement les différences entre bases de données :

| Base de données | Placeholder | Quote identifiant |
|-----------------|-------------|-------------------|
| PostgreSQL | `$1`, `$2`, ... | `"column_name"` |
| MySQL | `?`, `?`, ... | `` `column_name` `` |
| SQLite | `?`, `?`, ... | `"column_name"` |

### Exemple de code portable

```rust
// Le même code fonctionne pour toutes les DB !
fn create_user_query<DB: Database>(pool: Pool<DB>, db_type: DbType) 
    -> QueryBuilder<DB> 
where
    DB: Database,
    for<'q> <DB as Database>::Arguments<'q>: sqlx::IntoArguments<'q, DB>,
    for<'c> &'c mut <DB as Database>::Connection: sqlx::Executor<'c, Database = DB>,
{
    let mut qb = QueryBuilder::new(pool, db_type);
    let sql = format!(
        "SELECT * FROM users WHERE email = {}",
        qb.placeholder()
    );
    qb.sql(sql)
}

// Utilisation avec Postgres
let qb = create_user_query(pg_pool, DbType::Postgres);

// Utilisation avec MySQL
let qb = create_user_query(mysql_pool, DbType::MySQL);
```

## ⚠️ Bonnes pratiques

### ✅ À FAIRE

```rust
// Toujours utiliser des placeholders et prepare() pour les valeurs
let mut qb = QueryBuilder::new(pool, DbType::Postgres);
let sql = format!("SELECT * FROM users WHERE id = {}", qb.placeholder());

let user: User = qb.sql(sql)
    .prepare()
    .bind(user_id)
    .fetch_one()
    .await?;
```

### ❌ À ÉVITER

```rust
// JAMAIS d'injection directe de valeurs dans le SQL !
let sql = format!("SELECT * FROM users WHERE id = {}", user_id); // DANGER !
```

### Gestion des erreurs

```rust
// Propager les erreurs SQLx
let users = sqlx::query_as::<_, User>(qb.get_sql())
    .fetch_all(qb.pool())
    .await
    .map_err(|e| {
        eprintln!("Database error: {}", e);
        MyError::Database(e)
    })?;
```

## 🎨 Type aliases

Pour simplifier, utilisez des type aliases :

```rust
pub type PgQueryBuilder = QueryBuilder<sqlx::Postgres>;
pub type MySqlQueryBuilder = QueryBuilder<sqlx::MySql>;
pub type SqliteQueryBuilder = QueryBuilder<sqlx::Sqlite>;

// Usage
let qb: PgQueryBuilder = QueryBuilder::new(pool, DbType::Postgres);
```

## 🔍 Debugging

```rust
// Afficher le SQL généré
println!("SQL: {}", qb.get_sql());
println!("Params count: {}", qb.param_count());
```

## 📝 Notes

- Le QueryBuilder ne vérifie pas la validité du SQL à la compilation
- Les paramètres doivent être bindés dans l'ordre avec `prepare()` ou SQLx
- Le quoting des identifiants n'est pas automatique (vous devez l'appliquer manuellement si nécessaire)
- Toutes les méthodes `_simple` n'acceptent pas de paramètres bindés
- Utilisez `prepare()` pour binder les paramètres de manière fluide et sûre

## 🤝 Contribution

Ce QueryBuilder est conçu pour être simple et extensible. N'hésitez pas à ajouter vos propres méthodes selon vos besoins !