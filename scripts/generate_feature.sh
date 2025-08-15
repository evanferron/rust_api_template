#!/bin/bash

# Script de génération automatique de fonctionnalités
# Ce script crée tous les fichiers nécessaires pour une nouvelle fonctionnalité

set -e

# Couleurs pour les messages
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Fonction pour afficher des messages colorés
print_info() {
    echo -e "${BLUE}[INFO]${NC} $1"
}

print_success() {
    echo -e "${GREEN}[SUCCESS]${NC} $1"
}

print_warning() {
    echo -e "${YELLOW}[WARNING]${NC} $1"
}

print_error() {
    echo -e "${RED}[ERROR]${NC} $1"
}

# Fonction pour convertir en snake_case
to_snake_case() {
    echo "$1" | sed 's/\([A-Z]\)/_\L\1/g' | sed 's/^_//' | tr '[:upper:]' '[:lower:]'
}

# Fonction pour convertir en PascalCase
to_pascal_case() {
    echo "$1" | sed 's/_\([a-z]\)/\U\1/g' | sed 's/^\([a-z]\)/\U\1/'
}

# Fonction pour convertir en camelCase
to_camel_case() {
    echo "$1" | sed 's/_\([a-z]\)/\U\1/g'
}

# Fonction pour convertir au pluriel (simple)
to_plural() {
    local word="$1"
    if [[ "$word" =~ [sxz]$ ]] || [[ "$word" =~ [sh|ch]$ ]]; then
        echo "${word}es"
    elif [[ "$word" =~ [^aeiou]y$ ]]; then
        echo "${word%y}ies"
    else
        echo "${word}s"
    fi
}

# Vérification que nous sommes dans le bon répertoire
if [[ ! -f "Cargo.toml" ]]; then
    print_error "Ce script doit être exécuté depuis la racine du projet Rust (où se trouve Cargo.toml)"
    exit 1
fi

print_info "🚀 Générateur de fonctionnalités pour l'API Rust"
echo

# Collecte des informations
read -p "Nom de la fonctionnalité (ex: message, notification): " feature_name

if [[ -z "$feature_name" ]]; then
    print_error "Le nom de la fonctionnalité est requis"
    exit 1
fi

# Conversion des noms
feature_snake=$(to_snake_case "$feature_name")
feature_pascal=$(to_pascal_case "$feature_snake")
feature_camel=$(to_camel_case "$feature_snake")
feature_plural=$(to_plural "$feature_snake")

print_info "Nom snake_case: $feature_snake"
print_info "Nom PascalCase: $feature_pascal"
print_info "Nom camelCase: $feature_camel"
print_info "Nom pluriel: $feature_plural"

# Demander les champs du modèle
echo
print_info "Définition des champs du modèle (en plus des champs par défaut: id, created_at, updated_at)"
print_info "Format: nom_champ:type_rust (ex: title:String, user_id:Uuid)"
print_info "Types disponibles: String, i32, i64, f64, bool, Uuid, Option<String>, Vec<u8>, etc."
print_info "Appuyez sur Entrée pour terminer la saisie"

fields=()
while true; do
    read -p "Champ $(( ${#fields[@]} + 1 )) (ou Entrée pour terminer): " field
    if [[ -z "$field" ]]; then
        break
    fi
    
    if [[ "$field" =~ ^[a-zA-Z_][a-zA-Z0-9_]*:.+$ ]]; then
        fields+=("$field")
    else
        print_warning "Format invalide. Utilisez: nom_champ:type_rust"
    fi
done

