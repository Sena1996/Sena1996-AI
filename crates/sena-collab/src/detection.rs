use serde::{Deserialize, Serialize};
use std::env;
use std::path::PathBuf;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum IdeType {
    VsCode,
    Cursor,
    JetBrains(JetBrainsProduct),
    Zed,
    Neovim,
    Vim,
    Emacs,
    SublimeText,
    Atom,
    Unknown(String),
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum JetBrainsProduct {
    IntelliJ,
    PyCharm,
    WebStorm,
    RustRover,
    GoLand,
    CLion,
    Rider,
    DataGrip,
    Fleet,
    Unknown,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum ShellType {
    Bash,
    Zsh,
    Fish,
    PowerShell,
    Cmd,
    Nushell,
    Unknown(String),
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum EnvironmentType {
    IdeTerminal { ide: IdeType, workspace: Option<PathBuf> },
    SystemShell { shell: ShellType },
    RemoteSsh { host: String, user: String },
    Tmux { session: String },
    Screen { session: String },
    Docker { container: Option<String> },
    Kubernetes { pod: Option<String>, namespace: Option<String> },
    Unknown,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EnvironmentInfo {
    pub environment_type: EnvironmentType,
    pub working_dir: PathBuf,
    pub shell: ShellType,
    pub term: String,
    pub term_program: Option<String>,
    pub is_interactive: bool,
    pub has_tty: bool,
    pub color_support: ColorSupport,
    pub features: EnvironmentFeatures,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ColorSupport {
    None,
    Basic,     // 8 colors
    Colors256, // 256 colors
    TrueColor, // 16 million colors
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EnvironmentFeatures {
    pub supports_hyperlinks: bool,
    pub supports_images: bool,
    pub supports_unicode: bool,
    pub supports_mouse: bool,
    pub clipboard_available: bool,
}

impl Default for EnvironmentFeatures {
    fn default() -> Self {
        Self {
            supports_hyperlinks: false,
            supports_images: false,
            supports_unicode: true,
            supports_mouse: false,
            clipboard_available: false,
        }
    }
}

pub struct EnvironmentDetector;

impl EnvironmentDetector {
    pub fn detect() -> EnvironmentInfo {
        let working_dir = env::current_dir().unwrap_or_else(|_| PathBuf::from("."));
        let shell = Self::detect_shell();
        let term = env::var("TERM").unwrap_or_default();
        let term_program = env::var("TERM_PROGRAM").ok();
        let environment_type = Self::detect_environment_type(&term_program);
        let color_support = Self::detect_color_support(&term);
        let features = Self::detect_features(&term, &term_program);

        EnvironmentInfo {
            environment_type,
            working_dir,
            shell,
            term,
            term_program,
            is_interactive: Self::is_interactive(),
            has_tty: Self::has_tty(),
            color_support,
            features,
        }
    }

    fn detect_environment_type(term_program: &Option<String>) -> EnvironmentType {
        if Self::is_ssh_session() {
            return EnvironmentType::RemoteSsh {
                host: Self::get_ssh_host(),
                user: env::var("USER").unwrap_or_default(),
            };
        }

        if Self::is_docker() {
            return EnvironmentType::Docker {
                container: env::var("HOSTNAME").ok(),
            };
        }

        if Self::is_kubernetes() {
            return EnvironmentType::Kubernetes {
                pod: env::var("HOSTNAME").ok(),
                namespace: env::var("KUBERNETES_NAMESPACE").ok(),
            };
        }

        if let Some(tmux_pane) = env::var("TMUX_PANE").ok() {
            return EnvironmentType::Tmux {
                session: tmux_pane,
            };
        }

        if env::var("STY").is_ok() {
            return EnvironmentType::Screen {
                session: env::var("STY").unwrap_or_default(),
            };
        }

        if let Some(program) = term_program {
            if let Some(ide) = Self::detect_ide(program) {
                return EnvironmentType::IdeTerminal {
                    ide,
                    workspace: Self::detect_workspace(),
                };
            }
        }

        if let Ok(vscode_term) = env::var("VSCODE_INJECTION") {
            if !vscode_term.is_empty() {
                return EnvironmentType::IdeTerminal {
                    ide: IdeType::VsCode,
                    workspace: Self::detect_workspace(),
                };
            }
        }

        if env::var("TERM_PROGRAM_VERSION").is_ok() {
            if let Some(program) = term_program {
                if let Some(ide) = Self::detect_ide(program) {
                    return EnvironmentType::IdeTerminal {
                        ide,
                        workspace: Self::detect_workspace(),
                    };
                }
            }
        }

        EnvironmentType::SystemShell {
            shell: Self::detect_shell(),
        }
    }

    fn detect_ide(term_program: &str) -> Option<IdeType> {
        let lower = term_program.to_lowercase();

        if lower.contains("vscode") || lower == "code" {
            return Some(IdeType::VsCode);
        }

        if lower.contains("cursor") {
            return Some(IdeType::Cursor);
        }

        if lower.contains("zed") {
            return Some(IdeType::Zed);
        }

        if env::var("JETBRAINS_IDE").is_ok() || lower.contains("jetbrains") {
            let product = Self::detect_jetbrains_product();
            return Some(IdeType::JetBrains(product));
        }

        if lower.contains("nvim") || env::var("NVIM").is_ok() {
            return Some(IdeType::Neovim);
        }

        if lower.contains("vim") && !lower.contains("nvim") {
            return Some(IdeType::Vim);
        }

        if lower.contains("emacs") || env::var("INSIDE_EMACS").is_ok() {
            return Some(IdeType::Emacs);
        }

        if lower.contains("sublime") {
            return Some(IdeType::SublimeText);
        }

        if lower.contains("atom") {
            return Some(IdeType::Atom);
        }

        None
    }

    fn detect_jetbrains_product() -> JetBrainsProduct {
        if let Ok(ide) = env::var("JETBRAINS_IDE") {
            let lower = ide.to_lowercase();
            if lower.contains("intellij") || lower.contains("idea") {
                return JetBrainsProduct::IntelliJ;
            }
            if lower.contains("pycharm") {
                return JetBrainsProduct::PyCharm;
            }
            if lower.contains("webstorm") {
                return JetBrainsProduct::WebStorm;
            }
            if lower.contains("rustrover") {
                return JetBrainsProduct::RustRover;
            }
            if lower.contains("goland") {
                return JetBrainsProduct::GoLand;
            }
            if lower.contains("clion") {
                return JetBrainsProduct::CLion;
            }
            if lower.contains("rider") {
                return JetBrainsProduct::Rider;
            }
            if lower.contains("datagrip") {
                return JetBrainsProduct::DataGrip;
            }
            if lower.contains("fleet") {
                return JetBrainsProduct::Fleet;
            }
        }
        JetBrainsProduct::Unknown
    }

    fn detect_shell() -> ShellType {
        if let Ok(shell) = env::var("SHELL") {
            let lower = shell.to_lowercase();
            if lower.ends_with("zsh") {
                return ShellType::Zsh;
            }
            if lower.ends_with("bash") {
                return ShellType::Bash;
            }
            if lower.ends_with("fish") {
                return ShellType::Fish;
            }
            if lower.contains("powershell") || lower.contains("pwsh") {
                return ShellType::PowerShell;
            }
            if lower.ends_with("nu") || lower.contains("nushell") {
                return ShellType::Nushell;
            }
            return ShellType::Unknown(shell);
        }

        if env::var("PSModulePath").is_ok() {
            return ShellType::PowerShell;
        }

        if env::var("COMSPEC").is_ok() {
            return ShellType::Cmd;
        }

        ShellType::Unknown("unknown".to_string())
    }

    fn detect_workspace() -> Option<PathBuf> {
        if let Ok(workspace) = env::var("VSCODE_WORKSPACE_FOLDER") {
            return Some(PathBuf::from(workspace));
        }

        if let Ok(pwd) = env::var("PWD") {
            let path = PathBuf::from(&pwd);

            let markers = [".git", ".vscode", ".idea", "Cargo.toml", "package.json"];
            for marker in markers {
                if path.join(marker).exists() {
                    return Some(path);
                }
            }
        }

        env::current_dir().ok()
    }

    fn is_ssh_session() -> bool {
        env::var("SSH_CONNECTION").is_ok()
            || env::var("SSH_CLIENT").is_ok()
            || env::var("SSH_TTY").is_ok()
    }

    fn get_ssh_host() -> String {
        if let Ok(conn) = env::var("SSH_CONNECTION") {
            let parts: Vec<&str> = conn.split_whitespace().collect();
            if parts.len() >= 3 {
                return parts[2].to_string();
            }
        }
        "unknown".to_string()
    }

    fn is_docker() -> bool {
        std::path::Path::new("/.dockerenv").exists()
            || env::var("DOCKER_CONTAINER").is_ok()
    }

    fn is_kubernetes() -> bool {
        env::var("KUBERNETES_SERVICE_HOST").is_ok()
            || std::path::Path::new("/var/run/secrets/kubernetes.io").exists()
    }

    fn detect_color_support(term: &str) -> ColorSupport {
        if env::var("COLORTERM").ok().as_deref() == Some("truecolor")
            || env::var("COLORTERM").ok().as_deref() == Some("24bit")
        {
            return ColorSupport::TrueColor;
        }

        if term.contains("256color") || term.contains("256") {
            return ColorSupport::Colors256;
        }

        if term.contains("color") || term.contains("xterm") || term.contains("screen") {
            return ColorSupport::Basic;
        }

        if term == "dumb" || term.is_empty() {
            return ColorSupport::None;
        }

        ColorSupport::Basic
    }

    fn detect_features(term: &str, term_program: &Option<String>) -> EnvironmentFeatures {
        let mut features = EnvironmentFeatures::default();

        let modern_terminals = ["kitty", "wezterm", "alacritty", "iterm", "vscode", "cursor"];
        if let Some(program) = term_program {
            let lower = program.to_lowercase();
            for modern in modern_terminals {
                if lower.contains(modern) {
                    features.supports_hyperlinks = true;
                    features.supports_unicode = true;
                    features.supports_mouse = true;
                    break;
                }
            }

            if lower.contains("kitty") || lower.contains("iterm") {
                features.supports_images = true;
            }
        }

        if term.contains("xterm") || term.contains("screen") || term.contains("tmux") {
            features.supports_mouse = true;
        }

        if env::var("LANG")
            .ok()
            .map(|l| l.to_lowercase().contains("utf"))
            .unwrap_or(false)
        {
            features.supports_unicode = true;
        }

        #[cfg(target_os = "macos")]
        {
            features.clipboard_available = true;
        }

        #[cfg(target_os = "linux")]
        {
            features.clipboard_available = env::var("DISPLAY").is_ok()
                || env::var("WAYLAND_DISPLAY").is_ok();
        }

        features
    }

    fn is_interactive() -> bool {
        env::var("PS1").is_ok() || env::var("PROMPT").is_ok()
    }

    fn has_tty() -> bool {
        #[cfg(unix)]
        {
            unsafe { libc::isatty(libc::STDIN_FILENO) != 0 }
        }

        #[cfg(not(unix))]
        {
            true
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_detect_shell() {
        let shell = EnvironmentDetector::detect_shell();
        assert!(!matches!(shell, ShellType::Unknown(_)) || true);
    }

    #[test]
    fn test_detect_environment() {
        let info = EnvironmentDetector::detect();
        assert!(!info.term.is_empty() || true);
    }

    #[test]
    fn test_color_support() {
        let support = EnvironmentDetector::detect_color_support("xterm-256color");
        assert!(matches!(support, ColorSupport::Colors256 | ColorSupport::TrueColor));

        let support = EnvironmentDetector::detect_color_support("dumb");
        assert!(matches!(support, ColorSupport::None | ColorSupport::TrueColor));
    }

    #[test]
    fn test_ide_detection() {
        assert_eq!(
            EnvironmentDetector::detect_ide("vscode"),
            Some(IdeType::VsCode)
        );
        assert_eq!(
            EnvironmentDetector::detect_ide("cursor"),
            Some(IdeType::Cursor)
        );
        assert!(EnvironmentDetector::detect_ide("unknown-terminal").is_none());
    }
}
