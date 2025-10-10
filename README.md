# ProjectMD

A plain text, LLM-friendly project management system for hackers who live in the CLI.

ProjectMD uses markdown files with YAML front matter to manage tasks and automatically syncs them with backend issue trackers like GitHub Issues. Perfect for developers who prefer working with text files and want AI assistants to easily understand and manage their projects.

## Features

- 📝 **Plain text format** - All project data in markdown files with YAML front matter
- 🤖 **LLM-friendly** - Designed for easy parsing by AI coding assistants
- 🔄 **Two-way sync** - Create and update GitHub Issues from markdown files
- 🎯 **Simple syntax** - Bulleted lists with status markers
- 🔌 **Backend agnostic** - Currently supports GitHub, extensible to GitLab, Jira, etc.
- ⚡ **Fast** - Built with Rust using pest parser

## Installation

```bash
cargo install --path .
```

Or build from source:

```bash
cargo build --release
# Binary will be at target/release/projectmd
```

## Quick Start

1. **Initialize a new project:**

```bash
projectmd init --backend github --repo your-username/your-repo
```

This creates:
- `project.md` - Main project file with task list
- `tasks/` - Directory for individual task files

2. **Set your GitHub token:**

```bash
export GITHUB_TOKEN=ghp_your_token_here
```

Or pass it via CLI:

```bash
projectmd --github-token ghp_your_token_here sync
```

3. **Edit your tasks and sync:**

```bash
# Edit project.md and task files
vim project.md
vim tasks/example.md

# Sync to GitHub
projectmd sync
```

## Usage

### Commands

#### `init` - Initialize a new project

```bash
projectmd init --backend github --repo owner/repo
```

Creates a new project with example files.

#### `sync` - Sync tasks with backend

```bash
# Sync all tasks
projectmd sync

# Dry run (preview changes without syncing)
projectmd sync --dry-run

# Use a different project file
projectmd -p my-project.md sync
```

The sync command will:
- Create new GitHub issues for tasks marked `[new]`
- Update existing issues for tasks marked `[#123]`
- Update task files with issue IDs after creation

#### `status` - Show task status

```bash
# Show all tasks
projectmd status

# Show with detailed information
projectmd status -v

# With GitHub token, also fetches live issue stats
GITHUB_TOKEN=xxx projectmd status -v
```

## File Format

### Project File (`project.md`)

The main project file contains YAML front matter with backend configuration and a bulleted list of tasks:

```markdown
backend: github
repo: vagmi/projectmd
---

# My Awesome Project

Project description goes here. You can use any markdown formatting.

## Tasks

* [#1] - tasks/setup_auth.md - Setup authentication system
* [#2] - tasks/api_endpoints.md - Create REST API endpoints
* [new] - tasks/frontend_ui.md - Build the frontend UI
* [new] - tasks/testing.md - Add comprehensive tests

## Notes

Any other markdown content is ignored by the parser.
```

**YAML Front Matter Fields:**
- `backend` - Backend type (currently only `github`)
- `repo` - Repository in `owner/repo` format

**Task List Format:**
- `* [#123]` - Existing issue (will be updated on sync)
- `* [new]` - New task (will create issue on sync)
- Followed by: ` - path/to/file.md - Task description`

### Task Files (`tasks/*.md`)

Individual task files contain details about each task:

```markdown
---
issue_id: 1
type: feature
tags: [backend, authentication, security]
---
# Setup authentication system

Implement JWT-based authentication with the following features:

## Requirements

- User registration and login
- Token refresh mechanism
- Password reset flow
- OAuth2 integration (Google, GitHub)

## Technical Details

- Use bcrypt for password hashing
- Store tokens in Redis for quick invalidation
- Implement rate limiting on auth endpoints

## Acceptance Criteria

- [ ] Users can register with email/password
- [ ] Users can login and receive JWT token
- [ ] Token refresh works correctly
- [ ] Password reset emails are sent
```

**YAML Front Matter Fields:**
- `issue_id` - GitHub issue number (auto-populated after first sync)
- `type` - Issue type (bug, feature, task, etc.)
- `tags` - Array of labels for the issue

The first `#` heading becomes the issue title, and everything after becomes the issue body.

## Examples

### Creating a New Project