if [[ ${#fields[@]} -eq 0 ]]; then
    print_warning "Aucun champ personnalisé défini. Seuls les champs par défaut seront utilisés."
fi

# Demander si c'est une entité avec relations
echo
read -p "Cette entité a-t-elle des relations avec d'autres entités? (y/N): " has_relations
has_relations=${has_relations:-n}

foreign_keys=()
if [[ "$has_relations" =~ ^[yY]$ ]]; then
    print_info "Définition des clés étrangères"
    print_info "Format: nom_champ:entité_référencée (ex: user_id:user, channel_id:channel)"
    
    while true; do
        read -p "Clé étrangère $(( ${#foreign_keys[@]} + 1 )) (ou Entrée pour terminer): " fk
        if [[ -z "$fk" ]]; then
            break
        fi
        
        if [[ "$fk" =~ ^[a-zA-Z_][a-zA-Z0-9_]*:[a-zA-Z_][a-zA-Z0-9_]*$ ]]; then
            foreign_keys+=("$fk")
        else
            print_warning "Format invalide. Utilisez: nom_champ:entité_référencée"
        fi
    done
fi

# Confirmation
echo
print_info "Résumé de la fonctionnalité à créer:"
echo "  - Nom: $feature_snake"
echo "  - Champs personnalisés: ${#fields[@]}"
echo "  - Clés étrangères: ${#foreign_keys[@]}"
echo

read -p "Confirmer la création? (y/N): " confirm
confirm=${confirm:-n}

if [[ ! "$confirm" =~ ^[yY]$ ]]; then
    print_info "Opération annulée"
    exit 0
fi

print_info "🔧 Création de la fonctionnalité '$feature_snake'..."

# Création des répertoires
mkdir -p "src/db/models"
mkdir -p "src/db/repositories"
mkdir -p "src/modules/$feature_snake"
mkdir -p "src/api/protected/$feature_snake"

# Génération du contenu des champs pour le modèle
model_fields=""
new_fields=""
columns_list=""
field_assignments=""

for field in "${fields[@]}"; do
    IFS=':' read -r field_name field_type <<< "$field"
    model_fields="$model_fields    pub $field_name: $field_type,\n"
    new_fields="$new_fields        $field_name,\n"
    columns_list="$columns_list            \"$field_name\",\n"
    
    if [[ "$field_type" == "Option<"* ]]; then
        field_assignments="$field_assignments        $field_name: None,\n"
    elif [[ "$field_type" == "String" ]]; then
        field_assignments="$field_assignments        $field_name: String::new(),\n"
    elif [[ "$field_type" == "Uuid" ]]; then
        field_assignments="$field_assignments        $field_name: Uuid::new_v4(),\n"
    else
        field_assignments="$field_assignments        $field_name: Default::default(),\n"
    fi
done

# Génération des paramètres du constructeur
constructor_params=""
constructor_assignments=""
if [[ ${#fields[@]} -gt 0 ]]; then
    for field in "${fields[@]}"; do
        IFS=':' read -r field_name field_type <<< "$field"
        constructor_params="$constructor_params$field_name: $field_type, "
        constructor_assignments="$constructor_assignments            $field_name,\n"
    done
    constructor_params="${constructor_params%, }"
fi

# 1. Génération du modèle de base de données
print_info "📄 Création du modèle de base de données..."

cat > "src/db/models/${feature_snake}.rs" << EOF
use crate::core::base::generic_repository::entry_trait::Entry;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use utoipa::ToSchema;
use uuid::Uuid;

#[derive(Debug, Serialize, Deserialize, FromRow, ToSchema)]
pub struct $feature_pascal {
    pub id: Uuid,
$(echo -e "$model_fields")    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl $feature_pascal {
    pub fn new($constructor_params) -> Self {
        let now = Utc::now();
        Self {
            id: Uuid::new_v4(),
$(echo -e "$constructor_assignments")            created_at: now,
            updated_at: now,
        }
    }
}

impl Entry for $feature_pascal {
    type Id = Uuid;

    fn set_created_at(&mut self, created_at: DateTime<Utc>) {
        self.created_at = created_at;
    }

    fn set_updated_at(&mut self, updated_at: DateTime<Utc>) {
        self.updated_at = updated_at;
    }

    fn table_name() -> &'static str {
        "$feature_plural"
    }

    fn columns() -> Vec<&'static str> {
        vec![
            "id",
$(echo -e "$columns_list")            "created_at",
            "updated_at",
        ]
    }
}
EOF

# 2. Génération du repository
print_info "🗄️  Création du repository..."

cat > "src/db/repositories/${feature_snake}_repository.rs" << EOF
use crate::core::base::generic_repository::repository_trait::RepositoryTrait;
use crate::core::errors::errors::ApiError;
use crate::db::models::${feature_snake}::${feature_pascal};
use sqlx::{Pool, Postgres};
use uuid::Uuid;

#[derive(Clone)]
pub struct ${feature_pascal}Repository {
    pool: Pool<Postgres>,
}

impl ${feature_pascal}Repository {
    pub fn new(pool: Pool<Postgres>) -> Self {
        Self { pool }
    }

    // Méthodes spécifiques à l'entité (à personnaliser selon vos besoins)
    
    // Exemple: recherche par un champ spécifique
    // pub async fn find_by_field(&self, field_value: &str) -> Result<Option<${feature_pascal}>, ApiError> {
    //     let ${feature_snake} = self.find_by_column("field_name", field_value).await?;
    //     Ok(${feature_snake}.into_iter().next())
    // }
}

impl RepositoryTrait<${feature_pascal}> for ${feature_pascal}Repository {
    fn get_pool(&self) -> &Pool<Postgres> {
        &self.pool
    }
}

// Implémentation passerelle pour simplifier l'utilisation
impl ${feature_pascal}Repository {
    pub async fn find_all_${feature_plural}(&self) -> Result<Vec<${feature_pascal}>, ApiError> {
        self.find_all().await
    }

    pub async fn find_${feature_snake}_by_id(&self, id: Uuid) -> Result<Option<${feature_pascal}>, ApiError> {
        self.find_by_id(id).await
    }

    pub async fn create_${feature_snake}(&self, ${feature_snake}: ${feature_pascal}) -> Result<${feature_pascal}, ApiError> {
        self.create(${feature_snake}).await
    }

    pub async fn update_${feature_snake}(&self, id: Uuid, ${feature_snake}: ${feature_pascal}) -> Result<${feature_pascal}, ApiError> {
        self.update(id, ${feature_snake}).await
    }
}
EOF

# 3. Génération des modèles du module
print_info "📋 Création des modèles du module..."

# Génération des champs pour les requêtes
create_fields=""
update_fields=""
for field in "${fields[@]}"; do
    IFS=':' read -r field_name field_type <<< "$field"
    
    # Pour CreateRequest, tous les champs requis
    if [[ "$field_type" == "Option<"* ]]; then
        create_fields="$create_fields    pub $field_name: $field_type,\n"
    else
        create_fields="$create_fields    #[validate(length(min = 1))]\n    pub $field_name: $field_type,\n"
    fi
    
    # Pour UpdateRequest, tous les champs optionnels
    if [[ "$field_type" == "Option<"* ]]; then
        update_fields="$update_fields    pub $field_name: $field_type,\n"
    else
        update_fields="$update_fields    pub $field_name: Option<$field_type>,\n"
    fi
done

cat > "src/modules/${feature_snake}/${feature_snake}_models.rs" << EOF
use crate::db::models::${feature_snake}::${feature_pascal};
use serde::{Deserialize, Serialize};
use utoipa::{IntoParams, ToSchema};
use uuid::Uuid;
use validator::Validate;

#[derive(Debug, Serialize, ToSchema)]
pub struct ${feature_pascal}Response {
    pub id: Uuid,
$(echo -e "$model_fields")    pub created_at: String,
    pub updated_at: String,
}

impl From<${feature_pascal}> for ${feature_pascal}Response {
    fn from(${feature_snake}: ${feature_pascal}) -> Self {
        Self {
            id: ${feature_snake}.id,
$(for field in "${fields[@]}"; do
    IFS=':' read -r field_name field_type <<< "$field"
    echo "            $field_name: ${feature_snake}.$field_name,"
done)
            created_at: ${feature_snake}.created_at.to_rfc3339(),
            updated_at: ${feature_snake}.updated_at.to_rfc3339(),
        }
    }
}

#[derive(Debug, Deserialize, Validate, ToSchema)]
pub struct Create${feature_pascal}Request {
$(echo -e "$create_fields")}

#[derive(Debug, Deserialize, Validate, ToSchema)]
pub struct Update${feature_pascal}Request {
$(echo -e "$update_fields")}

#[derive(Debug, Deserialize, IntoParams)]
pub struct ${feature_pascal}IdPath {
    pub id: Uuid,
}

#[derive(Debug, Deserialize, IntoParams)]
pub struct ${feature_pascal}QueryParams {
    #[serde(default)]
    pub page: Option<i32>,
    #[serde(default)]
    pub limit: Option<i32>,
    #[serde(default)]
    pub search: Option<String>,
}
EOF

# 4. Génération du service
print_info "⚙️  Création du service..."

cat > "src/modules/${feature_snake}/${feature_snake}_service.rs" << EOF
use crate::config::models::Repositories;
use crate::core::errors::errors::ApiError;
use crate::db::models::${feature_snake}::${feature_pascal};
use crate::modules::${feature_snake}::${feature_snake}_models::{Create${feature_pascal}Request, Update${feature_pascal}Request};
use std::sync::Arc;
use uuid::Uuid;

#[derive(Clone)]
pub struct ${feature_pascal}Service {
    pub repositories: Arc<Repositories>,
}

impl ${feature_pascal}Service {
    pub fn new(repositories: Arc<Repositories>) -> Self {
        ${feature_pascal}Service { repositories }
    }

    pub async fn get_${feature_plural}(&self) -> Result<Vec<${feature_pascal}>, ApiError> {
        self.repositories.${feature_snake}_repository.find_all_${feature_plural}().await
    }

    pub async fn get_${feature_snake}_by_id(&self, id: Uuid) -> Result<${feature_pascal}, ApiError> {
        let ${feature_snake} = self
            .repositories
            .${feature_snake}_repository
            .find_${feature_snake}_by_id(id)
            .await?;

        match ${feature_snake} {
            Some(${feature_snake}) => Ok(${feature_snake}),
            None => Err(ApiError::NotFound(format!(
                "${feature_pascal} avec l'ID {} non trouvé",
                id
            ))),
        }
    }

    pub async fn create_${feature_snake}(&self, request: Create${feature_pascal}Request) -> Result<${feature_pascal}, ApiError> {
        // Validation métier personnalisée ici si nécessaire
        
        let ${feature_snake} = ${feature_pascal}::new(
$(for field in "${fields[@]}"; do
    IFS=':' read -r field_name field_type <<< "$field"
    echo "            request.$field_name,"
done)
        );

        self.repositories
            .${feature_snake}_repository
            .create_${feature_snake}(${feature_snake})
            .await
    }

    pub async fn update_${feature_snake}(&self, id: Uuid, request: Update${feature_pascal}Request) -> Result<${feature_pascal}, ApiError> {
        // Vérifier que l'entité existe
        let mut ${feature_snake} = self.get_${feature_snake}_by_id(id).await?;

        // Mettre à jour les champs modifiés
$(for field in "${fields[@]}"; do
    IFS=':' read -r field_name field_type <<< "$field"
    echo "        if let Some(new_$field_name) = request.$field_name {"
    echo "            ${feature_snake}.$field_name = new_$field_name;"
    echo "        }"
done)

        self.repositories
            .${feature_snake}_repository
            .update_${feature_snake}(id, ${feature_snake})
            .await
    }

    pub async fn delete_${feature_snake}(&self, id: Uuid) -> Result<(), ApiError> {
        // Vérifier que l'entité existe
        self.get_${feature_snake}_by_id(id).await?;

        self.repositories
            .${feature_snake}_repository
            .delete_${feature_snake}(id)
            .await
    }
}
EOF

# 5. Génération du contrôleur
print_info "🎮 Création du contrôleur..."

cat > "src/api/protected/${feature_snake}/${feature_snake}_controller.rs" << EOF
use crate::config::models::Services;
use crate::core::errors::errors::{ApiError, ErrorResponse};
use crate::modules::${feature_snake}::${feature_snake}_models::{
    Create${feature_pascal}Request, Update${feature_pascal}Request, ${feature_pascal}IdPath, ${feature_pascal}Response,
};
use actix_web::{HttpResponse, Responder, delete, get, post, put, web};
use validator::Validate;

#[utoipa::path(
    get,
    path = "/api/protected/${feature_snake}",
    tag = "${feature_plural}",
    responses(
        (status = 200, description = "Liste des ${feature_plural}", body = Vec<${feature_pascal}Response>),
        (status = 500, description = "Erreur interne du serveur", body = ErrorResponse)
    )
)]
#[get("")]
pub async fn get_${feature_plural}(services: web::Data<Services>) -> Result<impl Responder, ApiError> {
    let ${feature_plural} = services.${feature_snake}_service.get_${feature_plural}().await?;

    let ${feature_snake}_responses: Vec<${feature_pascal}Response> = ${feature_plural}.into_iter().map(|item| item.into()).collect();

    Ok(web::Json(${feature_snake}_responses))
}

#[utoipa::path(
    get,
    path = "/api/protected/${feature_snake}/{id}",
    tag = "${feature_plural}",
    params(
        ("id" = Uuid, Path, description = "ID du ${feature_snake}")
    ),
    responses(
        (status = 200, description = "${feature_pascal} trouvé", body = ${feature_pascal}Response),
        (status = 404, description = "${feature_pascal} non trouvé", body = ErrorResponse),
        (status = 500, description = "Erreur interne du serveur", body = ErrorResponse)
    )
)]
#[get("/{id}")]
pub async fn get_${feature_snake}_by_id(
    path: web::Path<${feature_pascal}IdPath>,
    services: web::Data<Services>,
) -> Result<impl Responder, ApiError> {
    let ${feature_snake} = services.${feature_snake}_service.get_${feature_snake}_by_id(path.id).await?;
    Ok(web::Json(${feature_pascal}Response::from(${feature_snake})))
}

#[utoipa::path(
    post,
    path = "/api/protected/${feature_snake}",
    tag = "${feature_plural}",
    request_body = Create${feature_pascal}Request,
    responses(
        (status = 201, description = "${feature_pascal} créé avec succès", body = ${feature_pascal}Response),
        (status = 400, description = "Données invalides", body = ErrorResponse),
        (status = 409, description = "Conflit - ${feature_pascal} existe déjà", body = ErrorResponse),
        (status = 500, description = "Erreur interne du serveur", body = ErrorResponse)
    )
)]
#[post("")]
pub async fn create_${feature_snake}(
    ${feature_snake}_data: web::Json<Create${feature_pascal}Request>,
    services: web::Data<Services>,
) -> Result<impl Responder, ApiError> {
    // Validation des données
    ${feature_snake}_data.validate().map_err(|e| {
        ApiError::BadRequest(format!("Données invalides: {}", e))
    })?;

    let ${feature_snake} = services
        .${feature_snake}_service
        .create_${feature_snake}(${feature_snake}_data.into_inner())
        .await?;

    Ok(HttpResponse::Created().json(${feature_pascal}Response::from(${feature_snake})))
}

