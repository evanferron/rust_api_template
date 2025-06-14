# Utilisation du QueryBuilderUtil

Le `QueryBuilderUtil` est un utilitaire puissant pour construire dynamiquement des requêtes SQL complexes de manière sécurisée et typée. Il permet de chaîner des conditions, des tris, des jointures, de la pagination, etc., tout en évitant les injections SQL.

## 1. Création d'une requête simple

```rust
// Exemple : récupérer tous les utilisateurs dont l'email contient "test"
let users = user_repo
    .query()
    .where_ilike("email", "%test%")?
    .fetch_all(pool)
    .await?;
```

## 2. Chaînage de conditions et opérateurs logiques

```rust
let users = user_repo
    .query()
    .where_eq("role", "admin")?
    .and()
    .where_gt("created_at", "2024-01-01")?
    .fetch_all(pool)
    .await?;
```

## 3. Tri, pagination et sélection de colonnes

```rust
let users = user_repo
    .query()
    .order_by("created_at", OrderDirection::Desc)?
    .limit(10)
    .offset(20)
    .select(vec!["id", "email"])?
    .fetch_all(pool)
    .await?;
```

## 4. Utilisation des jointures

```rust
let results = user_repo
    .query()
    .inner_join("profiles", "users.id = profiles.user_id")
    .where_eq("profiles.active", true)?
    .fetch_all(pool)
    .await?;
```

## 5. Mise à jour et insertion

```rust
// Mise à jour
user_repo
    .query()
    .where_eq("id", 1)?
    .set("email", "nouveau@mail.com")?
    .update(pool)
    .await?;

// Insertion
user_repo
    .query()
    .value("email", "nouveau@mail.com")?
    .value("role", "user")?
    .insert(pool)
    .await?;
```

## 6. Conseils de sécurité et bonnes pratiques

- Utilisez toujours les méthodes du builder pour éviter les injections SQL.
- Validez les noms de colonnes via le système de types (le builder le fait pour vous).
- Privilégiez les méthodes typées plutôt que d'écrire du SQL brut.

---

Pour plus d'exemples, consultez le code source ou les tests d'intégration du projet.
