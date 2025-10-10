use anyhow::{Context, Result};
use std::path::{Path, PathBuf};
use std::fs;
use std::time::SystemTime;
use chrono::{DateTime, Utc};

use crate::backend::Backend;
use crate::parser::{parse_project_file, parse_task_file};
use crate::types::{TaskItem, TaskStatus, TaskFileConfig};

/// Check if a task should be synced based on file modification time
fn should_sync_task(task_file_path: &Path, config: &TaskFileConfig) -> Result<bool> {
    // Get file modification time
    let metadata = fs::metadata(task_file_path)?;
    let mtime: SystemTime = metadata.modified()?;
    let mtime_utc: DateTime<Utc> = mtime.into();

    // If no updated_at, always sync (first time)
    let Some(updated_at_str) = &config.updated_at else {
        return Ok(true);
    };

    // Parse stored updated_at timestamp
    let updated_at = DateTime::parse_from_rfc3339(updated_at_str)
        .context("Failed to parse updated_at timestamp")?
        .with_timezone(&Utc);

    // Only sync if file was modified after last sync
    Ok(mtime_utc > updated_at)
}

/// Sync engine for managing project tasks and backend issues
pub struct SyncEngine<B: Backend> {
    backend: B,
    project_root: PathBuf,
}

impl<B: Backend> SyncEngine<B> {
    pub fn new(backend: B, project_root: PathBuf) -> Self {
        Self {
            backend,
            project_root,
        }
    }

    /// Sync all tasks in the project file with the backend
    pub async fn sync(&self, project_file: &Path) -> Result<SyncResult> {
        let content = fs::read_to_string(project_file)
            .context("Failed to read project file")?;

        let project = parse_project_file(&content)?;

        let mut result = SyncResult {
            created: Vec::new(),
            updated: Vec::new(),
            skipped: Vec::new(),
            errors: Vec::new(),
        };

        for task_item in &project.tasks {
            match self.sync_task_item(task_item).await {
                Ok(action) => match action {
                    SyncAction::Created(issue_num) => {
                        result.created.push((task_item.path.clone(), issue_num));
                    }
                    SyncAction::Updated(issue_num) => {
                        result.updated.push((task_item.path.clone(), issue_num));
                    }
                    SyncAction::Skipped => {
                        result.skipped.push(task_item.path.clone());
                    }
                },
                Err(e) => {
                    result.errors.push((task_item.path.clone(), e.to_string()));
                }
            }
        }

        // Update project.md with new issue numbers
        if !result.created.is_empty() {
            self.update_project_file(project_file, &content, &result.created)?;
        }

        Ok(result)
    }

    /// Sync a single task item
    async fn sync_task_item(&self, task_item: &TaskItem) -> Result<SyncAction> {
        let task_file_path = self.project_root.join(&task_item.path);

        // Read and parse the task file
        let task_content = fs::read_to_string(&task_file_path)
            .with_context(|| format!("Failed to read task file: {:?}", task_file_path))?;

        let task_file = parse_task_file(&task_content)?;

        // Check if we need to sync this task (only for existing issues)
        if matches!(task_item.status, TaskStatus::Existing(_)) {
            if !should_sync_task(&task_file_path, &task_file.config)? {
                return Ok(SyncAction::Skipped);
            }
        }

        // Extract labels from tags
        let labels = task_file.config.tags
            .clone()
            .unwrap_or_default()
            .into_iter()
            .map(|s| s.to_string())
            .collect();

        match &task_item.status {
            TaskStatus::New => {
                // Create new issue
                let issue = self.backend
                    .create_issue(&task_file.title, &task_file.body, labels)
                    .await?;

                // Update the task file with the new issue ID and timestamps
                self.update_task_file_with_metadata(&task_file_path, &task_content, issue.number, true)?;

                Ok(SyncAction::Created(issue.number))
            }
            TaskStatus::Existing(issue_num) => {
                // Check if the task file has been modified (issue_id should match)
                if task_file.config.issue_id.is_none() ||
                   task_file.config.issue_id != Some(*issue_num) {
                    // Update the task file to match the project file
                    self.update_task_file_with_metadata(&task_file_path, &task_content, *issue_num, false)?;
                }

                // Update the issue
                let issue = self.backend
                    .update_issue(*issue_num, &task_file.title, &task_file.body, labels)
                    .await?;

                // Update the updated_at timestamp
                self.update_task_file_with_metadata(&task_file_path, &task_content, *issue_num, false)?;

                Ok(SyncAction::Updated(issue.number))
            }
        }
    }

