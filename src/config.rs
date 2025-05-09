// Allow dead code because enums are not fully implemented yet
#![allow(dead_code)]
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use std::{fs, path::PathBuf};
use toml;

use crate::services::ServiceConfig;

#[derive(PartialEq, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PushMethod {
    Http,
    Ssh,
}
impl Default for PushMethod {
    fn default() -> Self {
        PushMethod::Ssh
    }
}
#[derive(PartialEq, Clone)]
pub enum RedactLevel {
    Off,
    PrivateRepos,
    PrivateReposNoCrossLinking,
    Encrypted, // TODO: These will be properly implemented in the future
    Hashed,
}

impl Serialize for RedactLevel {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        match self {
            RedactLevel::Off => serializer.serialize_u8(0),
            RedactLevel::PrivateRepos => serializer.serialize_u8(1),
            RedactLevel::PrivateReposNoCrossLinking => serializer.serialize_u8(2),
            RedactLevel::Encrypted => serializer.serialize_u8(3),
            RedactLevel::Hashed => serializer.serialize_u8(4),
        }
    }
}

impl<'de> Deserialize<'de> for RedactLevel {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let value = u8::deserialize(deserializer)?;
        match value {
            0 => Ok(RedactLevel::Off),
            1 => Ok(RedactLevel::PrivateRepos),
            2 => Ok(RedactLevel::PrivateReposNoCrossLinking),
            3 => Err(serde::de::Error::custom(
                "Encrypted redaction is not implemented yet",
            )), // Ok(RedactLevel::Encrypted),
            4 => Ok(RedactLevel::Hashed),
            _ => Err(serde::de::Error::custom(
                "Invalid redact level. Must be 0-4",
            )),
        }
    }
}

impl Default for RedactLevel {
    fn default() -> Self {
        RedactLevel::PrivateRepos
    }
}

#[derive(Deserialize, Serialize, Clone)]
pub struct GitConfig {
    pub username: String,
    pub token: String,
    pub email: Option<String>,
    #[serde(default)]
    pub redact_level: RedactLevel,
    #[serde(default)]
    pub push_method: PushMethod,
}

#[derive(Deserialize, Serialize)]
pub struct Config {
    pub services: Vec<ServiceConfig>,
    pub github: GitConfig,
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
