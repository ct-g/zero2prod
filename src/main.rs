use zero2prod::startup::run;
use zero2prod::configuration::get_configuration;
use zero2prod::telemetry::{get_subscriber, init_subscriber};

use std::net::TcpListener;

use secrecy::ExposeSecret;
use sqlx::PgPool;

#[tokio::main]
async fn main() -> Result<(), hyper::Error> {
    // Setup tracing
    let subscriber = get_subscriber("zero2prod".into(), "info".into(), std::io::stdout);
    init_subscriber(subscriber);

    // Setup server
    let configuration = get_configuration().expect("Failed to read configuration");

    let connection_pool = PgPool::connect(
        &configuration.database.connection_string().expose_secret()
    )
    .await
    .expect("Failed to connect to Postgres");

    let address = format!("127.0.0.1:{}", configuration.application_port);
    let listener = TcpListener::bind(address).expect("Unable to bind to address");
    run(listener, connection_pool)?.await
}