# Utilisation du RepositoryTrait

Le `RepositoryTrait` fournit une interface générique pour interagir avec vos entités de base de données de manière sécurisée, typée et réutilisable. Il permet d'effectuer des opérations CRUD, des recherches avancées, de la pagination, etc.

## 1. Implémentation d'un Repository

Pour utiliser le `RepositoryTrait`, créez une structure pour votre repository et implémentez le trait :

```rust
use crate::core::base::generic_repository::{RepositoryTrait, Entry};
use crate::db::models::user::User;
use sqlx::Pool;
use sqlx::Postgres;

pub struct UserRepository {
    pool: Pool<Postgres>,
}

impl UserRepository {
    pub fn new(pool: Pool<Postgres>) -> Self {
        Self { pool }
    }
}

impl RepositoryTrait<User> for UserRepository {
    fn get_pool(&self) -> &Pool<Postgres> {
        &self.pool
    }
}
```

## 2. Utilisation des méthodes principales

### a. Récupérer tous les éléments

```rust
let users = user_repo.find_all().await?;
```

### b. Trouver par ID

```rust
let user = user_repo.find_by_id(1).await?;
```

### c. Créer un nouvel utilisateur

```rust
let new_user = User { /* ... */ };
let created = user_repo.create(new_user).await?;
```

### d. Mettre à jour un utilisateur

```rust
let updated = user_repo.update(1, user).await?;
```

### e. Supprimer un utilisateur

```rust
let deleted = user_repo.delete(1).await?;
```

### f. Recherche avancée

```rust
let users = user_repo.find_advanced(
    &[ ("email", json!("test@example.com")) ],
    Some(("created_at", OrderDirection::Desc)),
    Some(10),
    Some(0),
).await?;
```

## 3. Conseils de sécurité et bonnes pratiques

- Validez toujours les entrées utilisateur avant de les passer au repository.
- Utilisez les méthodes du trait pour éviter les injections SQL.
- Gérez les erreurs avec le type `ApiError` pour une meilleure robustesse.

## 4. Pagination et tri

```rust
let page = 1;
let page_size = 20;
let users = user_repo.paginate_sorted(page, page_size, Some("email"), Some(OrderDirection::Asc)).await?;
```

## 5. Tests

Pensez à tester vos repositories avec des bases de données de test et des mocks.

---

Pour plus d'exemples, consultez le code source du trait ou les tests d'intégration.
