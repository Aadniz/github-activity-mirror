use chrono::DateTime;
use serde_json::json;
use sha1_smol::Sha1;
use std::cmp::max;
use std::collections::{HashMap, HashSet};
use std::io::{self, Write};

use octocrab::Octocrab;

use crate::{
    activity::{self, ActivityContent},
    config::{self, GitConfig, RedactLevel},
    git::Git,
};

const MARK_STRING: &str = "<sub>This repo was mirrored using [github-activity-mirror](https://codeberg.org/Aadniz/github-activity-mirror), preserving the privacy while at the same time display your actual activity</sub>";
const _BRANCH: &str = "main";

pub struct GithubClient {
    config: GitConfig,
    // Might come in handy
    // client: reqwest::Client,
    octocrab: Octocrab,
    git: Git,
}

impl GithubClient {
    pub async fn new(mut github_config: GitConfig) -> Self {
        let mut headers = reqwest::header::HeaderMap::new();
        headers.insert(
            reqwest::header::AUTHORIZATION,
            reqwest::header::HeaderValue::from_str(&format!("token {}", github_config.token))
                .unwrap(),
        );

        let octocrab = octocrab::instance()
            .user_access_token(&*github_config.token)
            .unwrap();

        // Get the appropriate email
        if github_config.email.is_none() {
            let emails = octocrab
                .users(&github_config.username)
                .emails()
                .list()
                .await
                .unwrap()
                .items;

            // Grab the @users.noreply.github.com
            for email in emails {
                if email.email.ends_with("@users.noreply.github.com") {
                    github_config.email = Some(email.email);
                    break;
                }
            }
        }
        if github_config.email.is_none() {
            panic!("Unable to get github email. Specify this in the settings.toml file")
        }

        let git = Git::new(github_config.clone());

        Self {
            config: github_config,
            octocrab,
            git,
        }
    }

