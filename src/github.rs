use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize, Debug)]
pub struct GithubConfig {
    username: String,
    token: String,
}

impl GithubConfig {
    pub fn sync(&self, strngs: Vec<String>) -> anyhow::Result<()> {
        Ok(())
    }
}
