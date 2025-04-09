use chrono::DateTime;
use serde::{Deserialize, Serialize};
use url::Url;

#[derive(Debug, Deserialize, Serialize)]
pub struct Activity {
    pub id: u64,
    pub op_type: String,
    pub repo: Repository,
    pub created: DateTime<chrono::FixedOffset>,
    pub content: ActivityContent,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct Repository {
    pub id: u64,
    pub name: String,
    pub full_name: String,
    pub html_url: Url,
    pub clone_url: Url,
    pub private: bool,
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(untagged)]
pub enum ActivityContent {
    Commit {
        commits: Vec<Commit>,
        compare_url: String,
    },
    // Other activity types...
}

#[derive(Debug, Deserialize, Serialize)]
pub struct Commit {
    pub sha1: String,
    pub message: String,
    pub author_email: String,
    pub author_name: String,
    pub timestamp: DateTime<chrono::FixedOffset>,
}
