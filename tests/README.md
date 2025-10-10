# Parser Test Fixtures

This directory contains test fixtures for the projectmd parser.

## Fixtures

### simple.md
Basic project with two tasks, no trailing newlines or extra content.

### trailing_newline.md
Project with a single trailing newline at the end. Tests that the parser handles files that end with `\n`.

### multiple_newlines.md
Project with multiple blank lines throughout. Tests that empty lines don't break parsing.

### complex.md
Complex project with:
- Extra YAML front matter fields
- Multiple markdown sections (headings, paragraphs)
- Tasks scattered throughout the content
- Regular bullet points that aren't tasks
- Various issue numbers including large ones (#42)

### no_tasks.md
Project with no task items at all, just regular markdown content. Verifies parser handles empty task lists.

### mixed_content.md
Project with tasks interspersed with regular markdown paragraphs. Tests that the parser correctly identifies tasks among other content and handles trailing newlines.

## Running Tests

Run all parser tests:
```bash
cargo test
```

Run only fixture tests:
```bash
cargo test --test parser_tests
```

Run a specific test:
```bash
cargo test test_complex_project
```

## Adding New Fixtures

To add a new test fixture:

1. Create a new `.md` file in `tests/fixtures/`
2. Add a test function in `tests/parser_tests.rs`
3. Use the `load_fixture()` helper to load the file
4. Make assertions about the parsed result

Example:
```rust
#[test]
fn test_my_new_fixture() {
    let content = load_fixture("my_new_fixture.md");
    let result = parse_project_file(&content).expect("Failed to parse");

    assert_eq!(result.config.backend, "github");
    assert_eq!(result.tasks.len(), 3);
    // ... more assertions
}
```

## Grammar Coverage

The fixtures test the following grammar edge cases:

- [x] Simple task lists
- [x] Trailing newlines (single and multiple)
- [x] Empty lines between content
- [x] Multiple blank lines in succession
- [x] Tasks mixed with regular markdown
- [x] Projects with no tasks
- [x] Extra YAML fields
- [x] Various issue number formats (#1, #42, etc.)
- [x] Regular bullet points that aren't tasks
- [x] Multiple markdown sections and headings

## Parser Grammar

The parser uses Pest grammar defined in `src/projectmd.pest`:

- **document**: SOI ~ frontmatter ~ content ~ EOI
- **frontmatter**: YAML content followed by `---` separator
- **content**: List of lines (task_item or text_line)
- **task_item**: `* [#123]` or `* [new]` followed by ` - path - description`
- **text_line**: Any line that's not a task item

Key features:
- Task items must start with `* [`
- The `\n` at the end of task_item is optional to handle EOF
- text_line is atomic (@) to avoid backtracking issues
- Empty lines are handled as text_lines with just `\n`
