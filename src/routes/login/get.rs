use axum::{
    http::{
        header::{self, HeaderValue, HeaderMap},
        StatusCode
    },
    response::{IntoResponse, Response},
};
use axum_flash::{IncomingFlashes, Level};

use std::fmt::Write;

pub async fn login_form(
    flashes: IncomingFlashes,
) -> Response {
    let mut error_html = String::new();
    for (_, msg) in flashes.iter().filter(|(l, _)| *l == Level::Error) {
        writeln!(
            error_html,
            "<p><i>{}</i></p>",
            msg
        ).unwrap();
    }

    let mut headers = HeaderMap::new();
    headers.insert(
        header::CONTENT_TYPE,
        HeaderValue::from_str("text/html; charset=utf-8").unwrap(),
    );

    (
        StatusCode::OK,
        headers,
        flashes, // Flashes must be in returned data in order to be removed from client
        format!(
            r#"<!DOCTYPE html>
            <html lang="en">
            <head>
                <meta http-equiv="content-type" content="text/html; charset=utf-8">
                <title>Login</title>
            </head>
            <body>
                {error_html}
                <form action="/login" method="post">
                    <label>Username
                        <input
                            type="text"
                            placeholder="Enter Username"
                            name="username"
                        >
                    </label>
                    <label>Password
                        <input
                            type="password"
                            placeholder="Enter Password"
                            name="password"
                        >
                    </label>
                    <button type="submit">Login</button>
                </form>
            </body>
            </html>"#,
        )
    )
    .into_response()
}