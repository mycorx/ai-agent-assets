mod config;
mod server;

use rmcp::ServiceExt;
use rmcp::transport::stdio;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // stderr, never stdout: stdout is the MCP transport, and one stray
    // line on it corrupts the JSON-RPC stream.
    tracing_subscriber::fmt()
        .with_writer(std::io::stderr)
        .init();

    let service = server::WeatherServer::new(config::Config::from_env())
        .serve(stdio())
        .await?;
    service.waiting().await?;
    Ok(())
}