#[utoipa::path(
    put,
    path = "/api/protected/${feature_snake}/{id}",
    tag = "${feature_plural}",
    params(
        ("id" = Uuid, Path, description = "ID du ${feature_snake}")
    ),
    request_body = Update${feature_pascal}Request,
    responses(
        (status = 200, description = "${feature_pascal} mis à jour avec succès", body = ${feature_pascal}Response),
        (status = 400, description = "Données invalides", body = ErrorResponse),
        (status = 404, description = "${feature_pascal} non trouvé", body = ErrorResponse),
        (status = 500, description = "Erreur interne du serveur", body = ErrorResponse)
    )
)]
#[put("/{id}")]
pub async fn update_${feature_snake}(
    path: web::Path<${feature_pascal}IdPath>,
    ${feature_snake}_data: web::Json<Update${feature_pascal}Request>,
    services: web::Data<Services>,
) -> Result<impl Responder, ApiError> {
    // Validation des données
    ${feature_snake}_data.validate().map_err(|e| {
        ApiError::BadRequest(format!("Données invalides: {}", e))
    })?;

    let ${feature_snake} = services
        .${feature_snake}_service
        .update_${feature_snake}(path.id, ${feature_snake}_data.into_inner())
        .await?;

    Ok(web::Json(${feature_pascal}Response::from(${feature_snake})))
}

