use axum::{
    http::StatusCode, // Re-export of http crate
    response::IntoResponse,
    Router,
    routing::{get, IntoMakeService},
    Server // Re-export of Server from hyper crate
};

use std::net::TcpListener;

pub fn run(listener: TcpListener) -> Result<Server<hyper::server::conn::AddrIncoming, IntoMakeService<Router>>, hyper::Error> {
    let app = Router::new()
        .route("/health_check", get(health_check));

    let server = Server::from_tcp(listener)?
        .serve(app.into_make_service());

    Ok(server)
}

async fn health_check() -> impl IntoResponse{
    StatusCode::OK
}