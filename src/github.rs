use std::collections::HashMap;

use chrono::NaiveDate;
use serde::{Deserialize, Serialize};

use crate::activity::Activity;

#[derive(Deserialize, Serialize, Debug)]
pub struct GithubConfig {
    username: String,
    token: String,
}

impl GithubConfig {
    pub fn sync(&self, activities: Vec<Activity>) -> anyhow::Result<()> {
        let mut daily_activities: HashMap<NaiveDate, Vec<Activity>> = HashMap::new();
        let mut focus_date = NaiveDate::from_num_days_from_ce_opt(0).unwrap();
        for activity in activities {
            let naive_date = activity.date.date_naive();
            if focus_date != naive_date {
                focus_date = naive_date;
            }
            daily_activities
                .entry(naive_date)
                .or_insert_with(Vec::new)
                .push(activity);
        }

        for daily_activity in daily_activities {
            println!("{} - {}", daily_activity.0, daily_activity.1.len());
            for activity in daily_activity.1 {
                match activity.content {
                    crate::activity::ActivityContent::Commit(commit) => {
                        println!(
                            "  {:?} {} - {} - {} - {}",
                            activity.op_type,
                            commit.timestamp,
                            commit.sha1[0..10].to_string(),
                            activity.repo.full_name,
                            commit.message.trim()
                        )
                    } //_ => {
                      //    println!("  {:?}", activity.op_type,)
                      //}
                }
            }
        }

        Ok(())
    }
}
