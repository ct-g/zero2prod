use axum::{
    http::StatusCode,
    response::IntoResponse,
};

pub async fn admin_dashboard() -> impl IntoResponse {
    StatusCode::OK.into_response()
}