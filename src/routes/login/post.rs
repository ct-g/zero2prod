use crate::authentication::{validate_credentials, Credentials, AuthError};
use crate::routes::error_chain_fmt;

use axum::{
    Extension,
    Form,
    http::{
        header::{self, HeaderValue, HeaderMap},
        StatusCode
    },
    response::{IntoResponse, Response}
};
use axum_extra::extract::CookieJar;
use axum_extra::extract::cookie::Cookie;
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
        let encoded_error = urlencoding::Encoded::new(self.to_string());
        let mut response = StatusCode::SEE_OTHER.into_response();
        let header_value = HeaderValue::from_str(
            &format!("/login?error={}", encoded_error)
        ).unwrap();
        response
            .headers_mut()
            .insert(header::LOCATION, header_value);
        response
    }
}

#[tracing::instrument(
    skip(form, pool),
    fields(username=tracing::field::Empty, user_id=tracing::field::Empty)
)]
pub async fn login(
    Extension(pool): Extension<Arc<PgPool>>,
    form: Form<FormData>
) -> Result<Response, Response> {
    let credentials = Credentials {
        username: form.0.username,
        password: form.0.password,
    };
    tracing::Span::current()
        .record("username", &tracing::field::display(&credentials.username));

    match validate_credentials(credentials, &pool).await {
        Ok(user_id) => {
            tracing::Span::current()
                .record("user_id", &tracing::field::display(&user_id));
            let mut response = StatusCode::SEE_OTHER.into_response();
            let header_value = HeaderValue::from_str("/").unwrap();
            response
                .headers_mut()
                .insert(header::LOCATION, header_value);
            Ok(response)
        },
        Err(e) => {
            let e = match e {
                AuthError::InvalidCredentials(_) => LoginError::AuthError(e.into()),
                AuthError::UnexpectedError(_) => LoginError::UnexpectedError(e.into()),
            };

            let header_value = HeaderValue::from_str(&format!("/login")).unwrap();
            let mut headers = HeaderMap::new();
            headers.insert(header::LOCATION, header_value);
            let cookies = CookieJar::new()
                .add(
                    Cookie::build("_flash", e.to_string())
                        .max_age(time::Duration::seconds(1))
                        .finish()
                );
            Err((StatusCode::SEE_OTHER, headers, cookies).into_response())
        },
    }
}