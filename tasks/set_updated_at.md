---
issue_id: 2
type: feature
tags:
- optimization
- metadata
- sync
---

# Add updated_at timestamp to optimize sync operations

Add `updated_at` timestamp to task YAML front matter to avoid unnecessary file writes during sync. Only sync tasks whose local file modification time is newer than the stored `updated_at` timestamp.

## Problem

Currently, `projectmd sync` updates task files and GitHub issues on every sync, even when nothing has changed. This causes:
- Unnecessary GitHub API calls (rate limit concerns)
- Redundant file writes (noisy git diffs)
- Slower sync operations
- Confusing modification times on files

## Solution

Add an `updated_at` timestamp to each task's YAML front matter. During sync:

1. Check task file's modification time (`mtime`)
2. Compare with stored `updated_at` timestamp
3. Only sync if `mtime > updated_at` (file was edited since last sync)
4. Update `updated_at` after successful sync

This ensures we only sync tasks that have actually been modified locally.

## Implementation steps

### 1. Add timestamp fields to TaskFileConfig

Update `src/types.rs`:

```rust
use chrono::{DateTime, Utc};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskFileConfig {
    pub issue_id: Option<u64>,
    #[serde(rename = "type")]
    pub task_type: Option<String>,
    pub tags: Option<Vec<String>>,

    // New fields
    pub created_at: Option<String>,   // ISO 8601 timestamp
    pub updated_at: Option<String>,   // ISO 8601 timestamp

    #[serde(flatten)]
    pub extra: std::collections::HashMap<String, serde_yaml::Value>,
}
```

### 2. Add file modification time checking

Create a new utility in `src/sync.rs`:

```rust
use std::fs;
use std::time::SystemTime;
use chrono::{DateTime, Utc};

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
    let updated_at = DateTime::parse_from_rfc3339(updated_at_str)?
        .with_timezone(&Utc);

    // Only sync if file was modified after last sync
    Ok(mtime_utc > updated_at)
}
```

### 3. Update sync logic to check timestamps

Modify `src/sync.rs` `sync_task_item()`:

```rust
async fn sync_task_item(&self, task_item: &TaskItem) -> Result<SyncAction> {
    let task_file_path = self.project_root.join(&task_item.path);
    let task_content = fs::read_to_string(&task_file_path)?;
    let task_file = parse_task_file(&task_content)?;

    // Check if we need to sync this task
    if !should_sync_task(&task_file_path, &task_file.config)? {
        return Ok(SyncAction::Skipped);
    }

    // Rest of sync logic...
    match &task_item.status {
        TaskStatus::New => {
            let issue = self.backend.create_issue(...).await?;

            // Update task file with issue_id and timestamps
            self.update_task_file_with_metadata(
                &task_file_path,
                &task_content,
                issue.number,
                true  // is_new
            )?;

            Ok(SyncAction::Created(issue.number))
        }
        TaskStatus::Existing(issue_num) => {
            let issue = self.backend.update_issue(...).await?;

            // Update only the updated_at timestamp
            self.update_task_file_with_metadata(
                &task_file_path,
                &task_content,
                *issue_num,
                false  // not new
            )?;

            Ok(SyncAction::Updated(issue.number))
        }
    }
}
```

### 4. Update task file metadata

Replace `update_task_file_issue_id()` with a more comprehensive function:

```rust
fn update_task_file_with_metadata(
    &self,
    path: &Path,
    content: &str,
    issue_id: u64,
    is_new: bool
) -> Result<()> {
    let mut task_file = parse_task_file(content)?;
    let mut config = task_file.config;

    // Set issue_id
    config.issue_id = Some(issue_id);

    // Set timestamps
    let now = Utc::now().to_rfc3339();

    if is_new || config.created_at.is_none() {
        config.created_at = Some(now.clone());
    }

    config.updated_at = Some(now);

    // Serialize and write
    let yaml_str = serde_yaml::to_string(&config)?;
    let parts: Vec<&str> = content.splitn(3, "---").collect();
    let updated_content = format!("---\n{}\n---\n{}", yaml_str.trim(), parts[2]);

    fs::write(path, updated_content)?;
    Ok(())
}
```

### 5. Update sync result reporting

Show skipped tasks in the summary:

```rust
impl SyncResult {
    pub fn print_summary(&self) {
        println!("\n=== Sync Summary ===");

        // ... created and updated sections ...

        if !self.skipped.is_empty() {
            println!("\nSkipped (no changes) ({}):", self.skipped.len());
            for path in &self.skipped {
                println!("  ✓ {}", path.display());
            }
        }

        // ... rest of output ...
    }
}
```

### 6. Add chrono dependency

Update `Cargo.toml`:

```toml
[dependencies]
chrono = { version = "0.4", features = ["serde"] }
```

## Expected behavior