    pub async fn sync(
        &self,
        repos: HashMap<activity::Repository, HashSet<activity::Activity>>,
    ) -> anyhow::Result<()> {
        // Get all unique repos
        print!("\n");

        'repoloop: for (source_repo, activities) in repos {
            let owner = &source_repo.owner;
            let name = if source_repo.owned_by_you {
                &source_repo.name
            } else {
                &format!("{}-{}", owner, source_repo.name)
            };

            print!("\r\x1B[KChecking {}/{}", self.config.username, name);
            io::stdout().flush().unwrap();
            let repo = match self.octocrab.repos(&self.config.username, name).get().await {
                Ok(r) => Some(r),
                Err(octocrab::Error::GitHub { source, .. }) if source.status_code == 404 => {
                    let hashed_name = Sha1::from(&name).digest().to_string();
                    print!(
                        "\r\x1B[KChecking {}/{} ({}/{})",
                        self.config.username, hashed_name, self.config.username, name
                    );
                    io::stdout().flush().unwrap();
                    match self
                        .octocrab
                        .repos(&self.config.username, &hashed_name)
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
                            eprintln!(
                                "\nError checking repo {}/{} ({}/{}): {}",
                                owner, &hashed_name, owner, &hashed_name, e
                            );
                            continue 'repoloop;
                        }
                    }
                }
                Err(e) => {
                    eprintln!("\nError checking repo {}/{}: {}", owner, name, e);
                    continue 'repoloop;
                }
            };
            print!("\n");

            let repo = if let Some(repo) = repo {
                if !self.is_mirror(&repo).await? {
                    continue 'repoloop;
                }
                repo
            } else {
                let first_activity = {
                    let mut first_activity = activities.iter().last().unwrap();
                    for activity in activities.iter() {
                        if first_activity.date > activity.date {
                            first_activity = &activity;
                        }
                    }
                    first_activity
                };
                println!("Creating repo: {}", source_repo.full_name);
                self.create_repo(&source_repo, first_activity).await?
            };

            self.sync_repo(repo, activities).await?;
        }

        Ok(())
    }

    async fn sync_repo(
        &self,
        repo: octocrab::models::Repository,
        activities: HashSet<activity::Activity>,
    ) -> anyhow::Result<()> {
        let last_commit = self.git.last_commit(&repo)?;
        let last_issue = self.last_issue(&repo).await;
        let last_activity = match last_issue {
            Some(issue) => max(issue.created_at.into(), last_commit.timestamp),
            None => last_commit.timestamp,
        };

        let mut existing_issues: Option<Vec<octocrab::models::issues::Issue>> = None;
        let mut activities = activities.into_iter().collect::<Vec<activity::Activity>>();
        let mut sync_informed = false;

        // Sort date in ascending order
        activities.sort_by(|a, b| a.date.cmp(&b.date));

        for activity in activities {
            // Squaching/force push will make this unreliable
            if last_activity > activity.date {
                continue;
            }
            if !sync_informed {
                println!("Syncing: {}", repo.full_name.clone().unwrap());
                sync_informed = true;
            }
            match activity.content {
                ActivityContent::Commit(c) => {
                    let commit_message: String = match self.config.redact_level {
                        RedactLevel::PrivateReposNoCrossLinking => c.message.clone(),
                        RedactLevel::Encrypted => todo!("Not implemented yet"),
                        RedactLevel::Hashed => Sha1::from(&c.message).digest().to_string(),
                        _ => format!("{}\n\nMirrored from: {}", c.message, activity.source_link),
                    };
                    let commit_content = match self.config.redact_level {
                        RedactLevel::PrivateReposNoCrossLinking => {
                            format!("{} {}: {}", &c.sha1, c.timestamp, c.message)
                        }
                        RedactLevel::Encrypted => todo!("Not implemented yet"),
                        RedactLevel::Hashed => {
                            Sha1::from(format!("{} {}: {}", &c.sha1, c.timestamp, c.message))
                                .digest()
                                .to_string()
                        }
                        _ => format!(
                            "{} {}: {}\n\n*{}*",
                            &c.sha1, c.timestamp, c.message, activity.source_link
                        ),
                    };
                    self.git
                        .add_commit(&repo, commit_message, commit_content, activity.date)?;
                    println!(
                        "{} - {}: {}{}",
                        activity.date,
                        repo.full_name.clone().unwrap(),
                        c.message
                            .lines()
                            .find(|line| !line.trim().is_empty())
                            .unwrap_or("<Empty commit message>"),
                        if c.message.lines().count() > 1 {
                            " ..."
                        } else {
                            ""
                        }
                    )
                }
                ActivityContent::Issue(i) => {
                    let title = match self.config.redact_level {
                        RedactLevel::Encrypted => todo!("Not implemented yet"),
                        RedactLevel::Hashed => {
                            Sha1::from(format!("{}: {}", &i.issue_id, &i.message))
                                .digest()
                                .to_string()
                        }
                        _ => format!("[{}] {}", &i.issue_id, &i.message),
                    };
                    let title = if title.len() > 255 {
                        let mut truncated: String = title.chars().take(252).collect();
                        truncated.push_str("...");
                        truncated
                    } else {
                        title
                    };
                    let body = match self.config.redact_level {
                        RedactLevel::PrivateReposNoCrossLinking => {
                            format!(
                                "## Issue ID: {}\n\n{}\n\n{}",
                                &i.issue_id, &i.message, activity.date
                            )
                        }
                        RedactLevel::Encrypted => todo!("Not implemented yet"),
                        RedactLevel::Hashed => Sha1::from(format!(
                            "## Issue ID: {}\n\n{}\n\n{}",
                            &i.issue_id, &i.message, activity.date
                        ))
                        .digest()
                        .to_string(),
                        _ => format!(
                            "## Issue ID: {}\n\n{}\n\n{}\n\n*{}*",
                            &i.issue_id, &i.message, activity.date, activity.source_link
                        ),
                    };
                    let existing_issues = existing_issues
                        .get_or_insert(self.issues_since(&repo, last_activity).await);

                    if !existing_issues.iter().any(|issue| issue.title == *title) {
                        println!(
                            "{} - {}: [{}] {}",
                            activity.date,
                            repo.full_name.clone().unwrap(),
                            i.issue_id,
                            if i.message.is_empty() {
                                "<Empty commit message>"
                            } else {
                                &i.message
                            }
                        );
                        let issue = self.create_issue(&repo, title, body).await?;
                        existing_issues.push(issue);
                    }
                }
            }
        }

        let unpushed_commits = self.git.unpushed_commits(&repo)?;
        if unpushed_commits > 0 {
            self.git.push(&repo)?;
            println!(
                "Pushed {} new commits to {}",
                unpushed_commits,
                repo.html_url.clone().unwrap()
            );
        }

        Ok(())
    }

    // To verify if it is a mirror, check if the MARK_STRING
    async fn is_mirror(&self, repo: &octocrab::models::Repository) -> anyhow::Result<bool> {
        if let Some(owner) = repo.owner.as_ref().and_then(|o| Some(o.login.clone())) {
            let readme = self
                .octocrab
                .repos(owner, repo.name.clone())
                .get_readme()
                .send()
                .await?
                .decoded_content();

            if let Some(content) = readme {
                return Ok(content.trim_end().ends_with(MARK_STRING));
            }
        }
        Ok(false)
    }

    async fn create_repo(
        &self,
        source_repo: &activity::Repository,
        init_activity: &activity::Activity,
    ) -> anyhow::Result<octocrab::models::Repository> {
        let name = if source_repo.owned_by_you {
            &source_repo.name
        } else {
            &format!("{}-{}", source_repo.owner, source_repo.name)
        };
        let name = match self.config.redact_level {
            config::RedactLevel::Encrypted => todo!("Encrypting description not implemented yet"),
            config::RedactLevel::Hashed => Sha1::from(&name).digest().to_string(),
            _ => name.clone(),
        };
        let desc = match self.config.redact_level {
            config::RedactLevel::Encrypted => todo!("Encrypting description not implemented yet"),
            config::RedactLevel::Hashed => match &source_repo.description {
                Some(value) => Some(Sha1::from(&value).digest().to_string()),
                None => None,
            },
            _ => source_repo.description.clone(),
        };

        let mut req = json!({
            "name": name,
            "private": self.config.redact_level != config::RedactLevel::Off,
        });
        if let Some(desc) = desc {
            req["description"] = serde_json::Value::String(desc);
        };

        let body = serde_json::to_value(&req).unwrap();

        let new_repo = self.octocrab.post("/user/repos", Some(&body)).await?;
        self.git.create_init(&new_repo, init_activity)?;

        println!("Created repo: {}", new_repo.html_url.clone().unwrap());

        Ok(new_repo)
    }

    async fn last_issue(
        &self,
        repo: &octocrab::models::Repository,
    ) -> Option<octocrab::models::issues::Issue> {
        // Pull requests are also issues, so we need to scroll and find the first issue
        let mut page: u32 = 1;
        let per_page = 50;
        loop {
            let results = self
                .octocrab
                .issues_by_id(repo.id)
                .list()
                .page(page)
                .per_page(per_page)
                .sort(octocrab::params::issues::Sort::Created)
                .direction(octocrab::params::Direction::Descending)
                .send()
                .await
                .unwrap();

            if results.total_count.is_none_or(|r| r == 0) {
                return None;
            }

            for res in results {
                if res.pull_request.is_none() {
                    return Some(res);
                }
            }

            page += 1;
        }
    }

    async fn create_issue(
        &self,
        repo: &octocrab::models::Repository,
        title: String,
        body: String,
    ) -> anyhow::Result<octocrab::models::issues::Issue> {
        Ok(self
            .octocrab
            .issues_by_id(repo.id)
            .create(title)
            .body(body)
            .send()
            .await?)
    }

    async fn issues_since(
        &self,
        repo: &octocrab::models::Repository,
        since: DateTime<chrono::FixedOffset>,
    ) -> Vec<octocrab::models::issues::Issue> {
        let mut page: u32 = 0;
        let per_page = 50;

        let mut issues = vec![];

        println!("Getting issues from repo: {}", repo.name);

        loop {
            let results = self
                .octocrab
                .issues_by_id(repo.id)
                .list()
                .since(since)
                .page(page)
                .per_page(per_page)
                .sort(octocrab::params::issues::Sort::Created)
                .direction(octocrab::params::Direction::Descending)
                .send()
                .await
                .unwrap();

            let res: Vec<octocrab::models::issues::Issue> = results.items.into_iter().collect();

            if res.is_empty() {
                break;
            }

            issues.extend(res.into_iter().filter(|res| res.pull_request.is_none()));

            page += 1;
        }

        issues
    }
}
