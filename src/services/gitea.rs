use std::io;
use std::io::prelude::*;

use crate::activity;
use crate::config::ServiceConfig;
use async_trait::async_trait;
use chrono::{DateTime, FixedOffset};
use reqwest;
use serde;
use serde::{Deserialize, Deserializer, Serialize};
use url::Url;

use super::ServiceClient;

#[derive(Debug, Deserialize, Serialize)]
pub struct GiteaUser {
    pub id: u64,
    pub login: String,
    pub login_name: String,
    pub source_id: u64,
    pub full_name: String,
    pub email: String,
    pub html_url: Url,
    pub language: String,
    pub is_admin: bool,
    pub last_login: DateTime<FixedOffset>,
    pub created: DateTime<FixedOffset>,
    pub restricted: bool,
    pub active: bool,
    pub prohibit_login: bool,
    pub location: String,
    pub website: String,
    pub description: String,
    pub visibility: String,
    pub followers_count: u64,
    pub following_count: u64,
    pub starred_repos_count: u64,
    pub username: String,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct GiteaRepo {
    pub id: u64,
    pub owner: GiteaUser,
    pub name: String,
    pub full_name: String,
    pub description: String,
    pub empty: bool,
    pub private: bool,
    pub fork: bool,
    pub template: bool,
    // pub parent: null
    pub mirror: bool,
    pub size: u64,
    pub language: String,
    pub languages_url: Url,
    pub html_url: Url,
    pub url: Url,
    pub ssh_url: String,
    pub clone_url: Url,
    pub default_branch: String,
    pub archived: bool,
    pub created_at: DateTime<FixedOffset>,
    pub updated_at: DateTime<FixedOffset>,
    pub permissions: serde_json::Value,
    pub stars_count: u64,
    pub forks_count: u64,
    pub watchers_count: u64,
    pub open_issues_count: u64,
    pub open_pr_counter: u64,
    pub release_counter: u64,
    pub archived_at: DateTime<FixedOffset>,
    pub has_issues: bool,
    pub internal_tracker: serde_json::Value,
    pub has_wiki: bool,
    pub has_pull_requests: bool,
    pub has_projects: bool,
    pub projects_mode: String,
    pub has_releases: bool,
    pub has_packages: bool,
    pub has_actions: bool,
    pub ignore_whitespace_conflicts: bool,
    pub allow_merge_commits: bool,
    pub allow_rebase: bool,
    pub allow_rebase_explicit: bool,
    pub allow_squash_merge: bool,
    pub allow_fast_forward_only_merge: bool,
    pub allow_rebase_update: bool,
    pub default_delete_branch_after_merge: bool,
    pub default_merge_style: String,
    pub default_allow_maintainer_edit: bool,
    pub internal: bool,
    // pub mirror_interval: "",
    pub object_format_name: String,
    pub mirror_updated: DateTime<FixedOffset>,
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "PascalCase")]
#[serde(untagged)]
pub enum GiteaContent {
    Commit(CommitContent),
    String(String),
    None,
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "PascalCase")]
struct CommitContent {
    commits: Vec<CommitInfoShort>,
    head_commit: CommitInfoShort,
    compare_u_r_l: String,
    len: u64,
}

fn string_json<'de, D>(deserializer: D) -> Result<GiteaContent, D::Error>
where
    D: Deserializer<'de>,
{
    let s = String::deserialize(deserializer)?;

    // If it looks like JSON, try to parse it as GiteaContent::Commit
    if s.starts_with('{') {
        match serde_json::from_str::<GiteaContent>(&s) {
            Ok(content) => Ok(content),
            Err(e) => {
                println!("{e}");
                Ok(GiteaContent::String(s))
            }
        }
    } else if s.is_empty() {
        Ok(GiteaContent::None)
    } else {
        Ok(GiteaContent::String(s))
    }
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "PascalCase")]
struct CommitInfoShort {
    sha1: String,
    message: String,
    author_email: String,
    author_name: String,
    committer_email: String,
    committer_name: String,
    timestamp: DateTime<FixedOffset>,
}

#[derive(Debug, Deserialize, Serialize)]
struct CommitInfoSummary {
    url: Url,
    sha: String,
    created: DateTime<FixedOffset>,
}

#[derive(Debug, Deserialize, Serialize)]
struct GiteaUserShort {
    name: String,
    email: String,
    date: DateTime<FixedOffset>,
}

#[derive(Debug, Deserialize, Serialize)]
struct CommitDetails {
    url: Url,
    author: GiteaUserShort,
    committer: GiteaUserShort,
    message: String,
    tree: CommitInfoSummary,
    // verification: todo!("Unimportant"),
}

#[derive(Debug, Deserialize, Serialize)]
struct CommitInfo {
    url: Url,
    sha: String,
    created: DateTime<FixedOffset>,
    html_url: Url,
    commit: CommitDetails,
    author: Option<GiteaUser>,
    committer: Option<GiteaUser>,
    parents: Vec<CommitInfoSummary>,
    // files: todo!("Unimportant"),
    // stats: todo!("Unimportant"),
}

#[derive(Debug, Deserialize, Serialize)]
struct GiteaActivity {
    id: u64,
    user_id: u64,
    op_type: activity::OpType,
    act_user_id: u64,
    act_user: GiteaUser,
    repo_id: u64,
    repo: GiteaRepo,
    comment_id: u64,
    // comment: null,  // Don't know what type this is, doesn't matter either for this application
    ref_name: String,
    is_private: bool,
    #[serde(deserialize_with = "string_json")]
    content: GiteaContent,
    created: DateTime<FixedOffset>,
}

