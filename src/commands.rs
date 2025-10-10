use anyhow::{Context, Result};
use std::fs;
use std::path::Path;

use crate::backend::{Backend, github::GitHubBackend};
use crate::parser::parse_project_file;
use crate::sync::SyncEngine;
use crate::types::TaskStatus;

/// Execute the sync command
pub async fn sync(project_file: &Path, github_token: &str, dry_run: bool) -> Result<()> {
    let project_root = project_file.parent()
        .context("Failed to get project root directory")?
        .to_path_buf();

    // Read and parse project file
    let content = fs::read_to_string(project_file)
        .context("Failed to read project file")?;

    let project = parse_project_file(&content)?;

    // Validate backend
    if project.config.backend != "github" {
        anyhow::bail!("Unsupported backend: {}. Only 'github' is currently supported.",
            project.config.backend);
    }

    if dry_run {
        println!("DRY RUN: No changes will be made\n");
        println!("Would sync {} tasks to {}/{}\n",
            project.tasks.len(),
            project.config.backend,
            project.config.repo);

        for task in &project.tasks {
            match &task.status {
                TaskStatus::New => {
                    println!("  [CREATE] {} - {}", task.path.display(), task.description);
                }
                TaskStatus::Existing(num) => {
                    println!("  [UPDATE] #{} {} - {}", num, task.path.display(), task.description);
                }
            }
        }

        return Ok(());
    }

    // Create backend
    let backend = GitHubBackend::new(github_token, &project.config.repo)?;

    // Create sync engine and run sync
    let engine = SyncEngine::new(backend, project_root);
    let result = engine.sync(project_file).await?;

    // Print summary
    result.print_summary();

    if !result.errors.is_empty() {
        anyhow::bail!("Sync completed with errors");
    }

    Ok(())
}

/// Execute the status command
pub async fn status(project_file: &Path, github_token: Option<&str>, verbose: bool) -> Result<()> {
    // Read and parse project file
    let content = fs::read_to_string(project_file)
        .context("Failed to read project file")?;

    let project = parse_project_file(&content)?;

    println!("Project: {}", project_file.display());
    println!("Backend: {}", project.config.backend);
    println!("Repo: {}", project.config.repo);
    println!("\nTasks ({}):\n", project.tasks.len());

    for task in &project.tasks {
        match &task.status {
            TaskStatus::New => {
                println!("  [NEW] {} - {}", task.path.display(), task.description);
            }
            TaskStatus::Existing(num) => {
                println!("  [#{}] {} - {}", num, task.path.display(), task.description);
            }
        }

        if verbose {
            // Try to read the task file for more details
            let project_root = project_file.parent().unwrap_or(Path::new("."));
            let task_file_path = project_root.join(&task.path);

            if let Ok(task_content) = fs::read_to_string(&task_file_path) {
                if let Ok(task_file) = crate::parser::parse_task_file(&task_content) {
                    println!("       Title: {}", task_file.title);
                    if let Some(task_type) = &task_file.config.task_type {
                        println!("       Type: {}", task_type);
                    }
                    if let Some(tags) = &task_file.config.tags {
                        println!("       Tags: {}", tags.join(", "));
                    }
                }
            }
            println!();
        }
    }

    // If we have a token, we can fetch live status from backend
    let token_string;
    let token = match github_token {
        Some(t) => Some(t),
        None => {
            token_string = std::env::var("GITHUB_TOKEN").ok();
            token_string.as_deref()
        }
    };

    if let Some(token) = token {
        if project.config.backend == "github" {
            println!("\nFetching live status from GitHub...\n");

            let backend = GitHubBackend::new(token, &project.config.repo)?;
            let issues = backend.list_issues().await?;

            println!("Total issues in repository: {}", issues.len());

            let open_count = issues.iter().filter(|i| i.state == "open").count();
            let closed_count = issues.iter().filter(|i| i.state == "closed").count();

            println!("  Open: {}", open_count);
            println!("  Closed: {}", closed_count);
        }
    }

    Ok(())
}

/// Execute the init command
pub async fn init(backend: &str, repo: &str) -> Result<()> {
    let project_file = Path::new("project.md");

    if project_file.exists() {
        anyhow::bail!("project.md already exists");
    }

    let template = format!(
        r#"backend: {}
repo: {}
---

# My Project

Project description goes here.

## Tasks

* [new] - tasks/example.md - Example task

"#,
        backend, repo
    );

    fs::write(project_file, template)
        .context("Failed to write project.md")?;

    // Create tasks directory
    fs::create_dir_all("tasks")
        .context("Failed to create tasks directory")?;

    // Create example task file
    let example_task = r#"---
type: task
tags: [example]
---
# Example task

This is an example task file. Edit this to describe your task.

## Details

You can use full markdown here to describe:
- What needs to be done
- Why it's important
- Any technical details

When you run `projectmd sync`, this will be created as an issue in your backend.
"#;

    fs::write("tasks/example.md", example_task)
        .context("Failed to write example task")?;

    println!("Initialized new project.md with {} backend", backend);
    println!("Repository: {}", repo);
    println!("\nCreated:");
    println!("  - project.md");
    println!("  - tasks/example.md");
    println!("\nNext steps:");
    println!("  1. Edit project.md and tasks/example.md");
    println!("  2. Set GITHUB_TOKEN environment variable");
    println!("  3. Run: projectmd sync");

    Ok(())
}
