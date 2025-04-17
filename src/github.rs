use sha1_smol::Sha1;
use std::{
    collections::{HashMap, HashSet},
    error::Error,
    hash::Hash,
};

use octocrab::Octocrab;

use crate::{
    activity::{self},
    config::GithubConfig,
};

pub struct GithubClient {
    config: GithubConfig,
    client: reqwest::Client,
    // Might come in handy
    octocrab: Octocrab,
}

impl GithubClient {
    pub fn new(github_config: GithubConfig) -> Self {
        let mut headers = reqwest::header::HeaderMap::new();
        headers.insert(
            reqwest::header::AUTHORIZATION,
            reqwest::header::HeaderValue::from_str(&format!("token {}", github_config.token))
                .unwrap(),
        );

        let client = reqwest::Client::builder()
            .default_headers(headers)
            .build()
            .unwrap();

        let octocrab = octocrab::instance()
            .user_access_token(&*github_config.token)
            .unwrap();

        let hasher = Sha1::new();

        Self {
            config: github_config,
            client,
            octocrab,
        }
    }

    pub async fn sync(
        &self,
        repos: HashMap<activity::Repository, HashSet<activity::Activity>>,
    ) -> anyhow::Result<()> {
        // Get all unique repos
        println!("\nSyncing...");

        'repoloop: for (repo, activities) in repos {
            let name = if repo.owned_by_you {
                repo.name
            } else {
                format!("{}-{}", repo.owner, repo.name)
            };

            println!("Testing {}/{}", self.config.username, name);
            let mut repo = match self
                .octocrab
                .repos(&self.config.username, &name)
                .get()
                .await
            {
                Ok(r) => Some(r),
                Err(octocrab::Error::GitHub { source, .. }) if source.status_code == 404 => {
                    let name = Sha1::from(name).digest().to_string();
                    println!("Testing {}/{}", self.config.username, name);
                    match self
                        .octocrab
                        .repos(&self.config.username, &name)
                        .get()
                        .await
                    {
                        Ok(r) => Some(r),
                        Err(octocrab::Error::GitHub { source, .. })
                            if source.status_code == 404 =>
                        {
                            None
                        }
                        Err(e) => {
                            eprintln!("Error checking repo {}/{}: {}", repo.owner, &name, e);
                            continue 'repoloop;
                        }
                    }
                }
                Err(e) => {
                    eprintln!("Error checking repo {}/{}: {}", repo.owner, name, e);
                    continue 'repoloop;
                }
            };

            if let Some(repo) = repo {
                if !self.is_mirror(&repo).await {
                    continue 'repoloop;
                }
                println!(
                    "Repo is {}",
                    repo.private
                        .and_then(|p| Some(if p { "private" } else { "not private" }))
                        .unwrap_or("Undefined...")
                );
            } else {
                println!("NOT Found!!!!");
            }
        }

        let octocrab = octocrab::instance().user_access_token(self.config.token.clone())?;
        // Returns the first page of all issues.
        let mut page = octocrab
            .repos("Aadniz", "nix-config")
            .list_commits()
            // Optional Parameters
            .per_page(50)
            .send()
            .await?;

        // Go through every page of issues. Warning: There's no rate limiting so
        // be careful.
        for commit in &page {
            println!("{}", commit.commit.message);
        }

        Ok(())
    }

    async fn is_mirror(&self, repo: &octocrab::models::Repository) -> bool {
        if let Some(owner) = repo.owner.as_ref().and_then(|o| Some(o.login.clone())) {
            let something = self
                .octocrab
                .repos(owner, repo.name.clone())
                .get_readme()
                .send()
                .await
                .unwrap()
                .decoded_content()
                .unwrap();
            println!("{}", something);
        }
        true
    }
}
