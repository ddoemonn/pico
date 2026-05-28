use serde::Serialize;
use std::net::TcpListener;
use std::path::{Path, PathBuf};
use std::process::{Child, Command, Stdio};
use std::sync::Mutex;
use std::time::{Duration, Instant};
use tauri::{AppHandle, Emitter, State};

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

#[derive(Default)]
pub struct InferenceState {
    pub child: Mutex<Option<Child>>,
    pub port: Mutex<Option<u16>>,
}

fn pick_free_port() -> Result<u16, String> {
    let listener = TcpListener::bind("127.0.0.1:0").map_err(|e| e.to_string())?;
    let port = listener.local_addr().map_err(|e| e.to_string())?.port();
    drop(listener);
    Ok(port)
}

async fn wait_for_ready(port: u16) -> Result<(), String> {
    let url = format!("http://127.0.0.1:{port}/health");
    let deadline = Instant::now() + Duration::from_secs(300);
    let client = reqwest::Client::new();
    while Instant::now() < deadline {
        if let Ok(resp) = client.get(&url).send().await {
            if resp.status().is_success() {
                return Ok(());
            }
        }
        tokio::time::sleep(Duration::from_millis(250)).await;
    }
    Err("llama-server did not become ready in 5 minutes".into())
}

fn kill_running(state: &InferenceState) {
    let mut guard = state.child.lock().unwrap();
    if let Some(mut child) = guard.take() {
        let _ = child.kill();
        let _ = child.wait();
    }
    *state.port.lock().unwrap() = None;
}

#[tauri::command]
pub async fn start_inference(
    app: AppHandle,
    state: State<'_, InferenceState>,
    model_path: PathBuf,
    ctx_size: u32,
) -> Result<u16, String> {
    let fname = model_path
        .file_name()
        .and_then(|s| s.to_str())
        .unwrap_or("")
        .to_lowercase();
    if fname.starts_with("mmproj") {
        return Err(
            "This is a vision projector (mmproj) file, not the model itself. \
             Download the main .gguf weight file from the same repo."
                .into(),
        );
    }

    kill_running(&state);

    let server = which::which("llama-server")
        .ok()
        .or_else(|| {
            CANDIDATE_PATHS
                .iter()
                .map(PathBuf::from)
                .find(|p| p.exists())
        })
        .ok_or_else(|| "llama-server not found".to_string())?;

    let port = pick_free_port()?;
    let child = spawn_server(&app, &server, &model_path, ctx_size, port)?;

    *state.child.lock().unwrap() = Some(child);
    *state.port.lock().unwrap() = Some(port);

    wait_for_ready(port).await?;
    let _ = app.emit("load:ready", port);
    Ok(port)
}

fn spawn_server(
    app: &AppHandle,
    server: &Path,
    model: &Path,
    ctx: u32,
    port: u16,
) -> Result<Child, String> {
    use std::io::{BufRead, BufReader};

    let mut child = Command::new(server)
        .args([
            "-m",
            &model.to_string_lossy(),
            "-c",
            &ctx.to_string(),
            "--port",
            &port.to_string(),
            "--host",
            "127.0.0.1",
            "-ngl",
            "999",
        ])
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .map_err(|e| format!("failed to spawn llama-server: {e}"))?;

    if let Some(out) = child.stdout.take() {
        let app = app.clone();
        std::thread::spawn(move || {
            for line in BufReader::new(out).lines().map_while(Result::ok) {
                emit_load_line(&app, &line);
            }
        });
    }
    if let Some(err) = child.stderr.take() {
        let app = app.clone();
        std::thread::spawn(move || {
            for line in BufReader::new(err).lines().map_while(Result::ok) {
                emit_load_line(&app, &line);
            }
        });
    }

    Ok(child)
}

#[derive(serde::Serialize, Clone)]
struct LoadLine {
    line: String,
    percent: Option<u32>,
}

fn emit_load_line(app: &AppHandle, raw: &str) {
    let line = raw.trim().to_string();
    if line.is_empty() {
        return;
    }
    let percent = parse_percent(&line);
    let _ = app.emit("load:line", LoadLine { line, percent });
}

fn parse_percent(s: &str) -> Option<u32> {
    let mut chars = s.char_indices().peekable();
    while let Some((i, c)) = chars.next() {
        if c.is_ascii_digit() {
            let start = i;
            let mut end = i + 1;
            while let Some(&(j, nc)) = chars.peek() {
                if nc.is_ascii_digit() {
                    end = j + 1;
                    chars.next();
                } else {
                    break;
                }
            }
            if s[end..].starts_with('%') {
                if let Ok(n) = s[start..end].parse::<u32>() {
                    if n <= 100 {
                        return Some(n);
                    }
                }
            }
        }
    }
    None
}

#[tauri::command]
pub fn stop_inference(state: State<'_, InferenceState>) -> Result<(), String> {
    kill_running(&state);
    Ok(())
}

#[tauri::command]
pub fn current_port(state: State<'_, InferenceState>) -> Option<u16> {
    *state.port.lock().unwrap()
}

