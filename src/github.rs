use serde::{Deserialize, Serialize};

use crate::activity::Activity;

#[derive(Deserialize, Serialize, Debug)]
pub struct GithubConfig {
    username: String,
    token: String,
}

impl GithubConfig {
    pub fn sync(&self, activities: Vec<Activity>) -> anyhow::Result<()> {
        for activity in activities {
            println!(
                "{:?} - {} - {}",
                activity.op_type, activity.created, activity.repo.full_name
            );
            match activity.content {
                crate::activity::ActivityContent::Commit {
                    commits,
                    compare_url,
                } => {
                    for commit in commits {
                        println!(
                            "  {} - {} - {}",
                            commit.timestamp, commit.sha1, commit.message
                        )
                    }
                }
            }
        }

        Ok(())
    }
}
