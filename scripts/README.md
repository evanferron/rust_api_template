# Générateur de Fonctionnalités

Ce script automatise la création de nouvelles fonctionnalités dans votre API Rust. Il génère automatiquement tous les fichiers nécessaires selon l'architecture du projet.

## Utilisation

### Linux/macOS/WSL
```bash
./scripts/generate_feature.sh
```

### Windows (Git Bash)
```bash
bash scripts/generate_feature.sh
```

## Fonctionnalités

Le script génère automatiquement :

### 🗄️ Couche Base de Données
- **Modèle** (`src/db/models/{feature}.rs`) - Structure de données avec traits Entry
- **Repository** (`src/db/repositories/{feature}_repository.rs`) - Couche d'accès aux données
- **Migration SQL** (`migrations/{timestamp}_create_{features}_table.sql`) - Script de création de table

### 🔧 Couche Métier
- **Models** (`src/modules/{feature}/{feature}_models.rs`) - DTOs et structures de requête/réponse
- **Service** (`src/modules/{feature}/{feature}_service.rs`) - Logique métier
- **Module** (`src/modules/{feature}/mod.rs`) - Déclaration du module

### 🌐 Couche API
- **Controller** (`src/api/protected/{feature}/{feature}_controller.rs`) - Endpoints REST
- **Routes** (`src/api/protected/{feature}/mod.rs`) - Configuration des routes

## Exemple d'utilisation

```bash
$ ./scripts/generate_feature.sh

🚀 Générateur de fonctionnalités pour l'API Rust

Nom de la fonctionnalité (ex: message, notification): product

Nom snake_case: product
Nom PascalCase: Product
Nom camelCase: product
Nom pluriel: products

Définition des champs du modèle (en plus des champs par défaut: id, created_at, updated_at)
Format: nom_champ:type_rust (ex: title:String, user_id:Uuid)
Types disponibles: String, i32, i64, f64, bool, Uuid, Option<String>, Vec<u8>, etc.
Appuyez sur Entrée pour terminer la saisie

Champ 1 (ou Entrée pour terminer): name:String
Champ 2 (ou Entrée pour terminer): description:Option<String>
Champ 3 (ou Entrée pour terminer): price:f64
Champ 4 (ou Entrée pour terminer): 

Cette entité a-t-elle des relations avec d'autres entités? (y/N): y

Définition des clés étrangères
Format: nom_champ:entité_référencée (ex: user_id:user, channel_id:channel)

Clé étrangère 1 (ou Entrée pour terminer): user_id:user
Clé étrangère 2 (ou Entrée pour terminer): 

Résumé de la fonctionnalité à créer:
  - Nom: product
  - Champs personnalisés: 3
  - Clés étrangères: 1

Confirmer la création? (y/N): y

🔧 Création de la fonctionnalité 'product'...
✅ Fonctionnalité 'product' créée avec succès!
```

## Types de données supportés

| Type Rust | Type SQL | Description |
|-----------|----------|-------------|
| `String` | `VARCHAR NOT NULL` | Chaîne de caractères obligatoire |
| `Option<String>` | `VARCHAR` | Chaîne de caractères optionnelle |
| `i32` | `INTEGER NOT NULL` | Entier 32 bits |
| `i64` | `BIGINT NOT NULL` | Entier 64 bits |
| `f64` | `DOUBLE PRECISION NOT NULL` | Nombre à virgule flottante |
| `bool` | `BOOLEAN NOT NULL DEFAULT FALSE` | Booléen |
| `Uuid` | `UUID NOT NULL` | Identifiant unique |
| `Vec<u8>` | `BYTEA` | Données binaires |

## Structure générée

```
src/
├── db/
│   ├── models/
│   │   └── {feature}.rs              # Modèle de base de données
│   └── repositories/
│       └── {feature}_repository.rs   # Repository avec CRUD
├── modules/
│   └── {feature}/
│       ├── mod.rs                    # Module declaration
│       ├── {feature}_models.rs       # DTOs et requêtes
│       └── {feature}_service.rs      # Logique métier
└── api/
    └── protected/
        └── {feature}/
            ├── mod.rs                # Configuration des routes
            └── {feature}_controller.rs # Endpoints REST

migrations/
└── {timestamp}_create_{features}_table.sql  # Migration SQL
```

## Actions post-génération

Après l'exécution du script, vous devez :

1. **Ajouter le repository** dans `src/config/models.rs` :
```rust
pub struct Repositories {
    // ... existing repositories
    pub {feature}_repository: {Feature}Repository,
}
```

2. **Ajouter le service** dans `src/config/models.rs` :
```rust
pub struct Services {
    // ... existing services
    pub {feature}_service: {Feature}Service,
}
```

3. **Ajouter les routes** dans `src/api/protected/mod.rs` :
```rust
pub mod {feature};

pub fn routes_config(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/{feature}")
            .configure({feature}::routes_config)
    );
}
```

4. **Exécuter la migration** :
```bash
sqlx migrate run
```

5. **Compiler le projet** :
```bash
cargo build
```

## Fonctionnalités générées

### Endpoints API

- `GET /api/protected/{feature}` - Liste toutes les entités
- `GET /api/protected/{feature}/{id}` - Récupère une entité par ID
- `POST /api/protected/{feature}` - Crée une nouvelle entité
- `PUT /api/protected/{feature}/{id}` - Met à jour une entité
- `DELETE /api/protected/{feature}/{id}` - Supprime une entité

### Validation automatique

- Validation des données d'entrée avec le crate `validator`
- Gestion d'erreurs standardisée
- Documentation OpenAPI générée automatiquement

### Optimisations

- Index automatiques sur les clés étrangères
- Trigger pour mise à jour automatique de `updated_at`
- Repository pattern avec méthodes génériques

## Personnalisation

Le code généré inclut des commentaires pour vous guider dans la personnalisation :

- Ajoutez des validations métier spécifiques dans le service
- Implémentez des méthodes de recherche personnalisées dans le repository
- Ajoutez des endpoints API supplémentaires selon vos besoins

## Troubleshooting

### Erreur de permissions
```bash
chmod +x scripts/generate_feature.sh
```

### Script ne trouve pas Cargo.toml
Assurez-vous d'exécuter le script depuis la racine du projet.

### Erreur de compilation après génération
1. Vérifiez que tous les imports sont corrects
2. Assurez-vous d'avoir ajouté le repository et service dans `config/models.rs`
3. Vérifiez que les routes sont bien configurées
