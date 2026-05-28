use crate::paths;
use futures_util::StreamExt;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};
use tauri::{AppHandle, Emitter, Manager, State};
use tokio::fs;
use tokio::io::AsyncWriteExt;

#[derive(Default)]
pub struct DownloadState {
    cancels: Mutex<HashMap<String, Arc<AtomicBool>>>,
}

fn dl_key(repo: &str, file: &str) -> String {
    format!("{repo}/{file}")
}

const HF_BASE: &str = "https://huggingface.co";

#[derive(Deserialize, Serialize, Clone)]
pub struct HfModel {
    pub id: String,
    #[serde(default)]
    pub downloads: u64,
    #[serde(default)]
    pub likes: u64,
    #[serde(default)]
    pub tags: Vec<String>,
    #[serde(default, rename = "lastModified")]
    pub last_modified: Option<String>,
    #[serde(default, rename = "pipeline_tag")]
    pub pipeline_tag: Option<String>,
}

#[derive(Deserialize, Serialize, Clone)]
pub struct HfFile {
    pub path: String,
    #[serde(default)]
    pub size: u64,
}

#[derive(Deserialize)]
struct HfTreeEntry {
    #[serde(rename = "type")]
    kind: String,
    path: String,
    #[serde(default)]
    size: u64,
}

#[derive(Serialize, Clone)]
pub struct DownloadProgress {
    pub repo: String,
    pub file: String,
    pub downloaded: u64,
    pub total: u64,
}

#[derive(Serialize, Clone)]
pub struct LocalModel {
    pub repo: String,
    pub file: String,
    pub path: PathBuf,
    pub size: u64,
}

const REJECT_MODEL_TAGS: &[&str] = &[
    "text-to-image",
    "image-to-image",
    "stable-diffusion",
    "stable-diffusion-xl",
    "flux",
    "diffusion",
    "text-to-video",
    "image-to-video",
    "controlnet",
    "lora",
    "comfyui",
    "automatic-image-captioning",
    "feature-extraction",
    "sentence-similarity",
    "image-classification",
    "object-detection",
];

#[tauri::command]
pub async fn search_hf_models(
    query: String,
    sort: Option<String>,
    tags: Vec<String>,
) -> Result<Vec<HfModel>, String> {
    let sort = sort.unwrap_or_else(|| "trendingScore".to_string());
    let mut params = vec![
        ("filter".to_string(), "gguf".to_string()),
        ("pipeline_tag".to_string(), "text-generation".to_string()),
        ("sort".to_string(), sort),
        ("direction".to_string(), "-1".to_string()),
        ("limit".to_string(), "60".to_string()),
        ("full".to_string(), "true".to_string()),
    ];
    if !query.trim().is_empty() {
        params.push(("search".to_string(), query));
    }
    for t in tags {
        params.push(("filter".to_string(), t));
    }
    let qs: Vec<String> = params
        .into_iter()
        .map(|(k, v)| format!("{}={}", k, urlencoding(&v)))
        .collect();
    let url = format!("{HF_BASE}/api/models?{}", qs.join("&"));

    let resp = reqwest::get(&url).await.map_err(|e| e.to_string())?;
    if !resp.status().is_success() {
        return Err(format!("hf status {}", resp.status()));
    }
    let models: Vec<HfModel> = resp.json().await.map_err(|e| e.to_string())?;
    Ok(models
        .into_iter()
        .filter(|m| {
            let lower_tags: Vec<String> =
                m.tags.iter().map(|t| t.to_lowercase()).collect();
            !lower_tags
                .iter()
                .any(|t| REJECT_MODEL_TAGS.iter().any(|r| t == r))
        })
        .take(40)
        .collect())
}

fn is_llm_file(name: &str) -> bool {
    let lower = name.to_lowercase();
    let base = lower.rsplit('/').next().unwrap_or(&lower);

    if !base.ends_with(".gguf") {
        return false;
    }
    const REJECT_PREFIXES: &[&str] = &[
        "mmproj",
        "tokenizer",
        "vae",
        "ae",
        "clip_",
        "clip-",
        "t5xxl",
        "unet",
        "controlnet",
        "flux1",
        "flux-",
    ];
    if REJECT_PREFIXES.iter().any(|p| base.starts_with(p)) {
        return false;
    }
    const REJECT_SUBSTR: &[&str] = &[
        ".mmproj",
        "embed",
        "rerank",
        "draft",
        "stable-diffusion",
        "sdxl",
        "diffusion",
        "wan2",
        "hunyuan-video",
    ];
    if REJECT_SUBSTR.iter().any(|p| base.contains(p)) {
        return false;
    }
    true
}

#[tauri::command]
pub async fn list_hf_files(repo: String) -> Result<Vec<HfFile>, String> {
    let url = format!("{HF_BASE}/api/models/{repo}/tree/main?recursive=true");
    let resp = reqwest::get(&url).await.map_err(|e| e.to_string())?;
    if !resp.status().is_success() {
        return Err(format!("hf status {}", resp.status()));
    }
    let entries: Vec<HfTreeEntry> = resp.json().await.map_err(|e| e.to_string())?;
    Ok(entries
        .into_iter()
        .filter(|e| e.kind == "file" && is_llm_file(&e.path))
        .map(|e| HfFile {
            path: e.path,
            size: e.size,
        })
        .collect())
}

