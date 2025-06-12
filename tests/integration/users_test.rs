use crate::common::setup_test_db;
use rust_actix_api_template::models::user::User;
use rust_actix_api_template::repositories::user_repository::UserRepository;
use uuid::Uuid;

#[tokio::test]
async fn test_create_and_find_user() {
    // Configuration de la base de données de test
    let pool = setup_test_db().await;

    // Création du repository
    let repo = UserRepository::new(pool.clone());

    // Création d'un utilisateur
    let user = User::new(
        "testuser".to_string(),
        "test@example.com".to_string(),
        "hashed_password".to_string(),
    );

    // Enregistrement de l'utilisateur
    let created_user = repo.create(user).await.expect("Failed to create user");

    // Recherche de l'utilisateur par ID
    let found_user = repo
        .find_by_id(created_user.id)
        .await
        .expect("Failed to find user")
        .expect("User not found");

    // Vérification que l'utilisateur trouvé est le même que celui créé
    assert_eq!(found_user.username, "testuser");
    assert_eq!(found_user.email, "test@example.com");

    // Nettoyage
    repo.delete(created_user.id)
        .await
        .expect("Failed to delete user");
}

#[tokio::test]
async fn test_update_user() {
    // Configuration de la base de données de test
    let pool = setup_test_db().await;

    // Création du repository
    let repo = UserRepository::new(pool.clone());

    // Création d'un utilisateur
    let user = User::new(
        "updateuser".to_string(),
        "update@example.com".to_string(),
        "hashed_password".to_string(),
    );

    // Enregistrement de l'utilisateur
    let created_user = repo.create(user).await.expect("Failed to create user");

    // Modification de l'utilisateur
    let mut user_to_update = created_user;
    user_to_update.username = "updated".to_string();

    // Mise à jour de l'utilisateur
    let updated_user = repo
        .update(user_to_update)
        .await
        .expect("Failed to update user");

    // Vérification que l'utilisateur a bien été mis à jour
    assert_eq!(updated_user.username, "updated");

    // Nettoyage
    repo.delete(updated_user.id)
        .await
        .expect("Failed to delete user");
}

#[tokio::test]
async fn test_delete_user() {
    // Configuration de la base de données de test
    let pool = setup_test_db().await;

    // Création du repository
    let repo = UserRepository::new(pool.clone());

    // Création d'un utilisateur
    let user = User::new(
        "deleteuser".to_string(),
        "delete@example.com".to_string(),
        "hashed_password".to_string(),
    );

    // Enregistrement de l'utilisateur
    let created_user = repo.create(user).await.expect("Failed to create user");

    // Suppression de l'utilisateur
    let deleted = repo
        .delete(created_user.id)
        .await
        .expect("Failed to delete user");
    assert!(deleted);

    // Vérification que l'utilisateur a bien été supprimé
    let not_found = repo
        .find_by_id(created_user.id)
        .await
        .expect("Failed to query user");
    assert!(not_found.is_none());
}

#[tokio::test]
async fn test_find_by_email() {
    // Configuration de la base de données de test
    let pool = setup_test_db().await;

    // Création du repository
    let repo = UserRepository::new(pool.clone());

    // Création d'un utilisateur
    let user = User::new(
        "emailuser".to_string(),
        "unique@example.com".to_string(),
        "hashed_password".to_string(),
    );

    // Enregistrement de l'utilisateur
    let created_user = repo.create(user).await.expect("Failed to create user");

    // Recherche de l'utilisateur par email
    let found_user = repo
        .find_by_email("unique@example.com")
        .await
        .expect("Failed to find user by email")
        .expect("User not found by email");

    // Vérification que l'utilisateur trouvé est le même que celui créé
    assert_eq!(found_user.id, created_user.id);

    // Nettoyage
    repo.delete(created_user.id)
        .await
        .expect("Failed to delete user");
}
