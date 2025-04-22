use std::collections::{HashMap, HashSet};

use async_trait::async_trait;
use gitea::GiteaClient;
use serde::{Deserialize, Serialize};
use url::Url;

use crate::activity::{Activity, Repository};

pub mod gitea;

// Taken from here https://github.com/awesome-selfhosted/awesome-selfhosted?tab=readme-ov-file#software-development---project-management
// For now there is only support for Gitea, but would be desirable to support all these (and more!!)
#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum ServiceType {
    Bitbucket,
    CGit,
    Codebase,
    Codeberg,
    Forgejo,
    Fossil,
    Gerrit,
    Gitblit,
    Gitbucket,
    Gitea,
    Gitlab,
    Gitolite,
    Gogs,
    Huly,
    Kallithea,
    Klaus,
    Launchpad,
    Leantime,
    Mindwendel,
    MinimalGitServer,
    Octobox,
    OneDev,
    OpenProject,
    Pagure,
    Phorge,
    Plane,
    ProjeQtOr,
    RGit,
    Redmine,
    ReviewBoard,
    RhodeCode,
    Rukovoditel,
    SCMManager,
    Smederee,
    Sourcehut,
    Taiga,
    Titra,
    Trac,
    Traq,
    Tuleap,
    UVDesk,
    ZenTao,
}
impl ServiceType {
    pub fn create_client(&self, config: &ServiceConfig) -> anyhow::Result<Box<dyn ServiceClient>> {
        match self {
            ServiceType::Gitea => Ok(Box::new(GiteaClient::new(config)?)),
            ServiceType::Codeberg => Ok(Box::new(GiteaClient::new(config)?)), // Pretty much identical API to Gitea
            // ... other service implementations
            _ => anyhow::bail!("Service not yet implemented: {:?}", self),
        }
    }
}

#[derive(Deserialize, Serialize)]
pub struct ServiceConfig {
    pub service_type: ServiceType,
    pub username: String,
    pub url: Url,
    pub token: String,
    #[serde(skip)] // This field won't be loaded from config
    pub client: Option<Box<dyn ServiceClient + Send + Sync>>,
}
impl ServiceConfig {
    pub fn init_client(&mut self) -> anyhow::Result<()> {
        self.client = Some(self.service_type.create_client(self)?);
        Ok(())
    }
}

#[async_trait]
pub trait ServiceClient: Send + Sync {
    async fn get_repos(&self) -> anyhow::Result<HashMap<Repository, HashSet<Activity>>>;
}