#[tauri::command]
pub async fn download_model(
    app: AppHandle,
    repo: String,
    file: String,
) -> Result<PathBuf, String> {
    let dest = paths::model_file_path(&repo, &file);
    if let Some(parent) = dest.parent() {
        fs::create_dir_all(parent).await.map_err(|e| e.to_string())?;
    }

    let key = dl_key(&repo, &file);
    let cancel = Arc::new(AtomicBool::new(false));
    {
        let state: State<'_, DownloadState> = app.state();
        state.cancels.lock().unwrap().insert(key.clone(), cancel.clone());
    }

    let result = stream_to_disk(&app, &repo, &file, &dest, cancel).await;

    {
        let state: State<'_, DownloadState> = app.state();
        state.cancels.lock().unwrap().remove(&key);
    }

    result
}

async fn stream_to_disk(
    app: &AppHandle,
    repo: &str,
    file: &str,
    dest: &PathBuf,
    cancel: Arc<AtomicBool>,
) -> Result<PathBuf, String> {
    let url = format!("{HF_BASE}/{repo}/resolve/main/{file}");
    let resp = reqwest::get(&url).await.map_err(|e| e.to_string())?;
    if !resp.status().is_success() {
        return Err(format!("hf status {}", resp.status()));
    }
    let total = resp.content_length().unwrap_or(0);

    let tmp = dest.with_extension("gguf.part");
    let mut writer = fs::File::create(&tmp).await.map_err(|e| e.to_string())?;
    let mut stream = resp.bytes_stream();
    let mut downloaded: u64 = 0;
    let mut last_emit = 0u64;

    while let Some(chunk) = stream.next().await {
        if cancel.load(Ordering::Relaxed) {
            drop(writer);
            let _ = fs::remove_file(&tmp).await;
            return Err("cancelled".into());
        }
        let bytes = chunk.map_err(|e| e.to_string())?;
        writer.write_all(&bytes).await.map_err(|e| e.to_string())?;
        downloaded += bytes.len() as u64;

        if downloaded - last_emit > 1_048_576 || downloaded == total {
            let _ = app.emit(
                "download:progress",
                DownloadProgress {
                    repo: repo.to_string(),
                    file: file.to_string(),
                    downloaded,
                    total,
                },
            );
            last_emit = downloaded;
        }
    }

    writer.flush().await.map_err(|e| e.to_string())?;
    drop(writer);
    fs::rename(&tmp, dest).await.map_err(|e| e.to_string())?;

    Ok(dest.clone())
}

#[tauri::command]
pub fn cancel_download(
    state: State<'_, DownloadState>,
    repo: String,
    file: String,
) -> Result<(), String> {
    let key = dl_key(&repo, &file);
    if let Some(flag) = state.cancels.lock().unwrap().get(&key) {
        flag.store(true, Ordering::Relaxed);
        Ok(())
    } else {
        Err("no such download".into())
    }
}

#[tauri::command]
pub fn active_downloads(state: State<'_, DownloadState>) -> Vec<String> {
    state.cancels.lock().unwrap().keys().cloned().collect()
}

#[tauri::command]
pub async fn list_local_models() -> Result<Vec<LocalModel>, String> {
    let root = paths::models_dir();
    if !root.exists() {
        return Ok(vec![]);
    }
    let mut out = vec![];
    let mut repos = fs::read_dir(&root).await.map_err(|e| e.to_string())?;
    while let Some(repo_entry) = repos.next_entry().await.map_err(|e| e.to_string())? {
        let ft = repo_entry.file_type().await.map_err(|e| e.to_string())?;
        if !ft.is_dir() {
            continue;
        }
        let repo_name = repo_entry.file_name().to_string_lossy().replace("__", "/");
        walk(&repo_entry.path(), repo_entry.path().clone(), &repo_name, &mut out).await?;
    }
    Ok(out)
}

async fn walk(
    dir: &std::path::Path,
    root: PathBuf,
    repo_name: &str,
    out: &mut Vec<LocalModel>,
) -> Result<(), String> {
    let mut stack = vec![dir.to_path_buf()];
    while let Some(d) = stack.pop() {
        let mut entries = fs::read_dir(&d).await.map_err(|e| e.to_string())?;
        while let Some(entry) = entries.next_entry().await.map_err(|e| e.to_string())? {
            let p = entry.path();
            let ft = entry.file_type().await.map_err(|e| e.to_string())?;
            if ft.is_dir() {
                stack.push(p);
                continue;
            }
            if p.extension().and_then(|s| s.to_str()) != Some("gguf") {
                continue;
            }
            let rel = p
                .strip_prefix(&root)
                .map(|r| r.to_string_lossy().into_owned())
                .unwrap_or_else(|_| p.file_name().unwrap().to_string_lossy().into_owned());
            let meta = entry.metadata().await.map_err(|e| e.to_string())?;
            out.push(LocalModel {
                repo: repo_name.to_string(),
                file: rel,
                path: p,
                size: meta.len(),
            });
        }
    }
    Ok(())
}

fn urlencoding(s: &str) -> String {
    s.bytes()
        .map(|b| match b {
            b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9' | b'-' | b'_' | b'.' | b'~' => {
                (b as char).to_string()
            }
            _ => format!("%{:02X}", b),
        })
        .collect()
}
