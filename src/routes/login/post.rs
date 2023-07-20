use crate::authentication::{validate_credentials, Credentials, AuthError};
use crate::routes::error_chain_fmt;

use axum::{
    Extension,
    Form,
    http::{
        header::{self, HeaderValue},
        StatusCode
    },
    response::{IntoResponse, Response}
};
use secrecy::Secret;
use sqlx::PgPool;

use std::sync::Arc;

#[derive(serde::Deserialize)]
pub struct FormData {
    username: String,
    password: Secret<String>,
}

#[derive(thiserror::Error)]
pub enum LoginError {
    #[error("Authentication failed")]
    AuthError(#[source] anyhow::Error),
    #[error("Something went wrong")]
    UnexpectedError(#[from] anyhow::Error),
}

impl std::fmt::Debug for LoginError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        error_chain_fmt(self, f)
    }
}

impl IntoResponse for LoginError {
    fn into_response(self) -> Response {
        match self {
            LoginError::AuthError(_) => StatusCode::UNAUTHORIZED.into_response(),
            LoginError::UnexpectedError(_) => StatusCode::INTERNAL_SERVER_ERROR.into_response(),
        }
    }
}

#[tracing::instrument(
    skip(form, pool),
    fields(username=tracing::field::Empty, user_id=tracing::field::Empty)
)]
pub async fn login(
    Extension(pool): Extension<Arc<PgPool>>,
    form: Form<FormData>
) -> Result<Response, LoginError> {
    let credentials = Credentials {
        username: form.0.username,
        password: form.0.password,
    };
    tracing::Span::current()
        .record("username", &tracing::field::display(&credentials.username));
    let user_id = validate_credentials(credentials, &pool)
        .await
        .map_err(|e| match e {
            AuthError::InvalidCredentials(_) => LoginError::AuthError(e.into()),
            AuthError::UnexpectedError(_) => LoginError::UnexpectedError(e.into()),
        })?;
    tracing::Span::current()
        .record("user_id", &tracing::field::display(&user_id));
    Ok({
        let mut response = StatusCode::SEE_OTHER.into_response();
            let header_value = HeaderValue::from_str("/").unwrap();
            response
                .headers_mut()
                .insert(header::LOCATION, header_value);
            response
    })
}