    /// Update the task file with issue_id and timestamps
    fn update_task_file_with_metadata(
        &self,
        path: &Path,
        content: &str,
        issue_id: u64,
        is_new: bool
    ) -> Result<()> {
        // Parse the file to get the config
        let task_file = parse_task_file(content)?;

        // Update the config
        let mut updated_config = task_file.config;
        updated_config.issue_id = Some(issue_id);

        // Set timestamps
        let now = Utc::now().to_rfc3339();

        if is_new || updated_config.created_at.is_none() {
            updated_config.created_at = Some(now.clone());
        }

        updated_config.updated_at = Some(now);

        // Serialize back to YAML
        let yaml_str = serde_yaml::to_string(&updated_config)?;

        // Reconstruct the file
        let parts: Vec<&str> = content.splitn(3, "---").collect();
        if parts.len() < 3 {
            anyhow::bail!("Invalid task file format");
        }

        let updated_content = format!("---\n{}\n---\n{}", yaml_str.trim(), parts[2]);

        fs::write(path, updated_content)
            .context("Failed to write updated task file")?;

        Ok(())
    }

    /// Update project.md with new issue numbers
    fn update_project_file(&self, project_file: &Path, content: &str, created: &[(PathBuf, u64)]) -> Result<()> {
        let mut updated_content = content.to_string();

        for (task_path, issue_num) in created {
            // Find and replace [new] - path - with [#issue_num] - path -
            let task_path_str = task_path.to_string_lossy();

            // Pattern to match: * [new] - path/to/file.md -
            let pattern = format!("* [new] - {} -", task_path_str);
            let replacement = format!("* [#{}] - {} -", issue_num, task_path_str);

            updated_content = updated_content.replace(&pattern, &replacement);
        }

        fs::write(project_file, updated_content)
            .context("Failed to write updated project file")?;

        Ok(())
    }
}

#[derive(Debug)]
pub enum SyncAction {
    Created(u64),
    Updated(u64),
    Skipped,
}

#[derive(Debug)]
pub struct SyncResult {
    pub created: Vec<(PathBuf, u64)>,
    pub updated: Vec<(PathBuf, u64)>,
    pub skipped: Vec<PathBuf>,
    pub errors: Vec<(PathBuf, String)>,
}

impl SyncResult {
    pub fn print_summary(&self) {
        println!("\n=== Sync Summary ===");

        if !self.created.is_empty() {
            println!("\nCreated ({}):", self.created.len());
            for (path, issue_num) in &self.created {
                println!("  - {} -> Issue #{}", path.display(), issue_num);
            }
        }

        if !self.updated.is_empty() {
            println!("\nUpdated ({}):", self.updated.len());
            for (path, issue_num) in &self.updated {
                println!("  - {} -> Issue #{}", path.display(), issue_num);
            }
        }

        if !self.skipped.is_empty() {
            println!("\nSkipped (no changes) ({}):", self.skipped.len());
            for path in &self.skipped {
                println!("  ✓ {}", path.display());
            }
        }

        if !self.errors.is_empty() {
            println!("\nErrors ({}):", self.errors.len());
            for (path, error) in &self.errors {
                println!("  - {}: {}", path.display(), error);
            }
        }

        println!("\nTotal: {} tasks processed",
            self.created.len() + self.updated.len() + self.skipped.len() + self.errors.len());
    }
}
