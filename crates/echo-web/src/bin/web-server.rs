//! Simple web server binary for testing Echo web functionality

use anyhow::Result;
use echo_core::{EchoConfig, EchoRuntime};
use echo_web::{WebServer, WebServerConfig};

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize tracing
    echo_core::init_tracing();

    println!("Echo Web Server v{}", echo_web::VERSION);

    // Create Echo runtime with temporary database
    let config = EchoConfig {
        storage_path: "./echo-web-db".into(),
        debug: false,
        ..Default::default()
    };

    let runtime = EchoRuntime::new(config)?;

    // Create web server config
    let web_config = WebServerConfig {
        host: "127.0.0.1".to_string(),
        port: 8081,
        static_dir: "./static".into(),
        enable_cors: true,
    };

    // Create and start web server
    let web_server = WebServer::new(web_config, runtime);
    web_server.start().await?;

    Ok(())
}
