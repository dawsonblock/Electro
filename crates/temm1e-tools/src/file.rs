//! File tool — read, write, and list files within the session workspace.

use async_trait::async_trait;
use temm1e_core::types::error::Temm1eError;
use temm1e_core::{Tool, ToolContext, ToolInput, ToolOutput};
use temm1e_core::policy::{CapabilityPolicy, FileAccessPolicy};


/// Maximum file read size (32 KB — keeps tool output within token budget).
const MAX_READ_SIZE: usize = 32 * 1024;

#[derive(Default)]
pub struct FileReadTool;

impl FileReadTool {
    pub fn new() -> Self {
        Self
    }
}

#[async_trait]
impl Tool for FileReadTool {
    fn name(&self) -> &str {
        "file_read"
    }

    fn description(&self) -> &str {
        "Read the contents of a file inside the session workspace. Absolute paths and home-directory expansion are blocked."
    }

    fn parameters_schema(&self) -> serde_json::Value {
        serde_json::json!({
            "type": "object",
            "properties": {
                "path": {
                    "type": "string",
                    "description": "File path to read, relative to the workspace"
                }
            },
            "required": ["path"]
        })
    }

    fn declarations(&self) -> CapabilityPolicy {
        CapabilityPolicy {
            file_access: vec![FileAccessPolicy::Read(".".into())],
            network_access: temm1e_core::net_policy::NetworkPolicy::Blocked,
            shell_access: temm1e_core::policy::ShellPolicy::Blocked,
browser_access: temm1e_core::policy::BrowserPolicy::Blocked,
        }
    }

    async fn execute(
        &self,
        input: ToolInput,
        ctx: &ToolContext,
    ) -> Result<ToolOutput, Temm1eError> {
        let path_str = input
            .arguments
            .get("path")
            .and_then(|v| v.as_str())
            .ok_or_else(|| Temm1eError::Tool("Missing required parameter: path".into()))?;

        let path = match resolve_path(path_str, &ctx.workspace_path) {
            Ok(path) => path,
            Err(e) => {
                return Ok(ToolOutput {
                    content: e,
                    is_error: true,
                });
            }
        };

        match tokio::fs::read_to_string(&path).await {
            Ok(mut content) => {
                if content.len() > MAX_READ_SIZE {
                    content.truncate(MAX_READ_SIZE);
                    content.push_str("\n... [file truncated]");
                }
                Ok(ToolOutput {
                    content,
                    is_error: false,
                })
            }
            Err(e) => Ok(ToolOutput {
                content: format!("Failed to read file '{}': {}", path_str, e),
                is_error: true,
            }),
        }
    }
}

#[derive(Default)]
pub struct FileWriteTool;

impl FileWriteTool {
    pub fn new() -> Self {
        Self
    }
}

#[async_trait]
impl Tool for FileWriteTool {
    fn name(&self) -> &str {
        "file_write"
    }

    fn description(&self) -> &str {
        "Write content to a file inside the session workspace. Creates parent directories automatically. Absolute paths and home-directory expansion are blocked."
    }

    fn parameters_schema(&self) -> serde_json::Value {
        serde_json::json!({
            "type": "object",
            "properties": {
                "path": {
                    "type": "string",
                    "description": "File path to write, relative to the workspace"
                },
                "content": {
                    "type": "string",
                    "description": "Content to write to the file"
                }
            },
            "required": ["path", "content"]
        })
    }

    fn declarations(&self) -> CapabilityPolicy {
        CapabilityPolicy {
            file_access: vec![FileAccessPolicy::ReadWrite(".".into())],
            network_access: temm1e_core::net_policy::NetworkPolicy::Blocked,
            shell_access: temm1e_core::policy::ShellPolicy::Blocked,
browser_access: temm1e_core::policy::BrowserPolicy::Blocked,
        }
    }

    async fn execute(
        &self,
        input: ToolInput,
        ctx: &ToolContext,
    ) -> Result<ToolOutput, Temm1eError> {
        let path_str = input
            .arguments
            .get("path")
            .and_then(|v| v.as_str())
            .ok_or_else(|| Temm1eError::Tool("Missing required parameter: path".into()))?;

        let content = input
            .arguments
            .get("content")
            .and_then(|v| v.as_str())
            .ok_or_else(|| Temm1eError::Tool("Missing required parameter: content".into()))?;

        let path = match resolve_path(path_str, &ctx.workspace_path) {
            Ok(path) => path,
            Err(e) => {
                return Ok(ToolOutput {
                    content: e,
                    is_error: true,
                });
            }
        };

        if let Some(parent) = path.parent() {
            if let Err(e) = tokio::fs::create_dir_all(parent).await {
                return Ok(ToolOutput {
                    content: format!("Failed to create directories for '{}': {}", path_str, e),
                    is_error: true,
                });
            }
        }

        match tokio::fs::write(&path, content).await {
            Ok(()) => Ok(ToolOutput {
                content: format!("Written {} bytes to '{}'", content.len(), path_str),
                is_error: false,
            }),
            Err(e) => Ok(ToolOutput {
                content: format!("Failed to write file '{}': {}", path_str, e),
                is_error: true,
            }),
        }
    }
}

