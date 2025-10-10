use pest::Parser;
use pest_derive::Parser;
use anyhow::{Context, Result};
use std::path::PathBuf;

use crate::types::{ProjectConfig, ProjectMd, TaskFile, TaskFileConfig, TaskItem, TaskStatus};

#[derive(Parser)]
#[grammar = "projectmd.pest"]
pub struct ProjectMdParser;

/// Parse a project.md file
pub fn parse_project_file(content: &str) -> Result<ProjectMd> {
    let mut pairs = ProjectMdParser::parse(Rule::document, content)
        .context("Failed to parse project file")?;

    let document = pairs.next().context("Empty document")?;

    let mut config = None;
    let mut tasks = Vec::new();

    for pair in document.into_inner() {
        match pair.as_rule() {
            Rule::frontmatter => {
                let yaml_content = pair.into_inner()
                    .next()
                    .context("Missing YAML content")?
                    .as_str();
                config = Some(parse_yaml_frontmatter(yaml_content)?);
            }
            Rule::content => {
                for content_pair in pair.into_inner() {
                    if let Rule::task_item = content_pair.as_rule() {
                        tasks.push(parse_task_item(content_pair)?);
                    }
                }
            }
            Rule::EOI => {}
            _ => {}
        }
    }

    let config = config.context("Missing YAML front matter")?;

    Ok(ProjectMd { config, tasks })
}

/// Parse a task markdown file
pub fn parse_task_file(content: &str) -> Result<TaskFile> {
    // Split by --- separator
    let parts: Vec<&str> = content.splitn(3, "---").collect();

    if parts.len() < 3 {
        anyhow::bail!("Invalid task file format: missing YAML front matter");
    }

    let yaml_content = parts[1].trim();
    let markdown_content = parts[2].trim();

    let config: TaskFileConfig = serde_yaml::from_str(yaml_content)
        .context("Failed to parse task file YAML front matter")?;

    // Extract title (first # heading) and body
    let (title, body) = extract_title_and_body(markdown_content);

    Ok(TaskFile {
        config,
        title,
        body,
    })
}

fn parse_yaml_frontmatter(yaml_str: &str) -> Result<ProjectConfig> {
    serde_yaml::from_str(yaml_str)
        .context("Failed to parse YAML front matter")
}

fn parse_task_item(pair: pest::iterators::Pair<Rule>) -> Result<TaskItem> {
    let mut status = None;
    let mut path = None;
    let mut description = None;

    for inner in pair.into_inner() {
        match inner.as_rule() {
            Rule::task_status => {
                status = Some(parse_task_status(inner)?);
            }
            Rule::task_path => {
                path = Some(PathBuf::from(inner.as_str()));
            }
            Rule::task_description => {
                description = Some(inner.as_str().to_string());
            }
            _ => {}
        }
    }

    Ok(TaskItem {
        status: status.context("Missing task status")?,
        path: path.context("Missing task path")?,
        description: description.context("Missing task description")?,
    })
}

fn parse_task_status(pair: pest::iterators::Pair<Rule>) -> Result<TaskStatus> {
    let inner = pair.into_inner().next().context("Empty task status")?;

    match inner.as_rule() {
        Rule::existing_issue => {
            let issue_num = inner.into_inner()
                .next()
                .context("Missing issue number")?
                .as_str()
                .parse::<u64>()
                .context("Invalid issue number")?;
            Ok(TaskStatus::Existing(issue_num))
        }
        Rule::new_issue => Ok(TaskStatus::New),
        _ => anyhow::bail!("Invalid task status"),
    }
}

fn extract_title_and_body(markdown: &str) -> (String, String) {
    let lines: Vec<&str> = markdown.lines().collect();

    let mut title = String::new();
    let mut body_lines = Vec::new();
    let mut found_title = false;

    for line in lines {
        let trimmed = line.trim();
        if !found_title && trimmed.starts_with("# ") {
            title = trimmed.trim_start_matches("# ").to_string();
            found_title = true;
        } else if found_title {
            body_lines.push(line);
        }
    }

    (title, body_lines.join("\n").trim().to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_project_file() {
        let content = r#"backend: github
repo: vagmi/projectmd
---

# Your glorious project name

Description paragraph.

* [#1] - tasks/setup_auth.md - setup the authentication
* [new] - tasks/scaffold_ui.md - Scaffold the UI
"#;

        let result = parse_project_file(content).unwrap();
        assert_eq!(result.config.backend, "github");
        assert_eq!(result.config.repo, "vagmi/projectmd");
        assert_eq!(result.tasks.len(), 2);

        assert_eq!(result.tasks[0].status, TaskStatus::Existing(1));
        assert_eq!(result.tasks[0].path.to_str().unwrap(), "tasks/setup_auth.md");

        assert_eq!(result.tasks[1].status, TaskStatus::New);
    }

    #[test]
    fn test_parse_task_file() {
        let content = r#"---
issue_id: 1
type: bug
tags: [chore, infra]
---
# Setup the authentication

Some details go here.
"#;

        let result = parse_task_file(content).unwrap();
        assert_eq!(result.config.issue_id, Some(1));
        assert_eq!(result.title, "Setup the authentication");
        assert_eq!(result.body, "Some details go here.");
    }
}
