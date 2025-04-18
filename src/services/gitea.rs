use async_trait::async_trait;
use chrono::{DateTime, FixedOffset};
use reqwest;
use serde;
use serde::{Deserialize, Deserializer, Serialize};
use std::collections::{HashMap, HashSet};
use std::hash::{Hash, Hasher};
use std::io::{self, Write};
use url::Url;

use crate::activity;
use crate::config::ServiceConfig;

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
enum GiteaContent {
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

#[derive(Debug, Deserialize, Serialize, Clone)]
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

impl From<CommitInfo> for activity::Activity {
    fn from(commit: CommitInfo) -> activity::Activity {
        let (author_email, author_name) = commit
            .author
            .as_ref()
            .and_then(|a| Some((a.email.clone(), a.username.clone())))
            .unwrap_or((
                commit.commit.author.email.clone(),
                commit.commit.author.name.clone(),
            ));

        let content = activity::ActivityContent::Commit(activity::Commit {
            sha1: commit.sha.clone(),
            message: commit.commit.message.clone(),
            author_email: author_email.to_string(),
            author_name: author_name.to_string(),
            timestamp: commit.created,
        });

        activity::Activity {
            op_type: activity::OpType::CommitRepo,
            date: commit.created,
            content,
            username: author_name,
        }
    }
}

impl From<CommitInfoShort> for activity::Activity {
    fn from(commit_info_short: CommitInfoShort) -> activity::Activity {
        let content = activity::ActivityContent::Commit(activity::Commit {
            sha1: commit_info_short.sha1,
            message: commit_info_short.message,
            author_email: commit_info_short.author_email,
            author_name: commit_info_short.author_name.clone(),
            timestamp: commit_info_short.timestamp,
        });

        activity::Activity {
            op_type: activity::OpType::CommitRepo,
            date: commit_info_short.timestamp,
            content,
            username: commit_info_short.author_name,
        }
    }
}

#[derive(Debug)]
pub struct GiteaClient {
    base_url: Url,
    username: String,
    _token: Option<String>,
    client: reqwest::Client,
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
            _token: Some(config.token.clone()),
        })
    }

    fn to_activity_repo(&self, gitea_repo: &GiteaRepo) -> activity::Repository {
        activity::Repository {
            owned_by_you: self.username.to_lowercase() == gitea_repo.owner.username.to_lowercase(),
            owner: gitea_repo.owner.username.clone(),
            name: gitea_repo.name.clone(),
            full_name: gitea_repo.full_name.clone(),
            html_url: gitea_repo.html_url.clone(),
            clone_url: gitea_repo.clone_url.clone(),
            private: gitea_repo.private,
            description: (!gitea_repo.description.is_empty())
                .then(|| gitea_repo.description.clone()),
            created_date: gitea_repo.created_at,
        }
    }
}

#[async_trait]
impl ServiceClient for GiteaClient {
    async fn get_repos(
        &self,
    ) -> anyhow::Result<HashMap<activity::Repository, HashSet<activity::Activity>>> {
        println!("Fetching activities from {}", self.base_url);
        let mut repos: HashMap<activity::Repository, HashSet<activity::Activity>> = HashMap::new();
        let mut page = 1;
        let limit = 50;
        loop {
            let url = format!(
                "{}/users/{}/activities/feeds?only-performed-by=true&page={}&limit={}",
                self.base_url, self.username, page, limit
            );

            let result: Vec<GiteaActivity> = self
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

            for activity in result {
                let repo = self.to_activity_repo(&activity.repo);
                let activities = repos.entry(repo).or_insert_with(HashSet::new);

                if let GiteaContent::Commit(c) = activity.content {
                    let mut count: i64 = (c.len as i64) - (c.commits.len() as i64);

                    let last_sha1 = c.commits.last().and_then(|lc| Some(lc.sha1.clone()));
                    activities.extend(c.commits.into_iter().map(|c| c.into()));

                    // Gitea users/username/activities only show maximum of 4 activities, so we dig further to get the rest
                    if count > 0 {
                        let mut page2 = 1;
                        // Super slow otherwise
                        let limit2 = count * 2;

                        'scroller: loop {
                            let url = format!(
                                "{}/repos/{}/commits?sha={}&page={}&limit={}",
                                self.base_url,
                                activity.repo.full_name,
                                last_sha1.clone().unwrap(),
                                page2,
                                limit2
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
                                // Should technically never land here
                                print!("!");
                                io::stdout().flush().ok().expect("Could not flush stdout");
                                break 'scroller;
                            }

                            for commit in result {
                                print!(".");
                                io::stdout().flush().ok().expect("Could not flush stdout");

                                if 0 >= count {
                                    break 'scroller;
                                }

                                let activity: activity::Activity = commit.into();

                                if !activities.contains(&activity)
                                    && activity.username.to_lowercase() == self.username
                                {
                                    activities.insert(activity);
                                    count -= 1;
                                }
                            }

                            page2 += 1;
                        }
                    }
                }
            }

            page += 1;
            print!(".");
            io::stdout().flush().ok().expect("Could not flush stdout");
        }

        Ok(repos)
    }
}

impl Hash for activity::Activity {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.op_type.hash(state);
        self.content.hash(state);
    }
}

impl Eq for activity::Activity {}
impl PartialEq for activity::Activity {
    fn eq(&self, other: &Self) -> bool {
        self.op_type == other.op_type && self.content == other.content
    }
}