#[utoipa::path(
    delete,
    path = "/api/protected/${feature_snake}/{id}",
    tag = "${feature_plural}",
    params(
        ("id" = Uuid, Path, description = "ID du ${feature_snake}")
    ),
    responses(
        (status = 204, description = "${feature_pascal} supprimé avec succès"),
        (status = 404, description = "${feature_pascal} non trouvé", body = ErrorResponse),
        (status = 500, description = "Erreur interne du serveur", body = ErrorResponse)
    )
)]
#[delete("/{id}")]
pub async fn delete_${feature_snake}(
    path: web::Path<${feature_pascal}IdPath>,
    services: web::Data<Services>,
) -> Result<impl Responder, ApiError> {
    services
        .${feature_snake}_service
        .delete_${feature_snake}(path.id)
        .await?;

    Ok(HttpResponse::NoContent().finish())
}
EOF

# 6. Génération des fichiers mod.rs
print_info "🔗 Création des fichiers de module..."

# Module service
cat > "src/modules/${feature_snake}/mod.rs" << EOF
pub mod ${feature_snake}_models;
pub mod ${feature_snake}_service;
EOF

# Module API
cat > "src/api/protected/${feature_snake}/mod.rs" << EOF
use actix_web::web;
pub mod ${feature_snake}_controller;

