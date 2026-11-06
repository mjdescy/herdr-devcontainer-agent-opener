use std::env;
use std::fmt::Display;
use std::fs;
use std::os::unix::process::CommandExt;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};

use clap::Parser;
use serde::Deserialize;

/// Start the project's devcontainer and run a coding agent inside it.
///
/// Run from anywhere inside the project.
#[derive(Parser)]
#[command(version, trailing_var_arg = true)]
struct Cli {
    /// Agent TUI command name (defaults to default_agent from the config file)
    agent: Option<String>,

    /// Arguments passed through to the agent
    #[arg(allow_hyphen_values = true)]
    args: Vec<String>,
}

/// Optional settings read from the TOML config file.
#[derive(Deserialize, Default)]
struct Config {
    /// The agent name/TUI command name used when none is given.
    default_agent: Option<String>,
    /// The container CLI passed to devcontainer as `--docker-path`
    /// (a leading `~/` is expanded to `$HOME`).
    docker_path: Option<String>,
}

fn main() {
    let cli = Cli::parse();

    if !command_in_path("devcontainer") {
        die("devcontainer CLI not found in PATH");
    }

    let config_file = config_file_path();
    let config = read_config(&config_file);

    let agent = cli
        .agent
        .filter(|a| !a.is_empty())
        .or(config.default_agent.filter(|a| !a.is_empty()))
        .unwrap_or_else(|| {
            die(format!(
                "no agent specified and no default_agent found in {}",
                config_file.display()
            ))
        });

    let cwd =
        env::current_dir().unwrap_or_else(|e| die(format!("cannot read current directory: {e}")));
    let workspace = find_workspace_folder(&cwd).unwrap_or_else(|| {
        die(format!(
            "no devcontainer found in {} or any parent directory",
            cwd.display()
        ))
    });

    let docker_args = docker_args(config.docker_path.as_deref());

    // The CLI reports errors as JSON on stdout, so show its output on failure.
    let up = Command::new("devcontainer")
        .arg("up")
        .args(&docker_args)
        .arg("--workspace-folder")
        .arg(&workspace)
        .stdin(Stdio::inherit())
        .stderr(Stdio::inherit())
        .output()
        .unwrap_or_else(|e| die(format!("failed to run devcontainer up: {e}")));
    if !up.status.success() {
        eprintln!(
            "{}",
            String::from_utf8_lossy(&up.stdout).trim_end_matches('\n')
        );
        die("devcontainer up failed");
    }

    let err = Command::new("devcontainer")
        .arg("exec")
        .args(&docker_args)
        .arg("--workspace-folder")
        .arg(&workspace)
        .arg(&agent)
        .args(&cli.args)
        .env("HERDR_AGENT", &agent)
        .exec();
    die(format!("failed to run devcontainer exec: {err}"));
}

/// Print an error message to stderr and exit with status 1.
fn die(msg: impl Display) -> ! {
    eprintln!("open-agent-in-devcontainer: {msg}");
    std::process::exit(1);
}

fn command_in_path(name: &str) -> bool {
    env::var_os("PATH")
        .is_some_and(|paths| env::split_paths(&paths).any(|dir| dir.join(name).is_file()))
}

fn home_dir() -> PathBuf {
    env::var_os("HOME").map(PathBuf::from).unwrap_or_default()
}

fn config_file_path() -> PathBuf {
    let base = match env::var_os("XDG_CONFIG_HOME") {
        Some(dir) if !dir.is_empty() => PathBuf::from(dir),
        _ => home_dir().join(".config"),
    };
    base.join("herdr-devcontainer-agent-opener")
        .join("config.toml")
}

/// Read the config file. A missing file yields the defaults.
fn read_config(path: &Path) -> Config {
    match fs::read_to_string(path) {
        Ok(text) => toml::from_str(&text)
            .unwrap_or_else(|e| die(format!("invalid config file {}: {e}", path.display()))),
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => Config::default(),
        Err(e) => die(format!("cannot read config file {}: {e}", path.display())),
    }
}

/// Return the nearest directory, starting at `start` and walking up to `/`,
/// that holds a devcontainer config.
fn find_workspace_folder(start: &Path) -> Option<PathBuf> {
    start
        .ancestors()
        .find(|dir| {
            dir.join(".devcontainer/devcontainer.json").is_file()
                || dir.join(".devcontainer.json").is_file()
        })
        .map(Path::to_path_buf)
}

/// The devcontainer CLI's `--docker-path` option, or nothing when `docker_path`
/// is not set.
fn docker_args(docker_path: Option<&str>) -> Vec<String> {
    match docker_path.filter(|p| !p.is_empty()) {
        Some(path) => vec!["--docker-path".into(), expand_home(path, &home_dir())],
        None => Vec::new(),
    }
}

/// Expand a leading `~/` to the home directory.
fn expand_home(path: &str, home: &Path) -> String {
    match path.strip_prefix("~/") {
        Some(rest) => format!("{}/{rest}", home.display()),
        None => path.to_string(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn temp_dir(name: &str) -> PathBuf {
        let dir = env::temp_dir().join(format!("oaid-{name}-{}", std::process::id()));
        let _ = fs::remove_dir_all(&dir);
        fs::create_dir_all(&dir).unwrap();
        dir
    }

    #[test]
    fn expands_leading_tilde_only() {
        let home = Path::new("/home/u");
        assert_eq!(expand_home("~/bin/podman", home), "/home/u/bin/podman");
        assert_eq!(expand_home("podman", home), "podman");
        assert_eq!(expand_home("/a/~/b", home), "/a/~/b");
    }

    #[test]
    fn parses_quoted_values_and_ignores_tables() {
        let c: Config = toml::from_str(
            "default_agent = 'claude'\ndocker_path = \"podman\"\n[x]\ndefault_agent = \"no\"\n",
        )
        .unwrap();
        assert_eq!(c.default_agent.as_deref(), Some("claude"));
        assert_eq!(c.docker_path.as_deref(), Some("podman"));
    }

    #[test]
    fn missing_config_gives_defaults() {
        let c = read_config(Path::new("/nonexistent/config.toml"));
        assert!(c.default_agent.is_none() && c.docker_path.is_none());
    }

    #[test]
    fn finds_workspace_in_parent() {
        let root = temp_dir("ws");
        fs::create_dir_all(root.join(".devcontainer")).unwrap();
        fs::write(root.join(".devcontainer/devcontainer.json"), "{}").unwrap();
        fs::create_dir_all(root.join("a/b")).unwrap();
        assert_eq!(find_workspace_folder(&root.join("a/b")), Some(root.clone()));
        fs::remove_dir_all(&root).unwrap();
    }

    #[test]
    fn finds_dotfile_config() {
        let root = temp_dir("dot");
        fs::write(root.join(".devcontainer.json"), "{}").unwrap();
        assert_eq!(find_workspace_folder(&root), Some(root.clone()));
        fs::remove_dir_all(&root).unwrap();
    }
}
