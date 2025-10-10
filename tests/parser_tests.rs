use projectmd::parser::{parse_project_file, parse_task_file};
use projectmd::types::TaskStatus;
use std::fs;
use std::path::PathBuf;

/// Helper to load a fixture file
fn load_fixture(name: &str) -> String {
    let mut path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    path.push("tests");
    path.push("fixtures");
    path.push(name);
    fs::read_to_string(path).expect(&format!("Failed to load fixture: {}", name))
}

#[test]
fn test_simple_project() {
    let content = load_fixture("simple.md");
    let result = parse_project_file(&content).expect("Failed to parse simple.md");

    assert_eq!(result.config.backend, "github");
    assert_eq!(result.config.repo, "test/simple");
    assert_eq!(result.tasks.len(), 2);

    assert_eq!(result.tasks[0].status, TaskStatus::New);
    assert_eq!(result.tasks[0].path.to_str().unwrap(), "tasks/task1.md");
    assert_eq!(result.tasks[0].description, "First task");

    assert_eq!(result.tasks[1].status, TaskStatus::Existing(1));
    assert_eq!(result.tasks[1].path.to_str().unwrap(), "tasks/task2.md");
    assert_eq!(result.tasks[1].description, "Second task");
}

#[test]
fn test_trailing_newline() {
    let content = load_fixture("trailing_newline.md");
    let result = parse_project_file(&content).expect("Failed to parse trailing_newline.md");

    assert_eq!(result.config.backend, "github");
    assert_eq!(result.config.repo, "test/trailing");
    assert_eq!(result.tasks.len(), 2);
}

#[test]
fn test_multiple_newlines() {
    let content = load_fixture("multiple_newlines.md");
    let result = parse_project_file(&content).expect("Failed to parse multiple_newlines.md");

    assert_eq!(result.config.backend, "github");
    assert_eq!(result.config.repo, "test/multiple");
    assert_eq!(result.tasks.len(), 2);

    assert_eq!(result.tasks[0].status, TaskStatus::New);
    assert_eq!(result.tasks[1].status, TaskStatus::Existing(1));
}

#[test]
fn test_complex_project() {
    let content = load_fixture("complex.md");
    let result = parse_project_file(&content).expect("Failed to parse complex.md");

    assert_eq!(result.config.backend, "github");
    assert_eq!(result.config.repo, "test/complex");

    // Check extra field
    assert!(result.config.extra.contains_key("extra_field"));

    // Should find 4 tasks despite all the extra content
    assert_eq!(result.tasks.len(), 4);

    assert_eq!(result.tasks[0].status, TaskStatus::New);
    assert_eq!(result.tasks[0].description, "Setup the project");

    assert_eq!(result.tasks[1].status, TaskStatus::Existing(1));
    assert_eq!(result.tasks[1].description, "Build the application");

    assert_eq!(result.tasks[2].status, TaskStatus::Existing(42));
    assert_eq!(result.tasks[2].description, "Deploy to production");

    assert_eq!(result.tasks[3].status, TaskStatus::New);
    assert_eq!(result.tasks[3].description, "Write tests");
}

#[test]
fn test_no_tasks() {
    let content = load_fixture("no_tasks.md");
    let result = parse_project_file(&content).expect("Failed to parse no_tasks.md");

    assert_eq!(result.config.backend, "github");
    assert_eq!(result.config.repo, "test/notasks");
    assert_eq!(result.tasks.len(), 0);
}

#[test]
fn test_mixed_content() {
    let content = load_fixture("mixed_content.md");
    let result = parse_project_file(&content).expect("Failed to parse mixed_content.md");

    assert_eq!(result.config.backend, "github");
    assert_eq!(result.config.repo, "test/mixed");
    assert_eq!(result.tasks.len(), 4);

    // Verify tasks are parsed correctly despite mixed content
    assert_eq!(result.tasks[0].status, TaskStatus::New);
    assert_eq!(result.tasks[0].path.to_str().unwrap(), "tasks/first.md");

    assert_eq!(result.tasks[1].status, TaskStatus::Existing(10));
    assert_eq!(result.tasks[1].path.to_str().unwrap(), "tasks/second.md");

    assert_eq!(result.tasks[2].status, TaskStatus::Existing(20));
    assert_eq!(result.tasks[2].path.to_str().unwrap(), "tasks/third.md");

    assert_eq!(result.tasks[3].status, TaskStatus::New);
    assert_eq!(result.tasks[3].path.to_str().unwrap(), "tasks/fourth.md");
}

#[test]
fn test_all_fixtures_parse() {
    // Ensure all fixtures can be parsed without errors
    let fixtures = vec![
        "simple.md",
        "trailing_newline.md",
        "multiple_newlines.md",
        "complex.md",
        "no_tasks.md",
        "mixed_content.md",
    ];

    for fixture in fixtures {
        let content = load_fixture(fixture);
        parse_project_file(&content)
            .expect(&format!("Failed to parse fixture: {}", fixture));
    }
}

#[test]
fn test_issue_numbers() {
    let content = load_fixture("complex.md");
    let result = parse_project_file(&content).unwrap();

    let issue_42 = result.tasks.iter().find(|t| {
        matches!(t.status, TaskStatus::Existing(42))
    });
    assert!(issue_42.is_some());
    assert_eq!(issue_42.unwrap().path.to_str().unwrap(), "tasks/deploy.md");
}

#[test]
fn test_yaml_frontmatter_extra_fields() {
    let content = load_fixture("complex.md");
    let result = parse_project_file(&content).unwrap();

    // Test that extra fields in YAML are preserved
    assert_eq!(result.config.extra.get("extra_field").and_then(|v| v.as_str()), Some("some_value"));
}

#[test]
fn test_task_file_with_timestamps() {
    let mut path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    path.push("tests");
    path.push("fixtures");
    path.push("with_timestamps.md");
    let content = fs::read_to_string(&path).expect("Failed to load with_timestamps.md");

    let result = parse_task_file(&content).expect("Failed to parse task file with timestamps");

    assert_eq!(result.config.issue_id, Some(5));
    assert_eq!(result.config.task_type.as_deref(), Some("feature"));
    assert_eq!(result.config.created_at.as_deref(), Some("2025-01-15T10:30:00Z"));
    assert_eq!(result.config.updated_at.as_deref(), Some("2025-01-20T15:45:32Z"));
    assert_eq!(result.title, "API with timestamps");
}

#[test]
fn test_task_file_without_timestamps() {
    // Task files without timestamps should still parse correctly
    let content = r#"---
issue_id: 1
type: bug
tags: [chore, infra]
---
# Setup the authentication

Some details go here.
"#;

    let result = parse_task_file(content).expect("Failed to parse task file without timestamps");

    assert_eq!(result.config.issue_id, Some(1));
    assert_eq!(result.config.created_at, None);
    assert_eq!(result.config.updated_at, None);
    assert_eq!(result.title, "Setup the authentication");
}
