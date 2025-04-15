use async_trait::async_trait;

use crate::activity::Activity;

pub mod gitea;

#[async_trait]
pub trait ServiceClient: std::fmt::Debug + Send + Sync {
    async fn get_activities(&self) -> anyhow::Result<Vec<Activity>>;
}
