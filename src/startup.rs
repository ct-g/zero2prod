use crate::configuration::{DatabaseSettings, Settings};
use crate::email_client::EmailClient;
use crate::routes::{health_check, home, subscribe, confirm, publish_newsletter};

use sqlx::PgPool;
use sqlx::postgres::PgPoolOptions;

use axum::{
    Extension,
    Router,
    routing::{get, post, IntoMakeService},
    Server, // Re-export of Server from hyper crate
};
use tower_http::trace::{TraceLayer, self};
use tower::ServiceBuilder;
use tracing::Level;

use std::{net::TcpListener, sync::Arc};

pub struct Application {
    port: u16,
    server: Server<hyper::server::conn::AddrIncoming, IntoMakeService<Router>>,
}

#[derive(Clone  )]
pub struct ApplicationBaseUrl(pub String);

impl Application {
    pub async fn build(
        configuration: Settings
    ) -> Result<Self, hyper::Error> {
        let connection_pool = get_connection_pool(&configuration.database); 
    
        let sender_email = configuration
            .email_client
            .sender()
            .expect("Invalid sender email address");
        let timeout = configuration.email_client.timeout();
        let email_client = EmailClient::new(
            configuration.email_client.base_url,
            sender_email,
            configuration.email_client.authorisation_token,
            timeout,
        );
    
        let address = format!("{}:{}", configuration.application.host, configuration.application.port);
        let listener = TcpListener::bind(address).expect("Unable to bind to address");
        let port = listener.local_addr().unwrap().port();
        let server = run(
            listener,
            connection_pool,
            email_client,
            configuration.application.base_url,
        )?;

        Ok(Self { port, server })
    }

    pub fn port(&self) -> u16 {
        self.port
    }

    pub async fn run_until_stopped(self) -> Result<(), hyper::Error> {
        self.server.await
    }
}

pub fn get_connection_pool(configuration: &DatabaseSettings) -> PgPool {
    PgPoolOptions::new()
    .acquire_timeout(std::time::Duration::from_secs(2))
    .connect_lazy_with(
        configuration.with_db()
    )
}

pub fn run(
    listener: TcpListener,
    db_pool: PgPool,
    email_client: EmailClient,
    base_url: String,
) -> Result<Server<hyper::server::conn::AddrIncoming, IntoMakeService<Router>>, hyper::Error> {
    // State must be cloneable for the into_make_service call, hence Arc
    let db_pool = Arc::new(db_pool);
    let email_client = Arc::new(email_client);
    let base_url = ApplicationBaseUrl(base_url);

    let app = Router::new()
        .route("/", get(home))
        .route("/health_check", get(health_check))
        .route("/subscriptions", post(subscribe))
        .route("/subscriptions/confirm", get(confirm))
        .route("/newsletters", post(publish_newsletter))
        .layer(
            ServiceBuilder::new()
                .layer(
                    TraceLayer::new_for_http()
                        .make_span_with(
                            trace::DefaultMakeSpan::new()
                                .level(Level::INFO)
                            )
                        .on_response(
                            trace::DefaultOnResponse::new()
                                .level(Level::INFO)
                        )
                )
        )
        .layer(Extension(db_pool))
        .layer(Extension(email_client))
        .layer(Extension(base_url));
        

    let server = Server::from_tcp(listener)?
        .serve(app.into_make_service());

    Ok(server)
}