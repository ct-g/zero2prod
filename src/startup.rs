use crate::routes::{health_check, subscribe};

use sqlx::PgPool;

use axum::{
    Extension,
    Router,
    routing::{get, post, IntoMakeService},
    Server, // Re-export of Server from hyper crate
};

use std::{net::TcpListener, sync::Arc};

pub fn run(listener: TcpListener, db_pool: PgPool) -> Result<Server<hyper::server::conn::AddrIncoming, IntoMakeService<Router>>, hyper::Error> {
    // State must be cloneable for the into_make_service call
    let db_pool = Arc::new(db_pool);

    let app = Router::new()
        .route("/health_check", get(health_check))
        .route("/subscriptions", post(subscribe))
        .layer(Extension(db_pool));

    let server = Server::from_tcp(listener)?
        .serve(app.into_make_service());

    Ok(server)
}