#[derive(Default)]
pub struct FileListTool;

impl FileListTool {
    pub fn new() -> Self {
        Self
    }
}

#[async_trait]
impl Tool for FileListTool {
    fn name(&self) -> &str {
        "file_list"
    }

    fn description(&self) -> &str {
        "List files and directories at a path inside the session workspace. Absolute paths and home-directory expansion are blocked."
    }

    fn parameters_schema(&self) -> serde_json::Value {
        serde_json::json!({
            "type": "object",
            "properties": {
                "path": {
                    "type": "string",
                    "description": "Directory path to list, relative to the workspace. Defaults to workspace root."
                }
            },
            "required": []
        })
    }

    fn declarations(&self) -> CapabilityPolicy {
        CapabilityPolicy {
            file_access: vec![FileAccessPolicy::Read(".".into())],
            network_access: temm1e_core::net_policy::NetworkPolicy::Blocked,
            shell_access: temm1e_core::policy::ShellPolicy::Blocked,
browser_access: temm1e_core::policy::BrowserPolicy::Blocked,
        }
    }

    async fn execute(
        &self,
        input: ToolInput,
        ctx: &ToolContext,
    ) -> Result<ToolOutput, Temm1eError> {
        let path_str = input
            .arguments
            .get("path")
            .and_then(|v| v.as_str())
            .unwrap_or(".");

        let path = match resolve_path(path_str, &ctx.workspace_path) {
            Ok(path) => path,
            Err(e) => {
                return Ok(ToolOutput {
                    content: e,
                    is_error: true,
                });
            }
        };

        match tokio::fs::read_dir(&path).await {
            Ok(mut entries) => {
                let mut items = Vec::new();
                while let Ok(Some(entry)) = entries.next_entry().await {
                    let name = entry.file_name().to_string_lossy().to_string();
                    let is_dir = entry.file_type().await.map(|t| t.is_dir()).unwrap_or(false);
                    if is_dir {
                        items.push(format!("{}/", name));
                    } else {
                        items.push(name);
                    }
                }
                items.sort();
                if items.is_empty() {
                    Ok(ToolOutput {
                        content: format!("Directory '{}' is empty", path_str),
                        is_error: false,
                    })
                } else {
                    Ok(ToolOutput {
                        content: items.join("\n"),
                        is_error: false,
                    })
                }
            }
            Err(e) => Ok(ToolOutput {
                content: format!("Failed to list directory '{}': {}", path_str, e),
                is_error: true,
            }),
        }
    }
}

/// Resolve a relative workspace path. Returns an error string for anything
/// outside the workspace or attempting home/absolute expansion.
fn resolve_path(
    path_str: &str,
    workspace: &std::path::Path,
) -> Result<std::path::PathBuf, String> {
    let trimmed = path_str.trim();
    if trimmed.is_empty() {
        return Err("Path cannot be empty.".to_string());
    }
    if trimmed == "~" || trimmed.starts_with("~/") || trimmed.starts_with("$HOME") {
        return Err("Home-directory expansion is disabled. Use a path relative to the workspace.".to_string());
    }

    let path = std::path::Path::new(trimmed);
    if path.is_absolute() {
        return Err("Absolute paths are disabled. Use a path relative to the workspace.".to_string());
    }

    let candidate = lexical_normalize(&workspace.join(path));
    let workspace_canonical = workspace
        .canonicalize()
        .unwrap_or_else(|_| workspace.to_path_buf());

    if !candidate.starts_with(&workspace_canonical) {
        return Err(format!(
            "Path '{}' escapes the workspace '{}'.",
            path_str,
            workspace.display()
        ));
    }

    Ok(candidate)
}

fn lexical_normalize(path: &std::path::Path) -> std::path::PathBuf {
    use std::path::Component;

    let mut normalized = std::path::PathBuf::new();
    for component in path.components() {
        match component {
            Component::CurDir => {}
            Component::ParentDir => {
                normalized.pop();
            }
            other => normalized.push(other.as_os_str()),
        }
    }
    normalized
}
