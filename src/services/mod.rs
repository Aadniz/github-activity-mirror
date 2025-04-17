use std::collections::{HashMap, HashSet};

use async_trait::async_trait;

use crate::activity::{Activity, Repository};

pub mod gitea;

#[async_trait]
pub trait ServiceClient: std::fmt::Debug + Send + Sync {
    async fn get_repos(&self) -> anyhow::Result<HashMap<Repository, HashSet<Activity>>>;
}
