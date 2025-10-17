use crate::common::setup_test_db;
use actix_web::{App, test, web};
use rust_actix_api_template::api;
use rust_actix_api_template::api::v1::users::{CreateUserRequest, UserResponse};
use rust_actix_api_template::config::Config;
use serde_json::{Value, json};
use uuid::Uuid;

#[actix_web::test]
async fn test_create_user_api() {
    // Test database configuration
    let pool = setup_test_db().await;

    // Application configuration
    let mut app = test::init_service(
        App::new()
            .app_data(web::Data::new(pool.clone()))
            .app_data(web::Data::new(
                Config::from_env().expect("Failed to load config"),
            ))
            .configure(api::routes_config),
    )
    .await;

    // Create a user creation request
    let request_body = json!({
        "username": "apitestuser",
        "email": "apitest@example.com",
        "password": "securepassword123"
    });

    // Send the request
    let req = test::TestRequest::post()
        .uri("/api/v1/users")
        .set_json(&request_body)
        .to_request();

    let resp = test::call_service(&mut app, req).await;

    // Verify the response
    assert!(resp.status().is_success());

    // Extract the response
    let body = test::read_body(resp).await;
    let user_response: UserResponse = serde_json::from_slice(&body).unwrap();

    // Verify the data
    assert_eq!(user_response.username, "apitestuser");
    assert_eq!(user_response.email, "apitest@example.com");

    // Retrieve the ID for cleanup
    let user_id = user_response.id;

    // Test get user by id
    let req = test::TestRequest::get()
        .uri(&format!("/api/v1/users/{}", user_id))
        .to_request();

    let resp = test::call_service(&mut app, req).await;

    // Verify the response
    assert!(resp.status().is_success());

    // Cleanup - delete the user
    let req = test::TestRequest::delete()
        .uri(&format!("/api/v1/users/{}", user_id))
        .to_request();

    let resp = test::call_service(&mut app, req).await;
    assert!(resp.status().is_success());
}

#[actix_web::test]
async fn test_update_user_api() {
    // Test database configuration
    let pool = setup_test_db().await;

    // Application configuration
    let mut app = test::init_service(
        App::new()
            .app_data(web::Data::new(pool.clone()))
            .app_data(web::Data::new(
                Config::from_env().expect("Failed to load config"),
            ))
            .configure(api::routes_config),
    )
    .await;

    // Create a user
    let create_req = test::TestRequest::post()
        .uri("/api/v1/users")
        .set_json(json!({
            "username": "updatetestuser",
            "email": "updatetest@example.com",
            "password": "securepassword123"
        }))
        .to_request();

    let resp = test::call_service(&mut app, create_req).await;
    assert!(resp.status().is_success());

    let body = test::read_body(resp).await;
    let user_response: UserResponse = serde_json::from_slice(&body).unwrap();
    let user_id = user_response.id;

    // Update the user
    let update_req = test::TestRequest::put()
        .uri(&format!("/api/v1/users/{}", user_id))
        .set_json(json!({
            "username": "updateduser"
        }))
        .to_request();

    let resp = test::call_service(&mut app, update_req).await;
    assert!(resp.status().is_success());

    // Verify the update
    let body = test::read_body(resp).await;
    let updated_user: UserResponse = serde_json::from_slice(&body).unwrap();
    assert_eq!(updated_user.username, "updateduser");
    assert_eq!(updated_user.email, "updatetest@example.com");

    // Cleanup
    let delete_req = test::TestRequest::delete()
        .uri(&format!("/api/v1/users/{}", user_id))
        .to_request();

    let resp = test::call_service(&mut app, delete_req).await;
    assert!(resp.status().is_success());
}
