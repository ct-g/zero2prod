use axum::{
    http::{
        header::{self, HeaderValue, HeaderMap},
        StatusCode
    },
    response::IntoResponse,
};
use axum_flash::IncomingFlashes;

use std::fmt::Write;

pub async fn publish_newsletter_form<T>(
    flash_messages: IncomingFlashes,
) -> impl IntoResponse
where
    T: axum_session::DatabasePool + Clone + std::fmt::Debug + Sync + Send + 'static
{
    let mut msg_html = String::new();
    for (_, msg) in flash_messages.iter() {
        writeln!(
            msg_html,
            "<p><i>{}</i></p>",
            msg
        ).unwrap();
    }

    let idempotency_key = uuid::Uuid::new_v4();

    let html = format!(
        r#"<!DOCTYPE html>
        <html lang="en">
        <head>
            <meta http-equiv="content-type" content="text/html; charset=utf-8">
            <title>Change Password</title>
        </head>
        <body>
        {msg_html}
        <form action="/admin/newsletters" method="post">
            <label>Title
                <input
                    type="text"
                    placeholder="Enter newsletter title"
                    name="title"
                >
            </label>
            <br>
            <label>Contents
                <input
                    type="text"
                    placeholder="Enter content in HTML form"
                    name="html"
                >
            </label>
            <br>
            <label>Contents raw text
                <input
                    type="text"
                    placeholder="Enter content in text form"
                    name="text"
                >
            </label>
            <br>
            <input hidden type="text" name="idempotency_key" value="{idempotency_key}">
            <button type="submit">Publish newsletter</button>
        </form>
        <p><a href="/admin/dashboard">&lt;- Back</a></p>
        </body>
        </html>"#
    );
    let mut headers = HeaderMap::new();
    headers.insert(
        header::CONTENT_TYPE,
        HeaderValue::from_str("text/html; charset=utf-8").unwrap(),
    );
    (StatusCode::OK, headers, html).into_response()
}