### First sync (no timestamps)
```bash
$ projectmd sync

=== Sync Summary ===

Created (1):
  - tasks/new-feature.md -> Issue #5

Updated (2):
  - tasks/setup.md -> Issue #1
  - tasks/deploy.md -> Issue #2

Total: 3 tasks processed
```

### Second sync (no file changes)
```bash
$ projectmd sync

=== Sync Summary ===

Skipped (no changes) (3):
  ✓ tasks/new-feature.md
  ✓ tasks/setup.md
  ✓ tasks/deploy.md

Total: 3 tasks processed
```

### Sync after editing one file
```bash
# Edit tasks/setup.md
$ vim tasks/setup.md

$ projectmd sync

=== Sync Summary ===

Updated (1):
  - tasks/setup.md -> Issue #1

Skipped (no changes) (2):
  ✓ tasks/new-feature.md
  ✓ tasks/deploy.md

Total: 3 tasks processed
```

## Task file example

After sync, task files look like:

```markdown
---
issue_id: 1
type: feature
tags: [auth, security]
created_at: "2025-01-15T10:30:00Z"
updated_at: "2025-01-20T15:45:32Z"
---
# Setup authentication

Implementation details...
```

## Technical considerations

### Timestamp format
- Use ISO 8601 (RFC 3339): `2025-01-15T10:30:00Z`
- Always use UTC to avoid timezone issues
- Format is both human-readable and parsable

### File modification time edge cases
- What if user uses `touch` on a file?
  - It will trigger sync (which is correct - they modified mtime)
- What if git checkout changes mtime?
  - May trigger unnecessary syncs, but better safe than missing updates
- What if system clock changes?
  - Compare timestamps, not wall clock time

### Backward compatibility
- If `updated_at` is missing, treat as "needs sync" (safe default)
- Existing task files without timestamps will get them on first sync
- No breaking changes to file format

### Race conditions
- If two syncs run concurrently, both might update the same task
- File-based locking or atomic writes may be needed for future
- For now, document that concurrent syncs are not supported

### Force sync option
Add `--force` flag to override timestamp checking:

```bash
# Force sync all tasks regardless of timestamps
projectmd sync --force
```

Useful for:
- Fixing desync between local and remote
- Testing
- Recovering from errors

## Acceptance criteria

- [ ] `created_at` and `updated_at` fields added to TaskFileConfig
- [ ] `should_sync_task()` function checks file mtime vs updated_at
- [ ] Sync skips tasks that haven't been modified
- [ ] Timestamps set correctly on create and update
- [ ] SyncAction::Skipped variant is used and reported
- [ ] Skipped tasks shown in sync summary
- [ ] chrono dependency added
- [ ] Tests added for timestamp comparison logic
- [ ] Backward compatibility tested with old task files
- [ ] Documentation updated

## Performance benefits

### Before optimization
- 10 tasks × 2 API calls each = 20 API calls per sync
- All files rewritten on every sync
- Sync time: ~5 seconds (with API latency)

### After optimization
- 10 tasks, 2 modified = 4 API calls
- Only 2 files rewritten
- Sync time: ~1 second
- **80% reduction in API calls and file writes**

## Future enhancements

### Smart conflict detection
If `updated_at` in file is older than GitHub issue's `updated_at`, warn about potential conflict:

```bash
⚠️  tasks/setup.md: Local (Jan 15) older than remote (Jan 20)
   Run 'projectmd pull' to fetch remote changes
```

### Selective sync
```bash
# Only sync specific tasks
projectmd sync tasks/setup.md tasks/deploy.md

# Sync all tasks matching a pattern
projectmd sync 'tasks/auth-*.md'
```

### Dry run improvements
```bash
$ projectmd sync --dry-run

Would sync:
  - tasks/setup.md (modified 5 mins ago)

Would skip:
  - tasks/deploy.md (last synced 2 hours ago)
  - tasks/auth.md (last synced yesterday)
```

## Testing strategy

### Unit tests
```rust
#[test]
fn test_should_sync_task_no_timestamp() {
    // Task without updated_at should always sync
}

#[test]
fn test_should_sync_task_file_newer() {
    // File modified after updated_at should sync
}

#[test]
fn test_should_sync_task_file_older() {
    // File not modified since updated_at should skip
}
```

### Integration tests
- Create task, sync, verify timestamp
- Modify task, sync, verify updated_at changes
- Don't modify task, sync, verify it's skipped
- Force sync, verify all tasks sync regardless

## Resources

- [chrono crate](https://docs.rs/chrono/latest/chrono/)
- [std::fs::Metadata](https://doc.rust-lang.org/std/fs/struct.Metadata.html)
- [RFC 3339 timestamp format](https://www.rfc-editor.org/rfc/rfc3339)

## Notes

- This optimization becomes more valuable as projects grow
- Consider adding metrics to track skip rate
- May want to add `--verbose` to show why tasks were skipped
- Could extend to check GitHub issue `updated_at` for bidirectional sync
