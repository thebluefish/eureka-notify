mod eureka;
mod ocean;

use tracing::*;
use tracing_subscriber;

const APP_ID: &str = "com.squirrel.XIVLauncher.XIVLauncher";

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt().with_max_level(Level::INFO).init();

    let eureka = tokio::spawn(eureka::run_loop());
    let ocean = tokio::spawn(ocean::run_loop());
    let (eureka, ocean) = futures::join!(eureka, ocean);
    eureka?;
    ocean?;
    Ok(())
}