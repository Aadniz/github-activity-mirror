use serde::{Deserialize, Serialize};
use std::{fs, path::PathBuf};
use toml;
use url::Url;

use crate::github;
use crate::services::{self, gitea::GiteaClient};

// Taken from here https://github.com/awesome-selfhosted/awesome-selfhosted?tab=readme-ov-file#software-development---project-management
// For now there is only support for Gitea, but would be desirable to support all these (and more!!)
#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
enum ServiceType {
    CGit,
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
    Redmine,
    ReviewBoard,
    RGit,
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
    pub fn create_client(
        &self,
        config: &ServiceConfig,
    ) -> anyhow::Result<Box<dyn services::ServiceClient>> {
        match self {
            ServiceType::Gitea => Ok(Box::new(GiteaClient::new(config)?)),
            // ... other service implementations
            _ => anyhow::bail!("Service not yet implemented: {:?}", self),
        }
    }
}

#[derive(Deserialize, Serialize, Debug)]
pub struct ServiceConfig {
    pub service_type: ServiceType,
    username: String,
    pub url: Url,
    pub token: String,
    #[serde(skip)] // This field won't be loaded from config
    pub client: Option<Box<dyn services::ServiceClient + Send + Sync>>,
}
impl ServiceConfig {
    pub fn init_client(&mut self) -> anyhow::Result<()> {
        self.client = Some(self.service_type.create_client(self)?);
        Ok(())
    }
}

#[derive(Deserialize, Serialize, Debug)]
pub struct Config {
    pub services: Vec<ServiceConfig>,
    pub github: github::GithubConfig,
}

impl Config {
    pub fn load(path: PathBuf) -> anyhow::Result<Self> {
        let config_content = fs::read_to_string(path)?;
        let mut config: Config = toml::from_str(&config_content)?;

        for service in &mut config.services {
            service.init_client()?
        }

        Ok(config)
    }
}
