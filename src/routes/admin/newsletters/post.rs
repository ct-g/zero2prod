use crate::{authentication::UserId, error::ResponseError};
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
use sqlx::{PgPool, Postgres, Transaction};
use uuid::Uuid;

use std::sync::Arc;

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
    skip_all,
    fields(user_id=%&*user_id)
)]
pub async fn publish_newsletter<T>(
    flash: Flash,
    Extension(user_id): Extension<UserId>,
    Extension(pool): Extension<Arc<PgPool>>,
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
    let mut transaction = match try_processing(&pool, &idempotency_key, *user_id)
        .await
        .map_err(PublishError::UnexpectedError)?
    {
        NextAction::StartProcessing(t) => t,
        NextAction::ReturnSavedResponse(saved_response) => {
            let flash = flash.info("The newsletter issue has been accepted - emails will go out shortly.");
            return Ok((flash, saved_response).into_response());
        }
    };
    let issue_id = insert_newsletter_issue(
        &mut transaction,
        &title,
        &text, 
        &html,
    )
    .await
    .context("Failed to store newsletter issue details")
    .map_err(PublishError::UnexpectedError)?;

    enqueue_delivery_tasks(&mut transaction, issue_id)
        .await
        .context("Failed to enqueue delivery tasks")
        .map_err(PublishError::UnexpectedError)?;

    let flash = flash.info("The newsletter issue has been accepted - emails will go out shortly.");
    let redirect = axum::response::Redirect::to("/admin/newsletters");
    let response = (flash, redirect).into_response();   
    let response = save_response(transaction, &idempotency_key, *user_id, response)
        .await
        .map_err(PublishError::UnexpectedError)?;
    Ok(response)
}

#[tracing::instrument(skip_all)]
async fn insert_newsletter_issue(
    transaction: &mut Transaction<'_, Postgres>,
    title: &str,
    text_content: &str,
    html_content: &str,
) -> Result<Uuid, sqlx::Error> {
    let newsletter_issue_id = Uuid::new_v4();
    sqlx::query!(
        r#"
        INSERT INTO newsletter_issues (
        newsletter_issue_id,
        title,
        text_content,
        html_content,
        published_at
        )
        VALUES ($1, $2, $3, $4, now())
        "#,
        newsletter_issue_id,
        title,
        text_content,
        html_content
    )
    .execute(transaction)
    .await?;
    Ok(newsletter_issue_id)
}

#[tracing::instrument(skip_all)]
async fn enqueue_delivery_tasks(
    transaction: &mut Transaction<'_, Postgres>,
    newsletter_issue_id: Uuid,
) -> Result<(), sqlx::Error> {
    sqlx::query!(
        r#"
        INSERT INTO issue_delivery_queue (
        newsletter_issue_id,
        subscriber_email
        )
        SELECT $1, email
        FROM subscriptions
        WHERE status = 'confirmed'
        "#,
        newsletter_issue_id,
    )
    .execute(transaction)
    .await?;
    Ok(())
}