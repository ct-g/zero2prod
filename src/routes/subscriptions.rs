use axum::{
    Extension,
    Form,
    http::StatusCode,
};
use chrono::Utc;
use serde::Deserialize;
use sqlx::PgPool;
use uuid::Uuid;

use axum_macros::debug_handler;

use std::sync::Arc;

#[derive(Deserialize)]
pub struct FormData {
    email: String,
    name: String,
}

#[debug_handler]
pub async fn subscribe(
    Extension(pool): Extension<Arc<PgPool>>,
    form: Form<FormData>, // Form must be last extractor, otherwise opaque error prevents compilation
) -> StatusCode {
    match sqlx::query!(
        r#"
        INSERT INTO subscriptions (id, email, name, subscribed_at)
        VALUES ($1, $2, $3, $4)
        "#,
        Uuid::new_v4(),
        form.email,
        form.name,
        Utc::now()
    )
    .execute(pool.as_ref())
    .await
    {
        Ok(_) => StatusCode::OK,
        Err(e) => {
            println!("Failed to execute query: {}", e);
            StatusCode::INTERNAL_SERVER_ERROR
        },
    }
}