use chrono::DateTime;
use serde::{Deserialize, Serialize};
use url::Url;

// https://github.com/go-gitea/gitea/blob/921d3a394de244de83650fa5dcc4866b085cf72b/models/activities/action.go#L66
#[derive(Debug, Deserialize, Serialize, PartialEq, Eq, Hash)]
#[serde(rename_all = "snake_case")]
pub enum OpType {
    CreateRepo,
    RenameRepo,
    StarRepo,
    WatchRepo,
    CommitRepo,
    CreateIssue,
    CreatePullRequest,
    TransferRepo,
    PushTag,
    CommentIssue,
    MergePullRequest,
    CloseIssue,
    ReopenIssue,
    ClosePullRequest,
    ReopenPullRequest,
    DeleteTag,
    DeleteBranch,
    MirrorSyncPush,
    MirrorSyncCreate,
    MirrorSyncDelete,
    ApprovePullRequest,
    RejectPullRequest,
    CommentPull,
    PublishRelease,
    PullReviewDismissed,
    PullRequestReadyForReview,
    AutoMergePullRequest,
}

#[derive(Deserialize, Serialize)]
pub struct Activity {
    pub op_type: OpType,
    pub date: DateTime<chrono::FixedOffset>,
    pub content: ActivityContent,
    pub source_link: Url,
    pub username: String,
    pub email: String,
}

#[derive(Deserialize, Serialize, Eq, Hash, PartialEq)]
pub struct Repository {
    // If it is owned under you, or if it is a repo under an organization or a friend
    pub owned_by_you: bool,
    pub owner: String,
    pub name: String,
    pub description: Option<String>,
    pub full_name: String,
    pub html_url: Url,
    pub clone_url: Url,
    pub private: bool,
    pub created_date: DateTime<chrono::FixedOffset>,
}

#[derive(Deserialize, Serialize)]
#[serde(untagged)]
pub enum ActivityContent {
    Commit(Commit),
    Issue(Issue),
    // Other activity types...
}

#[derive(Deserialize, Serialize)]
pub struct Commit {
    pub sha1: String,
    pub message: String,
    pub author_email: String,
    pub author_name: String,
    pub timestamp: DateTime<chrono::FixedOffset>,
}

#[derive(Deserialize, Serialize)]
pub struct Issue {
    pub issue_id: u64,
    pub message: String,
}