pub fn routes_config(cfg: &mut web::ServiceConfig) {
    cfg.service(${feature_snake}_controller::get_${feature_plural})
        .service(${feature_snake}_controller::get_${feature_snake}_by_id)
        .service(${feature_snake}_controller::create_${feature_snake})
        .service(${feature_snake}_controller::update_${feature_snake})
        .service(${feature_snake}_controller::delete_${feature_snake});
}
EOF

# 7. Génération du fichier de migration SQL
print_info "🗃️  Création du fichier de migration..."

mkdir -p "migrations"

# Génération des champs SQL
sql_fields=""
for field in "${fields[@]}"; do
    IFS=':' read -r field_name field_type <<< "$field"
    
    sql_type=""
    case "$field_type" in
        "String") sql_type="VARCHAR NOT NULL" ;;
        "Option<String>") sql_type="VARCHAR" ;;
        "i32") sql_type="INTEGER NOT NULL" ;;
        "Option<i32>") sql_type="INTEGER" ;;
        "i64") sql_type="BIGINT NOT NULL" ;;
        "Option<i64>") sql_type="BIGINT" ;;
        "f64") sql_type="DOUBLE PRECISION NOT NULL" ;;
        "Option<f64>") sql_type="DOUBLE PRECISION" ;;
        "bool") sql_type="BOOLEAN NOT NULL DEFAULT FALSE" ;;
        "Option<bool>") sql_type="BOOLEAN" ;;
        "Uuid") sql_type="UUID NOT NULL" ;;
        "Option<Uuid>") sql_type="UUID" ;;
        "Vec<u8>") sql_type="BYTEA" ;;
        "Option<Vec<u8>>") sql_type="BYTEA" ;;
        *) sql_type="TEXT" ;;
    esac
    
    sql_fields="$sql_fields    $field_name $sql_type,\n"
