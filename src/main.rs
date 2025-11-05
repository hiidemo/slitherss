mod config;
mod game;
mod packet;
mod server;

use clap::Parser;
use config::Config;
use server::GameServer;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    env_logger::init();

    let config = Config::parse();

    println!("╔═══════════════════════════════════════╗");
    println!("║   Slitherss - Rust Implementation    ║");
    println!("╚═══════════════════════════════════════╝");
    println!();
    println!("Server configuration:");
    println!("  Port: {}", config.port);
    println!("  Bots: {}", config.bots);
    println!("  Average length: {}", config.avg_len);
    println!("  Min length: {}", config.min_len);
    println!();

    let server = GameServer::new(config);
    server.run().await?;

    Ok(())
}
