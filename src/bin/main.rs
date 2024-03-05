use dotenv::dotenv;
use loco_rs::cli;
use migration::Migrator;
use threads_crush::app::App;

#[tokio::main]
async fn main() -> color_eyre::Result<()> {
    color_eyre::install()?;
    dotenv().ok();
    cli::main::<App, Migrator>().await
}
