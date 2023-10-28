use crate::session_state::TypedSession;

use anyhow::Context;
use axum::{
    http::{StatusCode, header::{self, HeaderMap, HeaderValue}},
    response::IntoResponse,
    Extension,
};
use axum_session::SessionRedisPool;
use sqlx::PgPool;
use uuid::Uuid;

use std::sync::Arc;

// TODO took shortcuts regarding error handling, differs from section 10.7.5.2
pub async fn admin_dashboard(
    Extension(pool): Extension<Arc<PgPool>>,
    session: TypedSession<SessionRedisPool>,
) -> impl IntoResponse {
    let username = match session.get_user_id() {
        Some(user_id) => match get_username(user_id, &pool).await {
            Ok(username) => username,
            Err(_) => return StatusCode::INTERNAL_SERVER_ERROR.into_response()
        },
        None => {
            let mut headers = HeaderMap::new();
            headers.insert(
                header::LOCATION,
                HeaderValue::from_str("/login").unwrap(),
            );
            return (StatusCode::SEE_OTHER, headers).into_response();
        },
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
</body>
</html>"#
    )).into_response()
}

#[tracing::instrument(name = "Get username", skip(pool))]
async fn get_username(
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