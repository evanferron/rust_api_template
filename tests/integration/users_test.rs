use crate::common::setup_test_db;
use rust_actix_api_template::models::user::User;
use rust_actix_api_template::repositories::user_repository::UserRepository;
use uuid::Uuid;

#[tokio::test]
async fn test_create_and_find_user() {
    // Test database configuration
    let pool = setup_test_db().await;

    // Create the repository
    let repo = UserRepository::new(pool.clone());

    // Create a user
    let user = User::new(
        "testuser".to_string(),
        "test@example.com".to_string(),
        "hashed_password".to_string(),
    );

    // Save the user
    let created_user = repo.create(user).await.expect("Failed to create user");

    // Find the user by ID
    let found_user = repo
        .find_by_id(created_user.id)
        .await
        .expect("Failed to find user")
        .expect("User not found");

    // Verify that the found user matches the created one
    assert_eq!(found_user.username, "testuser");
    assert_eq!(found_user.email, "test@example.com");

    // Cleanup
    repo.delete(created_user.id)
        .await
        .expect("Failed to delete user");
}

#[tokio::test]
async fn test_update_user() {
    // Test database configuration
    let pool = setup_test_db().await;

    // Create the repository
    let repo = UserRepository::new(pool.clone());

    // Create a user
    let user = User::new(
        "updateuser".to_string(),
        "update@example.com".to_string(),
        "hashed_password".to_string(),
    );

    // Save the user
    let created_user = repo.create(user).await.expect("Failed to create user");

    // Modify the user
    let mut user_to_update = created_user;
    user_to_update.username = "updated".to_string();

    // Update the user
    let updated_user = repo
        .update(user_to_update)
        .await
        .expect("Failed to update user");

    // Verify the user was updated
    assert_eq!(updated_user.username, "updated");

    // Cleanup
    repo.delete(updated_user.id)
        .await
        .expect("Failed to delete user");
}

#[tokio::test]
async fn test_delete_user() {
    // Test database configuration
    let pool = setup_test_db().await;

    // Create the repository
    let repo = UserRepository::new(pool.clone());

    // Create a user
    let user = User::new(
        "deleteuser".to_string(),
        "delete@example.com".to_string(),
        "hashed_password".to_string(),
    );

    // Save the user
    let created_user = repo.create(user).await.expect("Failed to create user");

    // Delete the user
    let deleted = repo
        .delete(created_user.id)
        .await
        .expect("Failed to delete user");
    assert!(deleted);

    // Verify the user was deleted
    let not_found = repo
        .find_by_id(created_user.id)
        .await
        .expect("Failed to query user");
    assert!(not_found.is_none());
}

#[tokio::test]
async fn test_find_by_email() {
    // Test database configuration
    let pool = setup_test_db().await;

    // Create the repository
    let repo = UserRepository::new(pool.clone());

    // Create a user
    let user = User::new(
        "emailuser".to_string(),
        "unique@example.com".to_string(),
        "hashed_password".to_string(),
    );

    // Save the user
    let created_user = repo.create(user).await.expect("Failed to create user");

    // Find the user by email
    let found_user = repo
        .find_by_email("unique@example.com")
        .await
        .expect("Failed to find user by email")
        .expect("User not found by email");

    // Verify the found user matches the created one
    assert_eq!(found_user.id, created_user.id);

    // Cleanup
    repo.delete(created_user.id)
        .await
        .expect("Failed to delete user");
}
