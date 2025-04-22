use anyhow::{Context, Result};
use chrono::DateTime;
use std::path::PathBuf;
use std::process::Command;

use crate::activity::{self, Activity};
use crate::config::{GitConfig, PushMethod};

const MARK_STRING: &str = "<sub>This repo was mirrored using [github-activity-mirror](https://codeberg.org/Aadniz/github-activity-mirror), preserving the privacy while at the same time display your actual activity</sub>";
const BRANCH: &str = "main";

pub struct Git {
    config: GitConfig,
}
impl Git {
    pub fn new(config: GitConfig) -> Self {
        Self { config }
    }

    pub fn get_path(&self, repo: &octocrab::models::Repository) -> PathBuf {
        let temp_dir = std::env::temp_dir();
        let repo_name = repo
            .full_name
            .as_ref()
            .context("No repository name")
            .unwrap();
        let repo_path = temp_dir.join(repo_name.replace('/', "_"));

        repo_path
    }

    pub fn create_init(
        &self,
        repo: &octocrab::models::Repository,
        activity: &Activity,
    ) -> Result<()> {
        let repo_path = self.initialize_local_git(repo)?;

        // Create README.md file
        std::fs::write(repo_path.join("README.md"), MARK_STRING)?;

        // Stage the file
        self.run_git_command(&repo_path, &["add", "README.md"])?;

        // Create commit with specific date
        self.commit(repo, "Initial commit".to_string(), activity.date)?;

        // Push the changes
        self.push(&repo)?;

        Ok(())
    }

    pub fn push(&self, repo: &octocrab::models::Repository) -> Result<()> {
        let repo_path = self.initialize_local_git(repo)?;
        self.run_git_command(&repo_path, &["push", "--set-upstream", "origin", BRANCH])?;
        Ok(())
    }

    fn initialize_local_git(&self, repo: &octocrab::models::Repository) -> Result<PathBuf> {
        let temp_dir = std::env::temp_dir();
        let repo_name = repo.full_name.as_ref().context("No repository name")?;
        let repo_path = temp_dir.join(repo_name.replace('/', "_"));

        // Clone the repository
        if repo_path.exists() {
            let _ = self.run_git_command(&repo_path, &["fetch", "origin"]);

            let branch_check = Command::new("git")
                .current_dir(&repo_path)
                .args(["ls-remote", "--heads", "origin", BRANCH])
                .output();

            if let Ok(output) = branch_check {
                if !output.stdout.is_empty() {
                    // Only pull if main branch exists
                    self.run_git_command(&repo_path, &["pull", "origin", BRANCH])?;
                }
            }
            return Ok(repo_path);
        }

        let clone_url: String = match self.config.push_method {
            PushMethod::Http => repo
                .clone_url
                .as_ref()
                .context("No HTTP clone URL available")?
                .to_string(),
            PushMethod::Ssh => repo
                .ssh_url
                .as_ref()
                .context("No SSH URL available")?
                .to_string(),
        };
        self.run_git_command(
            &temp_dir,
            &["clone", &clone_url, repo_path.to_str().unwrap()],
        )?;

        Ok(repo_path)
    }

    pub fn add_commit(
        &self,
        repo: &octocrab::models::Repository,
        commit_message: String,
        commit_content: String,
        date: DateTime<chrono::FixedOffset>,
    ) -> anyhow::Result<()> {
        let repo_path = self.get_path(repo);

        // Update README
        std::fs::write(
            repo_path.join("README.md"),
            format!("{}\n\n{}", commit_content, MARK_STRING),
        )?;

        self.run_git_command(&repo_path, &["add", "README.md"])?;
        self.commit(repo, commit_message, date)?;

        Ok(())
    }

