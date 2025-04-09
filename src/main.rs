use anyhow;
use clap::Parser;
use std::path::PathBuf;
use tokio;

use config::Config;

mod config;
mod github;
mod services;

/// Application to mirror GitHub activity from other git platforms
#[derive(Parser)]
#[command(version, about, long_about = None)]
struct Cli {
    /// Path to settings.toml config file
    #[clap(name = "PATH", default_value = "./settings.toml")]
    path: PathBuf,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();
    let config = Config::load(cli.path)?;

    let mut activities = vec![];

    for service in config.services {
        if let Some(client) = &service.client {
            let result = client.get_activities().await?;
            activities.extend(result);
        }
    }

    config.github.sync(activities)?;

    println!("{:?}", config.github);

    Ok(())
}
