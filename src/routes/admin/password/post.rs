use crate::{
    session_state::TypedSession,
    error::ResponseError,
    routes::admin::dashboard::get_username, authentication::{Credentials, validate_credentials, AuthError}
};

use axum::{
    Extension,
    Form,
    response::IntoResponse,
};
use axum_flash::Flash;
use secrecy::{ExposeSecret, Secret};
use sqlx::PgPool;

use std::sync::Arc;

#[derive(serde::Deserialize)]
pub struct FormData {
    current_password: Secret<String>,
    new_password: Secret<String>,
    new_password_check: Secret<String>,
}

pub async fn change_password<T>(
    session: TypedSession<T>,
    flash: Flash,
    Extension(pool): Extension<Arc<PgPool>>,
    form: Form<FormData>
) -> impl IntoResponse
where
    T: axum_session::DatabasePool + Clone + std::fmt::Debug + Sync + Send + 'static
{
    let user_id = session.get_user_id();
    if user_id.is_none() {
        return axum::response::Redirect::to("/login").into_response();
    }
    let user_id = user_id.unwrap();

    if form.new_password.expose_secret() != form.new_password_check.expose_secret() {
        let flash = flash.error(
            "You entered two different new passwords - the field values must match."
        );
        let response = axum::response::Redirect::to("/admin/password");
        return (flash, response).into_response();
    }

    if form.new_password.expose_secret().len() < 12 {
        let flash = flash.error("Password needs to be at least 12 characters.");
        return (flash, axum::response::Redirect::to("/admin/password")).into_response();
    }
    if form.new_password.expose_secret().len() >= 128 {
        let flash = flash.error("Password must be less than 128 characters.");
        return (flash, axum::response::Redirect::to("/admin/password")).into_response();
    }

    let username = match get_username(user_id, &pool).await {
        Ok(username) => username,
        Err(e) => return ResponseError::from(e).into_response(),
    };

    let credentials = Credentials {
        username,
        password: form.0.current_password,
    };
    if let Err(e) = validate_credentials(credentials, &pool).await {
        return match e {
            AuthError::InvalidCredentials(_) => {
                let flash = flash.error("The current password is incorrect.");
                (flash, axum::response::Redirect::to("/admin/password")).into_response()
            },
            AuthError::UnexpectedError(_) => ResponseError::from(e).into_response(),
        }
    }

    match crate::authentication::change_password(user_id, form.0.new_password, &pool).await {
        Ok(_) => (),
        Err(e) => return ResponseError::from(e).into_response(),
    }
    let flash = flash.error("Your password has been changed.");
    (flash, axum::response::Redirect::to("/admin/password")).into_response()
}