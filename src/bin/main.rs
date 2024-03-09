use dotenvy::dotenv;
use loco_rs::cli;
use migration::Migrator;
use threads_crush::app::App;
use tracing::warn;

#[tokio::main]
async fn main() -> color_eyre::Result<()> {
    color_eyre::install()?;
    dotenv().map_err(|e| warn!("Error loading .env file: {}", e)).ok();
    cli::main::<App, Migrator>().await
}
