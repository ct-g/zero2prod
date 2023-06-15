use zero2prod::startup::run;
use zero2prod::configuration::get_configuration;
use zero2prod::telemetry::{get_subscriber, init_subscriber};

use std::net::TcpListener;

use secrecy::ExposeSecret;

use sqlx::postgres::PgPoolOptions;

#[tokio::main]
async fn main() -> Result<(), hyper::Error> {
    // Setup tracing
    let subscriber = get_subscriber("zero2prod".into(), "info".into(), std::io::stdout);
    init_subscriber(subscriber);

    // Setup server
    let configuration = get_configuration().expect("Failed to read configuration");

    let connection_pool = PgPoolOptions::new()
        .acquire_timeout(std::time::Duration::from_secs(2))
        .connect_lazy(
        &configuration.database.connection_string().expose_secret()
    )
    .expect("Failed to connect to Postgres");

    let address = format!("{}:{}", configuration.application.host, configuration.application.port);
    let listener = TcpListener::bind(address).expect("Unable to bind to address");
    run(listener, connection_pool)?.await
}