use crate::authentication::UserId;

use anyhow::Context;
use axum::{
    http::{StatusCode, header::{self, HeaderMap, HeaderValue}},
    response::IntoResponse,
    Extension,
};
use sqlx::PgPool;
use uuid::Uuid;

use std::sync::Arc;

// TODO took shortcuts regarding error handling, differs from section 10.7.5.2
pub async fn admin_dashboard(
    Extension(user_id): Extension<UserId>,
    Extension(pool): Extension<Arc<PgPool>>,
) -> impl IntoResponse {
    let username = match get_username(*user_id, &pool).await {
        Ok(username) => username,
        Err(_) => return StatusCode::INTERNAL_SERVER_ERROR.into_response()
    };

    let mut headers = HeaderMap::new();
    headers.insert(
        header::CONTENT_TYPE,
        HeaderValue::from_str("text/html; charset=utf-8").unwrap(),
    );
    // Response
    (
        StatusCode::OK,
        headers,
        format!(
            r#"<!DOCTYPE html>
            <html lang="en">
            <head>
                <meta http-equiv="content-type" content="text/html; charset=utf-8">
                <title>Admin dashboard</title>
            </head>
            <body>
                <p>Welcome {username}!</p>
                <p>Available actions:</p>
                <ol>
                    <li><a href="/admin/password">Change password</a></li>
                    <li>
                        <form name="logoutForm" action="/admin/logout" method="post">
                        <input type="submit" value="Logout">
                        </form>
                    </li>
                </ol>
            </body>
            </html>"#,
    )).into_response()
}

#[tracing::instrument(name = "Get username", skip(pool))]
pub async fn get_username(
    user_id: Uuid,
    pool: &PgPool,
) -> Result<String, anyhow::Error> {
    let row = sqlx::query!(
        r#"
        SELECT username
        FROM users
        WHERE user_id = $1
        "#,
        user_id,
    )
    .fetch_one(pool)
    .await
    .context("Failed to perform a query to retrieve a username.")?;
    Ok(row.username)
}