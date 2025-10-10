use anyhow::Result;
use async_trait::async_trait;

pub mod github;

/// Represents an issue in the backend system
#[derive(Debug, Clone)]
pub struct Issue {
    pub id: u64,
    pub number: u64,
    pub title: String,
    pub body: String,
    pub state: String,
}

/// Backend trait for issue management
#[async_trait]
pub trait Backend: Send + Sync {
    /// Create a new issue
    async fn create_issue(&self, title: &str, body: &str, labels: Vec<String>) -> Result<Issue>;

    /// Update an existing issue
    async fn update_issue(&self, number: u64, title: &str, body: &str, labels: Vec<String>) -> Result<Issue>;

    /// Get an issue by number
    async fn get_issue(&self, number: u64) -> Result<Issue>;

    /// List all issues
    async fn list_issues(&self) -> Result<Vec<Issue>>;
}
