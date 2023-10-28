use crate::{
    authentication::UserId,
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
    Extension(user_id): Extension<UserId>,
    flash: Flash,
    Extension(pool): Extension<Arc<PgPool>>,
    form: Form<FormData>
) -> Result<impl IntoResponse, ResponseError>
where
    T: axum_session::DatabasePool + Clone + std::fmt::Debug + Sync + Send + 'static
{
    let user_id = user_id;

    if form.new_password.expose_secret() != form.new_password_check.expose_secret() {
        let flash = flash.error(
            "You entered two different new passwords - the field values must match."
        );
        let response = axum::response::Redirect::to("/admin/password");
        return Ok((flash, response).into_response());
    }

    if form.new_password.expose_secret().len() < 12 {
        let flash = flash.error("Password needs to be at least 12 characters.");
        return Ok((flash, axum::response::Redirect::to("/admin/password")).into_response());
    }
    if form.new_password.expose_secret().len() >= 128 {
        let flash = flash.error("Password must be less than 128 characters.");
        return Ok((flash, axum::response::Redirect::to("/admin/password")).into_response());
    }

    let username = match get_username(*user_id, &pool).await {
        Ok(username) => username,
        Err(e) => return Err(ResponseError::from(e)),
    };

    let credentials = Credentials {
        username,
        password: form.0.current_password,
    };
    if let Err(e) = validate_credentials(credentials, &pool).await {
        return match e {
            AuthError::InvalidCredentials(_) => {
                let flash = flash.error("The current password is incorrect.");
                Ok((flash, axum::response::Redirect::to("/admin/password")).into_response())
            },
            AuthError::UnexpectedError(_) => Err(ResponseError::from(e)),
        }
    }

    match crate::authentication::change_password(*user_id, form.0.new_password, &pool).await {
        Ok(_) => (),
        Err(e) => return Err(ResponseError::from(e)),
    }
    let flash = flash.error("Your password has been changed.");
    Ok((flash, axum::response::Redirect::to("/admin/password")).into_response())
}