done

# Génération des clés étrangères
fk_constraints=""
for fk in "${foreign_keys[@]}"; do
    IFS=':' read -r fk_field fk_table <<< "$fk"
    fk_table_plural=$(to_plural "$fk_table")
    fk_constraints="$fk_constraints    FOREIGN KEY ($fk_field) REFERENCES $fk_table_plural(id) ON DELETE CASCADE,\n"
done

timestamp=$(date +"%Y%m%d%H%M%S")
cat > "migrations/${timestamp}_create_${feature_plural}_table.sql" << EOF
-- Migration pour créer la table ${feature_plural}
-- Générée automatiquement le $(date)

CREATE TABLE IF NOT EXISTS ${feature_plural} (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
$(echo -e "$sql_fields")    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()$(if [[ -n "$fk_constraints" ]]; then echo ","; fi)
$(if [[ -n "$fk_constraints" ]]; then echo -e "$fk_constraints"; fi)
);

-- Index pour optimiser les performances
CREATE INDEX IF NOT EXISTS idx_${feature_plural}_created_at ON ${feature_plural}(created_at);
CREATE INDEX IF NOT EXISTS idx_${feature_plural}_updated_at ON ${feature_plural}(updated_at);

$(for fk in "${foreign_keys[@]}"; do
    IFS=':' read -r fk_field fk_table <<< "$fk"
    echo "CREATE INDEX IF NOT EXISTS idx_${feature_plural}_${fk_field} ON ${feature_plural}(${fk_field});"
done)

