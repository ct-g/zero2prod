use crate::helpers::{spawn_app, assert_is_redirect_to};

#[tokio::test]
async fn an_error_flash_message_is_set_on_failures() {
    // Arrange
    let app = spawn_app().await;

    // Act
    let login_body = serde_json::json!({
        "username": "random-username",
        "password": "random-password",
    });
    let response = app.post_login(&login_body).await;
    // Assert
    assert_is_redirect_to(&response, "/login");

    // Act 2 - Check error message on empty user/password submission
    let html_page = app.get_login_html().await;
    // Assert
    assert!(html_page.contains(r#"<p><i>Authentication failed</i></p>"#));

    // Act 3 - Check no error on page refresh
    let html_page = app.get_login_html().await;
    // Assert
    assert!(!html_page.contains("Authentication failed"));
}