# QueryBuilder Rust - Guide d'utilisation

Un query builder type-safe et fluent pour PostgreSQL utilisant SQLx, conçu pour simplifier la construction et l'exécution de requêtes SQL complexes.

## Table des matières

- [Installation](#installation)
- [Configuration](#configuration)
- [Utilisation de base](#utilisation-de-base)
- [Conditions WHERE](#conditions-where)
- [Groupes de conditions](#groupes-de-conditions)
- [Opérateurs de comparaison](#opérateurs-de-comparaison)
- [Tri et pagination](#tri-et-pagination)
- [Jointures](#jointures)
- [Groupement et agrégation](#groupement-et-agrégation)
- [Opérations CRUD](#opérations-crud)
- [Exemples avancés](#exemples-avancés)
- [Gestion d'erreurs](#gestion-derreurs)

## Installation

Ajoutez les dépendances suivantes à votre `Cargo.toml` :

```toml
[dependencies]
sqlx = { version = "0.7", features = ["runtime-tokio-rustls", "postgres", "uuid", "chrono"] }
serde_json = "1.0"
tokio = { version = "1.0", features = ["full"] }
uuid = { version = "1.0", features = ["v4", "serde"] }
```

## Configuration

Votre entité doit implémenter le trait `Entry` :

```rust
use sqlx::FromRow;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, FromRow, Serialize, Deserialize)]
pub struct User {
    pub id: uuid::Uuid,
    pub email: String,
    pub name: String,
    pub age: Option<i32>,
    pub created_at: chrono::DateTime<chrono::Utc>,
}

impl Entry for User {
    fn table_name() -> &'static str {
        "users"
    }

    fn columns() -> &'static [&'static str] {
        &["id", "email", "name", "age", "created_at"]
    }
}
```

## Utilisation de base

### Création d'un query builder

```rust
use your_crate::QueryBuilderUtil;

// Créer un nouveau query builder pour l'entité User
let query = QueryBuilderUtil::<User>::new();
```

### Requête SELECT simple

```rust
// Récupérer tous les utilisateurs
let users = QueryBuilderUtil::<User>::new()
    .fetch_all(&pool)
    .await?;

// Récupérer un utilisateur spécifique
let user = QueryBuilderUtil::<User>::new()
    .where_eq("email", "john@example.com")?
    .fetch_one(&pool)
    .await?;
```

## Conditions WHERE

### Conditions de base

```rust
// Égalité
let users = QueryBuilderUtil::<User>::new()
    .where_eq("name", "John")?
    .fetch_all(&pool)
    .await?;

// Inégalité
let users = QueryBuilderUtil::<User>::new()
    .where_ne("status", "inactive")?
    .fetch_all(&pool)
    .await?;

// Comparaisons numériques
let users = QueryBuilderUtil::<User>::new()
    .where_gt("age", 18)?
    .where_lte("age", 65)?
    .fetch_all(&pool)
    .await?;
```

### Conditions de recherche

```rust
// LIKE (sensible à la casse)
let users = QueryBuilderUtil::<User>::new()
    .where_like("name", "%John%")?
    .fetch_all(&pool)
    .await?;

// ILIKE (insensible à la casse)
let users = QueryBuilderUtil::<User>::new()
    .where_ilike("email", "%@GMAIL.COM")?
    .fetch_all(&pool)
    .await?;
```

### Conditions avec listes

```rust
// IN
let user_ids = vec![
    "550e8400-e29b-41d4-a716-446655440000",
    "550e8400-e29b-41d4-a716-446655440001"
];
let users = QueryBuilderUtil::<User>::new()
    .where_in("id", user_ids)?
    .fetch_all(&pool)
    .await?;

// NOT IN
let blocked_emails = vec!["spam@example.com", "fake@example.com"];
let users = QueryBuilderUtil::<User>::new()
    .where_not_in("email", blocked_emails)?
    .fetch_all(&pool)
    .await?;
```

### Conditions NULL

```rust
// IS NULL
let users = QueryBuilderUtil::<User>::new()
    .where_null("deleted_at")?
    .fetch_all(&pool)
    .await?;

// IS NOT NULL
let users = QueryBuilderUtil::<User>::new()
    .where_not_null("last_login")?
    .fetch_all(&pool)
    .await?;
```

### Conditions BETWEEN

```rust
// BETWEEN pour les dates
let users = QueryBuilderUtil::<User>::new()
    .where_between("created_at", "2023-01-01", "2023-12-31")?
    .fetch_all(&pool)
    .await?;

// BETWEEN pour les nombres
let users = QueryBuilderUtil::<User>::new()
    .where_between("age", 18, 65)?
    .fetch_all(&pool)
    .await?;
```

## Groupes de conditions

### Groupes avec AND

```rust
// WHERE (status = 'active' OR priority = 'high') AND department = 'IT'
let users = QueryBuilderUtil::<User>::new()
    .where_group_and(|group| {
        group
            .where_eq("status", "active")?
            .or()
            .where_eq("priority", "high")
    })?
    .and()
    .where_eq("department", "IT")?
    .fetch_all(&pool)
    .await?;
```

### Groupes avec OR

```rust
// WHERE status = 'admin' OR (department = 'IT' AND experience > 5)
let users = QueryBuilderUtil::<User>::new()
    .where_eq("status", "admin")?
    .or()
    .where_group_and(|group| {
        group
            .where_eq("department", "IT")?
            .and()
            .where_gt("experience", 5)
    })?
    .fetch_all(&pool)
    .await?;
```

### Groupes complexes imbriqués

```rust
// WHERE (name LIKE '%John%' OR email LIKE '%john%') 
//   AND (age > 18 AND (status = 'active' OR role = 'admin'))
let users = QueryBuilderUtil::<User>::new()
    .where_group_and(|group| {
        group
            .where_like("name", "%John%")?
            .or()
            .where_like("email", "%john%")
    })?
    .and()
    .where_group_and(|group| {
        group
            .where_gt("age", 18)?
            .and()
            .where_group_or(|inner_group| {
                inner_group
                    .where_eq("status", "active")?
                    .or()
                    .where_eq("role", "admin")
            })
    })?
    .fetch_all(&pool)
    .await?;
```

## Opérateurs de comparaison

| Méthode | Opérateur SQL | Description |
|---------|---------------|-------------|
| `where_eq` | `=` | Égalité |
| `where_ne` | `!=` | Inégalité |
| `where_gt` | `>` | Supérieur à |
| `where_gte` | `>=` | Supérieur ou égal à |
| `where_lt` | `<` | Inférieur à |
| `where_lte` | `<=` | Inférieur ou égal à |
| `where_like` | `LIKE` | Correspondance de motif (sensible à la casse) |
| `where_ilike` | `ILIKE` | Correspondance de motif (insensible à la casse) |
| `where_in` | `IN` | Valeur dans une liste |
| `where_not_in` | `NOT IN` | Valeur pas dans une liste |
| `where_null` | `IS NULL` | Valeur nulle |
| `where_not_null` | `IS NOT NULL` | Valeur non nulle |
| `where_between` | `BETWEEN` | Valeur dans un intervalle |

## Tri et pagination

### Tri

```rust
// Tri simple
let users = QueryBuilderUtil::<User>::new()
    .order_by_asc("name")?
    .fetch_all(&pool)
    .await?;

// Tri multiple
let users = QueryBuilderUtil::<User>::new()
    .order_by_desc("created_at")?
    .order_by_asc("name")?
    .fetch_all(&pool)
    .await?;
```

### Pagination

```rust
// LIMIT et OFFSET manuels
let users = QueryBuilderUtil::<User>::new()
    .limit(10)
    .offset(20)
    .fetch_all(&pool)
    .await?;

// Pagination automatique
let page = 3;
let page_size = 10;
let users = QueryBuilderUtil::<User>::new()
    .paginate(page, page_size)
    .order_by_asc("name")?
    .fetch_all(&pool)
    .await?;
```

### Comptage

```rust
// Compter le nombre total d'enregistrements
let total_users = QueryBuilderUtil::<User>::new()
    .where_eq("status", "active")?
    .count(&pool)
    .await?;

println!("Total users actifs: {}", total_users);
```

## Jointures

### Types de jointures

```rust
// INNER JOIN
let query = QueryBuilderUtil::<User>::new()
    .inner_join("profiles", "users.id = profiles.user_id")
    .where_eq("profiles.verified", true)?;

// LEFT JOIN
let query = QueryBuilderUtil::<User>::new()
    .left_join("orders", "users.id = orders.user_id")
    .where_null("orders.id")?; // Utilisateurs sans commandes

// RIGHT JOIN
let query = QueryBuilderUtil::<User>::new()
    .right_join("departments", "users.department_id = departments.id");

// FULL OUTER JOIN
let query = QueryBuilderUtil::<User>::new()
    .full_outer_join("permissions", "users.role_id = permissions.role_id");
```

### Jointures multiples

```rust
let users = QueryBuilderUtil::<User>::new()
    .inner_join("profiles", "users.id = profiles.user_id")
    .left_join("orders", "users.id = orders.user_id")
    .inner_join("departments", "users.department_id = departments.id")
    .where_eq("departments.active", true)?
    .where_gte("orders.total", 100.0)?
    .fetch_all(&pool)
    .await?;
```

## Groupement et agrégation

### GROUP BY

```rust
// Groupement simple
let query = QueryBuilderUtil::<User>::new()
    .select(vec!["department", "COUNT(*) as user_count"])?
    .group_by("department")?;

// Groupement multiple
let query = QueryBuilderUtil::<User>::new()
    .select(vec!["department", "status", "COUNT(*) as count"])?
    .group_by("department")?
    .group_by("status")?;
```

### SELECT avec colonnes spécifiques

```rust
// Sélectionner des colonnes spécifiques
let users = QueryBuilderUtil::<User>::new()
    .select(vec!["id", "name", "email"])?
    .where_eq("status", "active")?
    .fetch_all(&pool)
    .await?;

// SELECT DISTINCT
let unique_departments = QueryBuilderUtil::<User>::new()
    .distinct()
    .select(vec!["department"])?
    .fetch_all(&pool)
    .await?;
```

## Opérations CRUD

### INSERT

```rust
use std::collections::HashMap;
use serde_json::json;

// INSERT simple
let user_id = QueryBuilderUtil::<User>::new()
    .value("email", "new@example.com")?
    .value("name", "New User")?
    .value("age", 25)?
    .insert_returning(&pool)
    .await?;

// INSERT avec HashMap
let mut data = HashMap::new();
data.insert("email".to_string(), json!("bulk@example.com"));
data.insert("name".to_string(), json!("Bulk User"));
data.insert("age".to_string(), json!(30));

let rows_affected = QueryBuilderUtil::<User>::new()
    .values(data)?
    .insert(&pool)
    .await?;
```

### UPDATE

```rust
// UPDATE simple
let rows_updated = QueryBuilderUtil::<User>::new()
    .set("name", "Updated Name")?
    .set("age", 26)?
    .where_eq("email", "user@example.com")?
    .update(&pool)
    .await?;

// UPDATE avec retour des données modifiées
let updated_users = QueryBuilderUtil::<User>::new()
    .set("status", "active")?
    .where_eq("department", "IT")?
    .update_returning(&pool)
    .await?;

// UPDATE avec HashMap
let mut updates = HashMap::new();
updates.insert("last_login".to_string(), json!("2023-12-01T10:00:00Z"));
updates.insert("login_count".to_string(), json!(42));

let rows_updated = QueryBuilderUtil::<User>::new()
    .set_multiple(updates)?
    .where_eq("id", user_id)?
    .update(&pool)
    .await?;
```

### DELETE

```rust
// DELETE simple
let rows_deleted = QueryBuilderUtil::<User>::new()
    .where_eq("status", "inactive")?
    .where_lt("last_login", "2023-01-01")?
    .delete(&pool)
    .await?;

// DELETE avec retour des données supprimées
let deleted_users = QueryBuilderUtil::<User>::new()
    .where_eq("email", "to-delete@example.com")?
    .delete_returning(&pool)
    .await?;
```

## Exemples avancés

### Recherche complexe avec filtres

```rust
async fn search_users(
    pool: &Pool<Postgres>,
    search_term: Option<String>,
    department: Option<String>,
    min_age: Option<i32>,
    max_age: Option<i32>,
    page: u32,
    page_size: u32,
) -> Result<(Vec<User>, i64), ApiError> {
    let mut query = QueryBuilderUtil::<User>::new();

    // Recherche textuelle
    if let Some(term) = search_term {
        query = query.where_group_or(|group| {
            group
                .where_ilike("name", format!("%{}%", term))?
                .or()
                .where_ilike("email", format!("%{}%", term))
        })?;
    }

    // Filtre par département
    if let Some(dept) = department {
        query = query.and().where_eq("department", dept)?;
    }

    // Filtre par âge
    if let (Some(min), Some(max)) = (min_age, max_age) {
        query = query.and().where_between("age", min, max)?;
    } else if let Some(min) = min_age {
        query = query.and().where_gte("age", min)?;
    } else if let Some(max) = max_age {
        query = query.and().where_lte("age", max)?;
    }

    // Compter le total pour la pagination
    let total = query.clone().count(pool).await?;

    // Récupérer la page demandée
    let users = query
        .order_by_asc("name")?
        .paginate(page, page_size)
        .fetch_all(pool)
        .await?;

    Ok((users, total))
}
```

### Rapport avec jointures et agrégation

```rust
#[derive(Debug, FromRow)]
struct DepartmentStats {
    department: String,
    total_users: i64,
    active_users: i64,
    average_age: Option<f64>,
}

async fn get_department_stats(pool: &Pool<Postgres>) -> Result<Vec<DepartmentStats>, ApiError> {
    let stats = QueryBuilderUtil::<User>::new()
        .select(vec![
            "department",
            "COUNT(*) as total_users",
            "COUNT(CASE WHEN status = 'active' THEN 1 END) as active_users",
            "AVG(age) as average_age"
        ])?
        .where_not_null("department")?
        .group_by("department")?
        .order_by_desc("total_users")?
        .build_select_query()
        .build_query_as::<DepartmentStats>()
        .fetch_all(pool)
        .await
        .map_err(ApiError::Database)?;

    Ok(stats)
}
```

### Mise à jour conditionnelle

```rust
async fn update_user_status(
    pool: &Pool<Postgres>,
    user_id: uuid::Uuid,
    new_status: &str,
) -> Result<Option<User>, ApiError> {
    // Vérifier que l'utilisateur existe et n'est pas déjà dans le bon statut
    let updated = QueryBuilderUtil::<User>::new()
        .set("status", new_status)?
        .set("updated_at", chrono::Utc::now())?
        .where_eq("id", user_id)?
        .where_ne("status", new_status)?
        .update_returning(pool)
        .await?;

    Ok(updated.into_iter().next())
}
```

## Gestion d'erreurs

Le query builder utilise le type `ApiError` pour la gestion d'erreurs :

```rust
#[derive(Debug)]
pub enum ApiError {
    Database(sqlx::Error),
    NotFound(String),
    InvalidColumn(String),
    InvalidQuery(String),
    // ... autres erreurs
}

// Exemple de gestion d'erreurs
match QueryBuilderUtil::<User>::new()
    .where_eq("invalid_column", "value")
    .and()
    .fetch_one(&pool)
    .await
{
    Ok(user) => println!("Utilisateur trouvé: {:?}", user),
    Err(ApiError::InvalidColumn(col)) => {
        eprintln!("Colonne invalide: {}", col);
    },
    Err(ApiError::NotFound(msg)) => {
        eprintln!("Utilisateur non trouvé: {}", msg);
    },
    Err(ApiError::Database(db_err)) => {
        eprintln!("Erreur de base de données: {}", db_err);
    },
    Err(e) => eprintln!("Autre erreur: {:?}", e),
}
```

## Bonnes pratiques

### 1. Validation des colonnes

Le query builder valide automatiquement les noms de colonnes via le trait `Entry` :

```rust
impl Entry for User {
    fn columns() -> &'static [&'static str] {
        &["id", "email", "name", "age", "created_at", "updated_at"]
    }
}
```

### 2. Type safety avec les UUIDs

Le query builder gère automatiquement la conversion des UUIDs :

```rust
let user_id = uuid::Uuid::new_v4();
let user = QueryBuilderUtil::<User>::new()
    .where_eq("id", user_id.to_string())? // Conversion automatique
    .fetch_one(&pool)
    .await?;
```

### 3. Réutilisation des requêtes

```rust
// Créer une requête de base réutilisable
fn active_users_query() -> QueryBuilderUtil<User> {
    QueryBuilderUtil::<User>::new()
        .where_eq("status", "active")
        .unwrap()
        .where_not_null("email")
        .unwrap()
}

// L'utiliser dans différents contextes
let recent_users = active_users_query()
    .where_gte("created_at", "2023-01-01")?
    .order_by_desc("created_at")?
    .limit(10)
    .fetch_all(&pool)
    .await?;

let users_by_department = active_users_query()
    .where_eq("department", "Engineering")?
    .order_by_asc("name")?
    .fetch_all(&pool)
    .await?;
```

### 4. Tests unitaires

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_user_query_builder() {
        let pool = create_test_pool().await;
        
        let users = QueryBuilderUtil::<User>::new()
            .where_like("email", "%@test.com")?
            .where_gt("age", 18)?
            .order_by_asc("name")?
            .limit(5)
            .fetch_all(&pool)
            .await?;

        assert!(users.len() <= 5);
        // Autres assertions...
    }
}
```

Ce query builder offre une interface fluente et type-safe pour construire des requêtes SQL complexes tout en maintenant la lisibilité et la maintenabilité du code.
