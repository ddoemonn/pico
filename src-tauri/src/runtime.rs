use serde::Serialize;
use std::path::PathBuf;
use std::process::{Command, Stdio};
use tauri::{AppHandle, Emitter};

const CANDIDATE_PATHS: &[&str] = &[
    "/opt/homebrew/bin/llama-server",
    "/usr/local/bin/llama-server",
    "/opt/local/bin/llama-server",
];

#[derive(Serialize, Clone)]
pub struct LlamaServerStatus {
    pub installed: bool,
    pub path: Option<PathBuf>,
    pub version: Option<String>,
}

#[tauri::command]
pub fn detect_llama_server() -> LlamaServerStatus {
    let path = which::which("llama-server")
        .ok()
        .or_else(|| {
            CANDIDATE_PATHS
                .iter()
                .map(PathBuf::from)
                .find(|p| p.exists())
        });

    let version = path.as_ref().and_then(probe_version);

    LlamaServerStatus {
        installed: path.is_some(),
        path,
        version,
    }
}

fn probe_version(path: &PathBuf) -> Option<String> {
    let output = Command::new(path)
        .arg("--version")
        .stderr(Stdio::piped())
        .stdout(Stdio::piped())
        .output()
        .ok()?;

    let combined = format!(
        "{}{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );

    combined
        .lines()
        .find(|line| line.contains("version"))
        .map(|line| line.trim().to_string())
        .or_else(|| combined.lines().next().map(|line| line.trim().to_string()))
}

#[tauri::command]
pub async fn install_llama_cpp(app: AppHandle) -> Result<(), String> {
    use std::io::{BufRead, BufReader};

    let mut child = Command::new("/opt/homebrew/bin/brew")
        .args(["install", "llama.cpp"])
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .map_err(|e| format!("failed to spawn brew: {e}"))?;

    if let Some(stdout) = child.stdout.take() {
        let app = app.clone();
        std::thread::spawn(move || {
            for line in BufReader::new(stdout).lines().map_while(Result::ok) {
                let _ = app.emit("install:stdout", line);
            }
        });
    }

    if let Some(stderr) = child.stderr.take() {
        let app = app.clone();
        std::thread::spawn(move || {
            for line in BufReader::new(stderr).lines().map_while(Result::ok) {
                let _ = app.emit("install:stderr", line);
            }
        });
    }

    let status = child
        .wait()
        .map_err(|e| format!("failed to wait for brew: {e}"))?;

    if !status.success() {
        return Err(format!("brew exited with {status}"));
    }

    Ok(())
}
