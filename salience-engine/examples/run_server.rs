//! Example of running the salience engine server

use salience_engine::start_server;

#[tokio::main]
async fn main() -> std::io::Result<()> {
    // Start the server on port 8080
    start_server(8080).await?;
    
    Ok(())
}