    fn commit(
        &self,
        repo: &octocrab::models::Repository,
        commit_message: String,
        date: DateTime<chrono::FixedOffset>,
    ) -> anyhow::Result<()> {
        let repo_path = self.get_path(repo);

        let date_str = date.format("%Y-%m-%dT%H:%M:%SZ").to_string();
        let mut command = Command::new("git");
        let args = [
            "-c",
            &format!("user.name={}", self.config.username),
            "-c",
            &format!("user.email={}", self.config.email.clone().unwrap()),
            "commit",
            "-m",
            &commit_message,
            "--date",
            &date_str,
        ];
        let output = command
            .current_dir(&repo_path)
            .env("GIT_COMMITTER_DATE", &date_str)
            .args(args)
            .output()
            .context("Failed to execute git commit")?;

        if !output.status.success() {
            let stdout = String::from_utf8_lossy(&output.stdout);
            let stderr = String::from_utf8_lossy(&output.stderr);

            // Handle "nothing to commit"
            if stderr.is_empty()
                && stdout
                    .trim()
                    .ends_with("nothing to commit, working tree clean")
            {
                eprintln!("WARNING: Possible duplicate commit: {}", commit_message);
            } else {
                anyhow::bail!(
                    "Git command '{}$ git {}' failed with exit code {}\nstdout: {}\nstderr: {}",
                    repo_path.as_os_str().to_str().unwrap(),
                    args.join(" "),
                    output.status.code().unwrap(),
                    String::from_utf8_lossy(&output.stdout),
                    String::from_utf8_lossy(&output.stderr)
                );
            }
        }
        Ok(())
    }

    pub fn last_commit(
        &self,
        repo: &octocrab::models::Repository,
    ) -> anyhow::Result<activity::Commit> {
        let repo_path = self.initialize_local_git(repo)?;

        // Get the last commit details using git log
        let output = Command::new("git")
            .current_dir(&repo_path)
            .args([
                "log",
                "-1",                                // Only the most recent commit
                "--pretty=format:%H|%ae|%an|%aI|%s", // Format: sha|email|name|iso-date|message
            ])
            .output()
            .context("Failed to execute git log command")?;

        if !output.status.success() {
            anyhow::bail!(
                "Git log failed: {}",
                String::from_utf8_lossy(&output.stderr)
            );
        }

        let log_output = String::from_utf8(output.stdout)
            .context("Failed to parse git log output")?
            .trim()
            .to_string();

        if log_output.is_empty() {
            anyhow::bail!("Repository has no commits");
        }

        // Parse the git log output
        let mut parts = log_output.splitn(5, '|');
        let sha1 = parts.next().context("Missing SHA")?.to_string();
        let author_email = parts.next().context("Missing author email")?.to_string();
        let author_name = parts.next().context("Missing author name")?.to_string();
        let timestamp = parts
            .next()
            .and_then(|s| DateTime::parse_from_rfc3339(s).ok())
            .context("Invalid timestamp format")?;
        let message = parts.next().context("Missing message")?.to_string();

        Ok(activity::Commit {
            sha1,
            message,
            author_email,
            author_name,
            timestamp,
        })
    }

    pub fn unpushed_commits(&self, repo: &octocrab::models::Repository) -> anyhow::Result<u32> {
        let repo_path = self.get_path(repo);

        // Check for unpushed commits
        let output = Command::new("git")
            .current_dir(&repo_path)
            .args(["cherry", "-v", &format!("origin/{}", BRANCH)])
            .output()
            .context("Failed to execute git cherry command")?;

        if !output.status.success() {
            anyhow::bail!(
                "git cherry failed: {}",
                String::from_utf8_lossy(&output.stderr)
            );
        }

        let count = String::from_utf8(output.stdout)?
            .lines()
            .filter(|line| !line.trim().is_empty())
            .count() as u32;

        Ok(count)
    }

    fn run_git_command(&self, cwd: &PathBuf, args: &[&str]) -> Result<()> {
        let output = Command::new("git")
            .current_dir(cwd)
            .args(args)
            .output()
            .context("Failed to execute git command")?;

        if !output.status.success() {
            anyhow::bail!(
                "Git command '{}$ git {}' failed with exit code {}\nstdout: {}\nstderr: {}",
                cwd.as_os_str().to_str().unwrap(),
                args.join(" "),
                output.status.code().unwrap(),
                String::from_utf8_lossy(&output.stdout),
                String::from_utf8_lossy(&output.stderr)
            );
        }

        Ok(())
    }
}
