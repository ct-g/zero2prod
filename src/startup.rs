use crate::authentication::reject_anonymous_users;
use crate::configuration::{DatabaseSettings, Settings};
use crate::email_client::EmailClient;
use crate::routes::{
    health_check,
    home,
    subscribe, confirm,
    publish_newsletter,
    login_form, login, admin_dashboard, change_password_form, change_password, logout
};

use axum::middleware;
use sqlx::PgPool;
use sqlx::postgres::PgPoolOptions;

use axum::{
    Extension,
    extract::FromRef,
    Router,
    routing::{get, post, IntoMakeService},
    Server, // Re-export of Server from hyper crate
};
use axum_flash::Key;
use axum_session::{SessionStore, SessionConfig, SessionLayer, SessionRedisPool};
use secrecy::{Secret, ExposeSecret};
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
    ) -> Result<Self, anyhow::Error> {
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
            configuration.application.hmac_secret,
            configuration.redis_uri,
        ).await?;

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

#[derive(Clone)]
pub struct AppState {
    flash_config: axum_flash::Config,
}

impl FromRef<AppState> for axum_flash::Config {
    fn from_ref(app_state: &AppState) -> Self {
        app_state.flash_config.clone()
    }
}

pub async fn run(
    listener: TcpListener,
    db_pool: PgPool,
    email_client: EmailClient,
    base_url: String,
    hmac_secret: Secret<String>,
    redis_uri: Secret<String>,
) -> Result<Server<hyper::server::conn::AddrIncoming, IntoMakeService<Router>>, anyhow::Error> {
    // State must be cloneable for the into_make_service call, hence Arc
    let db_pool = Arc::new(db_pool);
    let email_client = Arc::new(email_client);
    let base_url = ApplicationBaseUrl(base_url);

    let secret_key = Key::from(hmac_secret.expose_secret().as_bytes());

    let redis = redis::Client::open(redis_uri.expose_secret().as_str())?;
    let redis_store = SessionStore::<SessionRedisPool>::new(Some(redis.into()), SessionConfig::new()).await?;
    let app_state = AppState {
        flash_config:
            axum_flash::Config::new(
                secret_key.clone()
            ),
    };
    let admin_routes = Router::new()
        .route("/admin/dashboard", get(admin_dashboard))
        .route("/admin/password", get(change_password_form::<SessionRedisPool>))
        .route("/admin/password", post(change_password::<SessionRedisPool>))
        .route("/admin/logout", post(logout::<SessionRedisPool>))
        .layer(middleware::from_fn_with_state(app_state.clone(), reject_anonymous_users));

    let app = Router::new()
        .route("/", get(home))
        .route("/health_check", get(health_check))
        .route("/login", get(login_form))
        .route("/login", post(login))
        .route("/subscriptions", post(subscribe))
        .route("/subscriptions/confirm", get(confirm))
        .route("/newsletters", post(publish_newsletter))
        .merge(admin_routes)
        .layer(SessionLayer::new(redis_store))
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
        .layer(Extension(base_url))
        .with_state(app_state);

    let server = Server::from_tcp(listener)?
        .serve(app.into_make_service());

    Ok(server)
}