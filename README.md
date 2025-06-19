# Rust API Template

Ce projet est un template d’API RESTful moderne en Rust, utilisant Actix-Web, SQLx, JWT, et une architecture modulaire, prêt pour la production et facilement extensible.

## Fonctionnalités principales

- Authentification sécurisée (JWT, Bcrypt)
- Gestion des utilisateurs (CRUD)
- Architecture modulaire (services, repositories, middlewares)
- Migrations SQL automatisées
- Documentation OpenAPI/Swagger intégrée
- Sécurité et bonnes pratiques (CORS, validation, gestion d’erreurs)
- Tests d’intégration et unitaires

## Structure du projet

```text
src/
│   main.rs                # Point d’entrée, initialise la config et lance le serveur
│
├── api/                   # Définition des routes HTTP, Swagger, middlewares d’authentification
│   ├── mod.rs             # Configuration des routes principales
│   ├── swagger.rs         # Documentation OpenAPI/Swagger
│   ├── auth/              # Contrôleurs et routes d’authentification
│   ├── health/            # Contrôleur de healthcheck
│   └── protected/         # Routes protégées par authentification
│
├── config/                # Gestion de la configuration de l’application
│   ├── config.rs          # Chargement des variables d’environnement
│   ├── server.rs          # Démarrage et configuration du serveur Actix
│   ├── models.rs          # Modèles de configuration (DB, JWT, etc.)
│   └── ...
│
├── core/                  # Composants génériques et middlewares globaux
│   ├── base/              # Repository générique, query builder
│   ├── errors/            # Gestion centralisée des erreurs
│   └── middlewares/       # Middlewares globaux (logger, etc.)
│
├── db/                    # Accès aux données et modèles SQLx
│   ├── connection.rs      # Connexion à la base PostgreSQL
│   ├── models/            # Modèles de données (User, etc.)
│   └── repositories/      # Repositories SQLx pour chaque entité
│
├── modules/               # Logique métier (services, helpers)
│   ├── auth/              # Service d’authentification, helpers JWT
│   └── user/              # Service de gestion des utilisateurs
│
└── ...

migrations/                # Scripts SQL de migration (création de tables, index, etc.)
tests/                     # Tests d’intégration et utilitaires de test

Cargo.toml                 # Dépendances et configuration du projet Rust
README.md                  # Documentation du projet
.docker-compose.yml        # (Optionnel) Configuration Docker pour la base de données
```

Chaque dossier est organisé pour séparer clairement la logique métier, l’accès aux données, la configuration, et l’API, facilitant la maintenance et l’extensibilité.

## Installation

1. **Prérequis** :
   - Rust (édition 2024)
   - PostgreSQL
   - Docker (optionnel, pour la base de données)

2. **Cloner le projet** :

   ```bash
   git clone <repo_url>
   cd rust_api_template
   ```

3. **Configurer les variables d’environnement** :
   Créez un fichier `.env` à la racine :

   ```env
   SERVER_HOST=127.0.0.1
   SERVER_PORT=8080
   ENVIRONMENT=development
   DATABASE_URL=postgres://user:password@localhost:5432/dbname
   JWT_SECRET=your_jwt_secret
   JWT_EXPIRATION=86400
   JWT_REFRESH_SECRET=your_refresh_secret
   JWT_REFRESH_EXPIRATION=604800
   ```

4. **Lancer les migrations** :

   ```bash
   cargo install sqlx-cli
   sqlx migrate run
   ```

5. **Démarrer le serveur** :

   ```bash
   cargo run
   ```

## Utilisation

- L’API démarre sur `http://127.0.0.1:8080` (modifiable via `.env`)
- Documentation Swagger disponible sur `/swagger-ui/`
- Endpoints principaux :
  - `POST /api/auth/register` : inscription utilisateur
  - `POST /api/auth/login` : authentification
  - `GET /api/protected/users` : liste des utilisateurs (protégé)
  - `GET /api/health` : healthcheck

## Tests

Lancez les tests d’intégration :

```bash
cargo test
```

## Sécurité

- Hashage des mots de passe avec Bcrypt
- Authentification JWT (access/refresh)
- Validation des entrées (validator)
- Middleware de logging et gestion d’erreurs centralisée
- CORS configuré pour le développement

## Contribution

1. Forkez le repo
2. Créez une branche
3. Commitez vos modifications
4. Poussez la branche
5. Ouvrez une Pull Request

