use zero2prod::run;

use std::net::TcpListener;

#[tokio::main]
async fn main() -> Result<(), hyper::Error> {
    let listener = TcpListener::bind("127.0.0.1:0").unwrap();

    println!("http://127.0.0.1:{}", listener.local_addr().unwrap().port());

    run(listener)?.await
}