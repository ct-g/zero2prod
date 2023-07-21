use axum::{
    http::{
        header::{self, HeaderValue},
        StatusCode
    },
    response::{IntoResponse, Response},
};
use axum_extra::extract::CookieJar;

pub async fn login_form(
    jar: CookieJar
) -> Response {
    let error_html = match jar.get("_flash") {
        None => "".into(),
        Some(cookie) => {
            format!("<p><i>{}</i></p>", cookie.value())
        }
    };

    let mut response = 
        (StatusCode::OK, format!(
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
        ))
        .into_response();
    let header_value = HeaderValue::from_str("text/html; charset=utf-8").unwrap();
    response
        .headers_mut()
        .insert(header::CONTENT_TYPE, header_value);
    response
}