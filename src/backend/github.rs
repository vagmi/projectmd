use anyhow::{Context, Result};
use async_trait::async_trait;
use octocrab::Octocrab;

use super::{Backend, Issue};

/// GitHub backend using octocrab
pub struct GitHubBackend {
    client: Octocrab,
    owner: String,
    repo: String,
}

impl GitHubBackend {
    /// Create a new GitHub backend with a personal access token
    pub fn new(token: &str, repo: &str) -> Result<Self> {
        let client = Octocrab::builder()
            .personal_token(token.to_string())
            .build()
            .context("Failed to create GitHub client")?;

        // Parse owner/repo format
        let parts: Vec<&str> = repo.split('/').collect();
        if parts.len() != 2 {
            anyhow::bail!("Invalid repo format. Expected: owner/repo");
        }

        Ok(Self {
            client,
            owner: parts[0].to_string(),
            repo: parts[1].to_string(),
        })
    }

    /// Convert octocrab issue to our Issue type
    fn convert_issue(&self, issue: octocrab::models::issues::Issue) -> Issue {
        let state = match issue.state {
            octocrab::models::IssueState::Open => "open",
            octocrab::models::IssueState::Closed => "closed",
            _ => "unknown",
        };

        Issue {
            id: issue.id.0,
            number: issue.number,
            title: issue.title,
            body: issue.body.unwrap_or_default(),
            state: state.to_string(),
        }
    }
}

#[async_trait]
impl Backend for GitHubBackend {
    async fn create_issue(&self, title: &str, body: &str, labels: Vec<String>) -> Result<Issue> {
        let issue = self
            .client
            .issues(&self.owner, &self.repo)
            .create(title)
            .body(body)
            .labels(labels)
            .send()
            .await
            .context("Failed to create GitHub issue")?;

        Ok(self.convert_issue(issue))
    }

    async fn update_issue(&self, number: u64, title: &str, body: &str, labels: Vec<String>) -> Result<Issue> {
        let issue = self
            .client
            .issues(&self.owner, &self.repo)
            .update(number)
            .title(title)
            .body(body)
            .labels(&labels)
            .send()
            .await
            .context("Failed to update GitHub issue")?;

        Ok(self.convert_issue(issue))
    }

    async fn get_issue(&self, number: u64) -> Result<Issue> {
        let issue = self
            .client
            .issues(&self.owner, &self.repo)
            .get(number)
            .await
            .context("Failed to get GitHub issue")?;

        Ok(self.convert_issue(issue))
    }

    async fn list_issues(&self) -> Result<Vec<Issue>> {
        let page = self
            .client
            .issues(&self.owner, &self.repo)
            .list()
            .state(octocrab::params::State::All)
            .per_page(100)
            .send()
            .await
            .context("Failed to list GitHub issues")?;

        Ok(page.items.into_iter().map(|i| self.convert_issue(i)).collect())
    }
}
