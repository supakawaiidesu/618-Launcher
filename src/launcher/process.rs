use std::path::Path;
use std::process::Command;
use thiserror::Error;

/// Launch a game executable
pub fn launch_game(executable_path: &Path, launch_args: Option<&str>) -> Result<(), LaunchError> {
    if !executable_path.exists() {
        return Err(LaunchError::ExecutableNotFound(
            executable_path.to_string_lossy().to_string(),
        ));
    }

    let mut command = Command::new(executable_path);

    // Set working directory to the executable's directory
    if let Some(parent) = executable_path.parent() {
        command.current_dir(parent);
    }

    // Add launch arguments if provided
    if let Some(args) = launch_args {
        // Split arguments by whitespace, respecting quotes
        let args = parse_args(args);
        command.args(&args);
    }

    // Spawn the process
    let child = command.spawn().map_err(|e| LaunchError::SpawnFailed(e.to_string()))?;

    tracing::info!(
        "Launched game: {:?} (PID: {})",
        executable_path,
        child.id()
    );

    Ok(())
}

/// Parse command line arguments, handling quoted strings
fn parse_args(args_str: &str) -> Vec<String> {
    let mut args = Vec::new();
    let mut current = String::new();
    let mut in_quotes = false;
    let mut quote_char = ' ';

    for c in args_str.chars() {
        match c {
            '"' | '\'' if !in_quotes => {
                in_quotes = true;
                quote_char = c;
            }
            c if c == quote_char && in_quotes => {
                in_quotes = false;
            }
            ' ' if !in_quotes => {
                if !current.is_empty() {
                    args.push(current.clone());
                    current.clear();
                }
            }
            _ => {
                current.push(c);
            }
        }
    }

    if !current.is_empty() {
        args.push(current);
    }

    args
}

/// Errors that can occur when launching a game
#[derive(Debug, Error)]
pub enum LaunchError {
    #[error("Executable not found: {0}")]
    ExecutableNotFound(String),

    #[error("Failed to spawn process: {0}")]
    SpawnFailed(String),

    #[error("Permission denied")]
    PermissionDenied,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_args_simple() {
        let args = parse_args("-windowed -width 1920");
        assert_eq!(args, vec!["-windowed", "-width", "1920"]);
    }

    #[test]
    fn test_parse_args_quoted() {
        let args = parse_args("-path \"C:\\Program Files\\Game\"");
        assert_eq!(args, vec!["-path", "C:\\Program Files\\Game"]);
    }

    #[test]
    fn test_parse_args_empty() {
        let args = parse_args("");
        assert!(args.is_empty());
    }
}