-- Trigger pour mettre à jour automatiquement updated_at
CREATE OR REPLACE FUNCTION update_updated_at_column()
RETURNS TRIGGER AS \$\$
BEGIN
    NEW.updated_at = NOW();
    RETURN NEW;
END;
\$\$ language 'plpgsql';

CREATE TRIGGER update_${feature_plural}_updated_at 
    BEFORE UPDATE ON ${feature_plural} 
    FOR EACH ROW EXECUTE FUNCTION update_updated_at_column();
EOF

# 8. Mise à jour des fichiers mod.rs principaux
print_info "🔄 Mise à jour des fichiers de modules principaux..."

# Ajouter le modèle aux models/mod.rs
if ! grep -q "pub mod ${feature_snake};" "src/db/models/mod.rs" 2>/dev/null; then
    echo "pub mod ${feature_snake};" >> "src/db/models/mod.rs"
fi

# Ajouter le repository aux repositories/mod.rs
if ! grep -q "pub mod ${feature_snake}_repository;" "src/db/repositories/mod.rs" 2>/dev/null; then
    echo "pub mod ${feature_snake}_repository;" >> "src/db/repositories/mod.rs"
fi

# Ajouter le module aux modules/mod.rs
if ! grep -q "pub mod ${feature_snake};" "src/modules/mod.rs" 2>/dev/null; then
    echo "pub mod ${feature_snake};" >> "src/modules/mod.rs"
fi

# 9. Instructions finales
print_success "✅ Fonctionnalité '$feature_snake' créée avec succès!"
echo
print_info "📋 Fichiers créés:"
echo "  - src/db/models/${feature_snake}.rs"
echo "  - src/db/repositories/${feature_snake}_repository.rs"
echo "  - src/modules/${feature_snake}/${feature_snake}_models.rs"
echo "  - src/modules/${feature_snake}/${feature_snake}_service.rs"
echo "  - src/modules/${feature_snake}/mod.rs"
echo "  - src/api/protected/${feature_snake}/${feature_snake}_controller.rs"
echo "  - src/api/protected/${feature_snake}/mod.rs"
echo "  - migrations/${timestamp}_create_${feature_plural}_table.sql"
echo
print_warning "📝 Actions manuelles requises:"
echo "  1. Ajouter le repository à 'src/config/models.rs' dans la struct Repositories"
echo "  2. Ajouter le service à 'src/config/models.rs' dans la struct Services"
echo "  3. Ajouter les routes dans 'src/api/protected/mod.rs'"
echo "  4. Exécuter la migration SQL: sqlx migrate run"
echo "  5. Compiler le projet: cargo build"
echo "  6. Personnaliser les validations et la logique métier selon vos besoins"
echo
print_info "🚀 Votre nouvelle fonctionnalité est prête à être configurée!"
