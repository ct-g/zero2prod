use axum::{
    http::{header::{self, HeaderValue}, StatusCode},
    response::{IntoResponse, Response},
};

pub async fn home() -> Response {
    let mut response = (StatusCode::OK, include_str!("home.html"))
        .into_response();
    let header_value = HeaderValue::from_str("text/html; charset=utf-8").unwrap();
    response.headers_mut()
        .insert(header::CONTENT_TYPE, header_value);
    response
}