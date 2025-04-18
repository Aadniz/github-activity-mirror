use anyhow;
use clap::Parser;
use std::{
    collections::{HashMap, HashSet},
    path::PathBuf,
};
use tokio;

use config::Config;

mod activity;
mod config;
mod git;
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

    let mut repos: HashMap<activity::Repository, HashSet<activity::Activity>> = HashMap::new();

    for service in config.services {
        if let Some(client) = &service.client {
            let result = client.get_repos().await?;
            repos.extend(result);
        }
    }

    let github_client = github::GithubClient::new(config.github).await;
    github_client.sync(repos).await?;

    Ok(())
}