#[derive(Debug)]
pub struct GiteaClient {
    base_url: Url,
    username: String,
    token: Option<String>,
    client: reqwest::Client,
}

impl From<GiteaActivity> for Option<activity::Activity> {
    fn from(gitea_activity: GiteaActivity) -> Option<activity::Activity> {
        let content = match gitea_activity.content {
            GiteaContent::Commit(commit_content) => {
                let commits = commit_content
                    .commits
                    .into_iter()
                    .map(|c| activity::Commit {
                        sha1: c.sha1,
                        message: c.message,
                        author_email: c.author_email,
                        author_name: c.author_name,
                        timestamp: c.timestamp,
                    })
                    .collect();

                activity::ActivityContent::Commit {
                    commits,
                    compare_url: commit_content.compare_u_r_l,
                }
            }
            // Skip non-commit activities for now
            _ => return None,
        };

        Some(activity::Activity {
            id: gitea_activity.id,
            op_type: gitea_activity.op_type,
            repo: activity::Repository {
                id: gitea_activity.repo.id,
                name: gitea_activity.repo.name,
                full_name: gitea_activity.repo.full_name,
                html_url: gitea_activity.repo.html_url,
                clone_url: gitea_activity.repo.clone_url,
                private: gitea_activity.repo.private,
            },
            created: gitea_activity.created,
            content,
        })
    }
}

impl GiteaClient {
    pub fn new(config: &ServiceConfig) -> anyhow::Result<Self> {
        let mut base_url = config.url.clone();
        base_url
            .path_segments_mut()
            .map_err(|_| anyhow::anyhow!("Invalid base URL"))?
            .extend("/api/v1".split('/'));

        let mut headers = reqwest::header::HeaderMap::new();
        headers.insert(
            reqwest::header::AUTHORIZATION,
            reqwest::header::HeaderValue::from_str(&format!("token {}", config.token))?,
        );

        let client = reqwest::Client::builder()
            .default_headers(headers)
            .build()?;

        Ok(Self {
            client,
            username: config.username.clone(),
            base_url,
            token: Some(config.token.clone()),
        })
    }
}

#[async_trait]
impl ServiceClient for GiteaClient {
    async fn get_activities(&self) -> anyhow::Result<Vec<activity::Activity>> {
        println!("Fetching activities from {}", self.base_url);
        let mut all_activities = Vec::new();
        let mut page = 1;
        let limit = 50;
        loop {
            let url = format!(
                "{}/users/{}/activities/feeds?only-performed-by=true&page={}&limit={}",
                self.base_url, self.username, page, limit
            );

            let mut result: Vec<GiteaActivity> = self
                .client
                .get(&url)
                .send()
                .await?
                .error_for_status()?
                .json()
                .await?;

            // Gitea API will return an empty array if the limit + page goes beyond the activity
            if result.is_empty() {
                break;
            }

            for activity in &mut result {
                if let GiteaContent::Commit(c) = &mut activity.content {
                    if c.len > c.commits.len() as u64 {
                        let last_commit = c.commits.last().unwrap();
                        let repo_name = activity.repo.full_name.clone();
                        let sha = last_commit.sha1.clone();
                        let count = c.len - c.commits.len() as u64;
                        let date = last_commit.timestamp;

                        match self.get_commits(&repo_name, &sha, count, date).await {
                            Ok(all_commits) => {
                                c.commits.extend(all_commits);
                            }
                            Err(e) => {
                                eprintln!("{e}");
                            }
                        }
                    }
                }
            }

            all_activities.extend(result.into_iter().filter_map(|a| a.into()));
            page += 1;
            print!(".");
            io::stdout().flush().ok().expect("Could not flush stdout");
        }

        Ok(all_activities)
    }
}

impl GiteaClient {
    async fn get_commits(
        &self,
        repo_path: &String,
        sha1: &String,
        amount: u64,
        date: DateTime<FixedOffset>,
    ) -> anyhow::Result<Vec<CommitInfoShort>> {
        let mut page = 1;
        // Super slow otherwise
        let limit = amount * 2;

        let mut results = vec![];

        loop {
            let url = format!(
                "{}/repos/{}/commits?sha={}&page={}&limit={}",
                self.base_url, repo_path, sha1, page, limit
            );

            let result: Vec<CommitInfo> = self
                .client
                .get(&url)
                .send()
                .await?
                .error_for_status()?
                .json()
                .await?;

            // Gitea API will return an empty array if the limit + page goes beyond the activity
            if result.is_empty() {
                break;
            }

            for commit in result {
                if sha1 == &commit.sha {
                    continue;
                }

                print!(".");
                io::stdout().flush().ok().expect("Could not flush stdout");

                if commit.created != date || results.len() as u64 >= amount {
                    return Ok(results);
                }

                let (author_email, author_name) = commit
                    .author
                    .and_then(|a| Some((a.email, a.full_name)))
                    .unwrap_or((commit.commit.author.email, commit.commit.author.name));
                let (committer_email, committer_name) = commit
                    .committer
                    .and_then(|a| Some((a.email, a.full_name)))
                    .unwrap_or((commit.commit.committer.email, commit.commit.committer.name));

                let commit_info = CommitInfoShort {
                    sha1: commit.sha,
                    message: commit.commit.message,
                    author_email,
                    author_name,
                    committer_email,
                    committer_name,
                    timestamp: commit.created,
                };
                results.push(commit_info);
            }

            page += 1;
        }

        Ok(results)
    }
}
