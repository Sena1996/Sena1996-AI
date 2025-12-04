use std::collections::HashMap;
use std::process::{Command, Output, Stdio};

use super::error::{GuardianError, GuardianResult};

pub trait InlineExecutable: Send + Sync {
    fn execute_inline(&self, args: &[&str]) -> GuardianResult<Output>;
    fn command_name(&self) -> &'static str;
}

pub struct DirectExecutor {
    inline_commands: HashMap<&'static str, Box<dyn InlineExecutable>>,
}

impl DirectExecutor {
    pub fn new() -> Self {
        let mut executor = Self {
            inline_commands: HashMap::new(),
        };
        executor.register_builtin_commands();
        executor
    }

    fn register_builtin_commands(&mut self) {
        self.inline_commands
            .insert("sena", Box::new(SenaInlineCommand));
        self.inline_commands
            .insert("echo", Box::new(EchoInlineCommand));
        self.inline_commands
            .insert("pwd", Box::new(PwdInlineCommand));
        self.inline_commands
            .insert("whoami", Box::new(WhoamiInlineCommand));
    }

    pub fn register_command(&mut self, command: Box<dyn InlineExecutable>) {
        self.inline_commands.insert(command.command_name(), command);
    }

    pub fn execute(&self, command: &str, args: &[&str]) -> GuardianResult<Output> {
        if let Some(handler) = self.inline_commands.get(command) {
            return handler.execute_inline(args);
        }

        self.execute_direct(command, args)
    }

    fn execute_direct(&self, command: &str, args: &[&str]) -> GuardianResult<Output> {
        Command::new(command)
            .args(args)
            .stdin(Stdio::null())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .output()
            .map_err(|e| {
                if e.kind() == std::io::ErrorKind::NotFound {
                    GuardianError::CommandNotFound(command.to_string())
                } else {
                    GuardianError::IoError(e)
                }
            })
    }

    pub fn has_inline_handler(&self, command: &str) -> bool {
        self.inline_commands.contains_key(command)
    }
}

impl Default for DirectExecutor {
    fn default() -> Self {
        Self::new()
    }
}

struct SenaInlineCommand;

impl InlineExecutable for SenaInlineCommand {
    fn execute_inline(&self, args: &[&str]) -> GuardianResult<Output> {
        let output_str = format!("SENA inline execution: {:?}", args);
        Ok(create_output(output_str.as_bytes(), &[], 0))
    }

    fn command_name(&self) -> &'static str {
        "sena"
    }
}

struct EchoInlineCommand;

impl InlineExecutable for EchoInlineCommand {
    fn execute_inline(&self, args: &[&str]) -> GuardianResult<Output> {
        let output_str = args.join(" ");
        Ok(create_output(output_str.as_bytes(), &[], 0))
    }

    fn command_name(&self) -> &'static str {
        "echo"
    }
}

struct PwdInlineCommand;

impl InlineExecutable for PwdInlineCommand {
    fn execute_inline(&self, _args: &[&str]) -> GuardianResult<Output> {
        let cwd = std::env::current_dir()
            .map(|p| p.display().to_string())
            .unwrap_or_else(|_| "unknown".to_string());
        Ok(create_output(cwd.as_bytes(), &[], 0))
    }

    fn command_name(&self) -> &'static str {
        "pwd"
    }
}

struct WhoamiInlineCommand;

impl InlineExecutable for WhoamiInlineCommand {
    fn execute_inline(&self, _args: &[&str]) -> GuardianResult<Output> {
        let user = std::env::var("USER")
            .or_else(|_| std::env::var("USERNAME"))
            .unwrap_or_else(|_| "unknown".to_string());
        Ok(create_output(user.as_bytes(), &[], 0))
    }

    fn command_name(&self) -> &'static str {
        "whoami"
    }
}

fn create_output(stdout: &[u8], stderr: &[u8], code: i32) -> Output {
    use std::os::unix::process::ExitStatusExt;

    Output {
        status: std::process::ExitStatus::from_raw(code),
        stdout: stdout.to_vec(),
        stderr: stderr.to_vec(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_executor_creation() {
        let executor = DirectExecutor::new();
        assert!(executor.has_inline_handler("sena"));
        assert!(executor.has_inline_handler("echo"));
        assert!(executor.has_inline_handler("pwd"));
    }

    #[test]
    fn test_echo_inline() {
        let executor = DirectExecutor::new();
        let result = executor.execute("echo", &["hello", "world"]).unwrap();
        let output = String::from_utf8_lossy(&result.stdout);
        assert_eq!(output, "hello world");
    }

    #[test]
    fn test_pwd_inline() {
        let executor = DirectExecutor::new();
        let result = executor.execute("pwd", &[]).unwrap();
        let output = String::from_utf8_lossy(&result.stdout);
        assert!(!output.is_empty());
    }

    #[test]
    fn test_direct_execution() {
        let executor = DirectExecutor::new();
        let result = executor.execute("ls", &["-la"]);
        assert!(result.is_ok());
    }
}
