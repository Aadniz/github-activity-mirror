use super::ServiceClient;
use crate::config::ServiceConfig;
use async_trait::async_trait;
use url::Url;

#[derive(Debug)]
pub struct GiteaClient {
    // client: Client,
    base_url: Url,
    token: Option<String>,
}

impl GiteaClient {
    pub fn new(config: &ServiceConfig) -> anyhow::Result<Self> {
        let mut base_url = config.url.clone();
        base_url
            .path_segments_mut()
            .map_err(|_| anyhow::anyhow!("Invalid base URL"))?
            .extend("/api/v1".split('/'));

        Ok(Self {
            // client: Client::new(),
            base_url,
            token: Some(config.token.clone()),
        })
    }
}

#[async_trait]
impl ServiceClient for GiteaClient {
    async fn get_activities(&self) -> anyhow::Result<Vec<String>> {
        println!("Fetching activities from {}", self.base_url);
        // Simulate async work
        tokio::time::sleep(std::time::Duration::from_millis(100)).await;
        Ok(vec!["wow".to_string(), "damn".to_string()])
    }
}
