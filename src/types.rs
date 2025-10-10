use serde::{Deserialize, Serialize};
use std::path::PathBuf;

/// Configuration from the main project.md YAML front matter
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProjectConfig {
    pub backend: String,
    pub repo: String,
    #[serde(flatten)]
    pub extra: std::collections::HashMap<String, serde_yaml::Value>,
}

/// Status of a task in the project file
#[derive(Debug, Clone, PartialEq)]
pub enum TaskStatus {
    /// Existing issue with ID
    Existing(u64),
    /// New issue to be created
    New,
}

/// A single task item from the bulleted list
#[derive(Debug, Clone)]
pub struct TaskItem {
    pub status: TaskStatus,
    pub path: PathBuf,
    pub description: String,
}

/// YAML front matter from individual task files
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskFileConfig {
    pub issue_id: Option<u64>,
    #[serde(rename = "type")]
    pub task_type: Option<String>,
    pub tags: Option<Vec<String>>,
    #[serde(flatten)]
    pub extra: std::collections::HashMap<String, serde_yaml::Value>,
}

/// A parsed task file
#[derive(Debug, Clone)]
pub struct TaskFile {
    pub config: TaskFileConfig,
    pub title: String,
    pub body: String,
}

/// The complete parsed project.md document
#[derive(Debug)]
pub struct ProjectMd {
    pub config: ProjectConfig,
    pub tasks: Vec<TaskItem>,
}

impl TaskStatus {
    pub fn is_new(&self) -> bool {
        matches!(self, TaskStatus::New)
    }

    pub fn issue_id(&self) -> Option<u64> {
        match self {
            TaskStatus::Existing(id) => Some(*id),
            TaskStatus::New => None,
        }
    }
}
