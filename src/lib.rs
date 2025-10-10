pub mod backend;
pub mod parser;
pub mod types;
pub mod sync;

// Re-export commonly used types
pub use types::{ProjectConfig, ProjectMd, TaskFile, TaskItem, TaskStatus};