```bash
# Initialize project
projectmd init --backend github --repo myusername/myproject

# Edit the project file
cat > project.md << 'EOF'
backend: github
repo: myusername/myproject
---

# My Web App

Building a modern web application.

* [new] - tasks/setup_backend.md - Setup Node.js backend
* [new] - tasks/setup_frontend.md - Setup React frontend
* [new] - tasks/deploy.md - Deploy to production
EOF

# Create task files
mkdir -p tasks

cat > tasks/setup_backend.md << 'EOF'
---
type: task
tags: [backend, setup]
---
# Setup Node.js backend

Initialize the backend with Express and PostgreSQL.

## Steps
1. Create Express app
2. Setup PostgreSQL database
3. Create initial migrations
4. Add authentication middleware
EOF

# Sync to GitHub
export GITHUB_TOKEN=ghp_xxxxxxxxxxxxx
projectmd sync
```

### Checking Status

```bash
# Basic status
projectmd status

# Output:
# Project: project.md
# Backend: github
# Repo: myusername/myproject
#
# Tasks (3):
#
#   [#1] tasks/setup_backend.md - Setup Node.js backend
#   [#2] tasks/setup_frontend.md - Setup React frontend
#   [#3] tasks/deploy.md - Deploy to production

# Detailed status with live GitHub data
GITHUB_TOKEN=xxx projectmd status -v
```

## Development

### Project Structure

```
projectmd/
├── src/
│   ├── main.rs          # CLI entry point
│   ├── cli.rs           # Command definitions
│   ├── commands.rs      # Command implementations
│   ├── parser.rs        # Pest parser + tests
│   ├── projectmd.pest   # Parser grammar
│   ├── types.rs         # Data structures
│   ├── sync.rs          # Sync engine
│   └── backend/
│       ├── mod.rs       # Backend trait
│       └── github.rs    # GitHub backend
├── Cargo.toml
└── README.md
```

### Building

```bash
# Development build
cargo build

# Release build
cargo build --release

# Run tests
cargo test

# Run with logging
RUST_LOG=debug cargo run -- sync
```

### Running Tests

```bash
cargo test

# Run with output
cargo test -- --nocapture

# Run specific test
cargo test test_parse_project_file
```

## Architecture

### Parser

Uses [Pest](https://pest.rs/) for parsing the projectmd format. The grammar in `src/projectmd.pest` defines:
- YAML front matter extraction
- Task list parsing with status markers
- Flexible content that ignores non-task lines

### Backend Trait

Extensible backend system via the `Backend` trait:

```rust
#[async_trait]
pub trait Backend: Send + Sync {
    async fn create_issue(&self, title: &str, body: &str, labels: Vec<String>) -> Result<Issue>;
    async fn update_issue(&self, number: u64, title: &str, body: &str, labels: Vec<String>) -> Result<Issue>;
    async fn get_issue(&self, number: u64) -> Result<Issue>;
    async fn list_issues(&self) -> Result<Vec<Issue>>;
}
```

Easy to add new backends like GitLab, Jira, Linear, etc.

### Sync Engine

The sync engine handles:
1. Reading and parsing project and task files
2. Determining which tasks need to be created vs updated
3. Calling backend methods to sync changes
4. Updating task files with new issue IDs

## Roadmap

- [ ] Add more subcommands:
  - `commits` - Link commits to tasks
  - `close` - Close tasks/issues
  - `search` - Search across tasks
- [ ] Support more backends:
  - GitLab
  - Jira
  - Linear
  - Azure DevOps
- [ ] Project templates
- [ ] Bulk operations
- [ ] Task dependencies and relationships
- [ ] Time tracking
- [ ] Milestone support
- [ ] Configuration file support

## Contributing

Contributions welcome! Please:

1. Fork the repository
2. Create a feature branch
3. Make your changes with tests
4. Submit a pull request

## License

MIT License - feel free to use this however you'd like!

## Why ProjectMD?

As developers who work with AI coding assistants, we wanted a project management system that:

1. **Lives in plain text** - Easy to version control, diff, and merge
2. **Is LLM-readable** - AI assistants can easily parse and understand your project structure
3. **Integrates with existing tools** - Syncs with GitHub Issues, not replacing but enhancing
4. **Stays out of the way** - No databases, no servers, just files
5. **Works from the CLI** - Because that's where we live

Perfect for solo developers and small teams who prefer text files over web interfaces.
