#[cfg(feature = "server")]
#[actix_rt::main]
async fn main() -> std::io::Result<()> {
    let port: u16 = std::env::var("PORT")
        .ok()
        .and_then(|s| s.parse().ok())
        .unwrap_or(8080);
    salience_engine::start_server(port).await
}

#[cfg(not(feature = "server"))]
fn main() {
    eprintln!("salience-engine binary built without 'server' feature. Enable with --features server.");
    std::process::exit(1);
}
