use axum::{
    http::{
        header::{self, HeaderValue, HeaderMap},
        StatusCode
    },
    response::IntoResponse,
};
use axum_flash::{IncomingFlashes, Level};

use std::fmt::Write;


pub async fn change_password_form<T>(
    flash_messages: IncomingFlashes,
) -> impl IntoResponse
where
    T: axum_session::DatabasePool + Clone + std::fmt::Debug + Sync + Send + 'static
{
    let mut msg_html = String::new();
    for (_, msg) in flash_messages.iter().filter(|(l, _)| *l == Level::Error) {
        writeln!(
            msg_html,
            "<p><i>{}</i></p>",
            msg
        ).unwrap();
    }

    let html = format!(
        r#"<!DOCTYPE html>
        <html lang="en">
        <head>
        <meta http-equiv="content-type" content="text/html; charset=utf-8">
        <title>Change Password</title>
        </head>
        <body>
        {msg_html}
        <form action="/admin/password" method="post">
        <label>Current password
        498
        CHAPTER 10. SECURING OUR API
        <input
        type="password"
        placeholder="Enter current password"
        name="current_password"
        >
        </label>
        <br>
        <label>New password
        <input
        type="password"
        placeholder="Enter new password"
        name="new_password"
        >
        </label>
        <br>
        <label>Confirm new password
        <input
        type="password"
        placeholder="Type the new password again"
        name="new_password_check"
        >
        </label>
        <br>
        <button type="submit">Change password</button>
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