use axum::{
    Form,
    http::StatusCode, // Re-export of http crate
    Router,
    routing::{get, post, IntoMakeService},
    Server // Re-export of Server from hyper crate
};

use serde::Deserialize;

use std::net::TcpListener;

pub fn run(listener: TcpListener) -> Result<Server<hyper::server::conn::AddrIncoming, IntoMakeService<Router>>, hyper::Error> {
    let app = Router::new()
        .route("/health_check", get(health_check))
        .route("/subscriptions", post(subscribe));

    let server = Server::from_tcp(listener)?
        .serve(app.into_make_service());

    Ok(server)
}

#[derive(Deserialize)]
struct FormData {
    email: String,
    name: String,
}

async fn health_check() -> StatusCode {
    StatusCode::OK
}

async fn subscribe(_form: Form<FormData>) -> StatusCode {
    StatusCode::OK
}