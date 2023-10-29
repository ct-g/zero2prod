use crate::{authentication::UserId, error::ResponseError};
use crate::domain::SubscriberEmail;
use crate::email_client::EmailClient;
use crate::error::error_chain_fmt;
use crate::idempotency::{IdempotencyKey, save_response, try_processing, NextAction};

use anyhow::Context;
use axum::{
    Extension,
    Form,
    http::{header::HeaderValue, StatusCode},
    response::{IntoResponse, Response}
};
use axum_flash::Flash;
use hyper::header;
use sqlx::PgPool;

use std::sync::Arc;

struct ConfirmedSubscriber {
    email: SubscriberEmail,
}

#[derive(serde::Deserialize)]
pub struct NewsletterFormData {
    title: String,
    text: String,
    html: String,
    idempotency_key: String,
}

#[derive(thiserror::Error)]
pub enum PublishError {
    #[error("Authentication failed")]
    AuthError(#[source] anyhow::Error),
    #[error(transparent)]
    UnexpectedError(#[from] anyhow::Error)
}

impl std::fmt::Debug for PublishError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        error_chain_fmt(self, f)
    }
}

impl IntoResponse for PublishError {
    fn into_response(self) -> Response {
        match self {
            PublishError::UnexpectedError(_) => StatusCode::INTERNAL_SERVER_ERROR.into_response(),
            PublishError::AuthError(_) => {
                let header_value = HeaderValue::from_str(r#"Basic realm="publish""#).unwrap();
                let mut response = StatusCode::UNAUTHORIZED.into_response();
                response
                    .headers_mut()
                    .insert(header::WWW_AUTHENTICATE, header_value);
                response
            }
        }
    }
}

#[tracing::instrument(
    name = "Publish a newsletter issue",
    skip(form, pool, email_client),
    fields(username=tracing::field::Empty, user_id=tracing::field::Empty)
)]
pub async fn publish_newsletter<T>(
    flash: Flash,
    Extension(user_id): Extension<UserId>,
    Extension(pool): Extension<Arc<PgPool>>,
    Extension(email_client): Extension<Arc<EmailClient>>,
    Form(form): Form<NewsletterFormData>,
) -> Result<impl IntoResponse, PublishError>
where
    T: axum_session::DatabasePool + Clone + std::fmt::Debug + Sync + Send + 'static
{
    let NewsletterFormData { title, text, html, idempotency_key } = form;
    let idempotency_key: IdempotencyKey =
        match idempotency_key.try_into(){
            Ok(key) => key,
            Err(e) => {
                let internal_error: Box<dyn std::error::Error> = e.into();
                return Ok(ResponseError::new(StatusCode::BAD_REQUEST, internal_error).into_response())
            }
        };
    // Return early if a saved response is found in the database
    let transaction = match try_processing(&pool, &idempotency_key, *user_id)
        .await
        .map_err(PublishError::UnexpectedError)?
    {
        NextAction::StartProcessing(t) => t,
        NextAction::ReturnSavedResponse(saved_response) => {
            let flash = flash.info("The newsletter has been published.");
            return Ok((flash, saved_response).into_response());
        }
    };

    let subscribers = get_confirmed_subscribers(&pool).await?;
    for subscriber in subscribers {
        match subscriber {
            Ok(subscriber) => {
                email_client
                .send_email(
                    &subscriber.email,
                    &title,
                    &html,
                    &text,
                )
                .await
                .with_context(|| {
                    format!("Failed to send newsletter issue to {}", subscriber.email)
                })?;
            }
            Err(error) => {
                tracing::warn!(
                    error.cause_chain = ?error,
                    "Skipping a confirmed subscriber. \
                    Their stored contact details are invalid",
                );
            }
        }

    }
    let flash = flash.info("The newsletter has been published.");
    let redirect = axum::response::Redirect::to("/admin/newsletters");
    let response = (flash, redirect).into_response();   
    let response = save_response(transaction, &idempotency_key, *user_id, response)
        .await
        .map_err(PublishError::UnexpectedError)?;
    Ok(response)
}

#[tracing::instrument(name = "Get confirmed subscribers", skip(pool))]
async fn get_confirmed_subscribers(
    pool: &PgPool,
) -> Result<Vec<Result<ConfirmedSubscriber, anyhow::Error>>, anyhow::Error> {
    let confirmed_subscribers = sqlx::query!(
        r#"
        SELECT email
        FROM subscriptions
        WHERE status = 'confirmed'
        "#,
    )
    .fetch_all(pool)
    .await?
    .into_iter()
    .map(|r| match SubscriberEmail::parse(r.email) {
        Ok(email) => Ok(ConfirmedSubscriber { email }),
        Err(error) => {
            Err(anyhow::anyhow!(error))
        }
    })
    .collect();
    Ok(confirmed_subscribers)
}