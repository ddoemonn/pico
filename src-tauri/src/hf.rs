use crate::paths;
use futures_util::StreamExt;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use tauri::{AppHandle, Emitter};
use tokio::fs;
use tokio::io::AsyncWriteExt;

const HF_BASE: &str = "https://huggingface.co";

#[derive(Deserialize, Serialize, Clone)]
pub struct HfModel {
    pub id: String,
    #[serde(default)]
    pub downloads: u64,
    #[serde(default)]
    pub likes: u64,
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

#[tauri::command]
pub async fn search_hf_models(query: String) -> Result<Vec<HfModel>, String> {
    let url = format!(
        "{HF_BASE}/api/models?search={}&filter=gguf&sort=downloads&limit=30",
        urlencoding(&query)
    );
    let resp = reqwest::get(&url).await.map_err(|e| e.to_string())?;
    if !resp.status().is_success() {
        return Err(format!("hf status {}", resp.status()));
    }
    resp.json::<Vec<HfModel>>().await.map_err(|e| e.to_string())
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
        .filter(|e| e.kind == "file" && e.path.to_lowercase().ends_with(".gguf"))
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
        let bytes = chunk.map_err(|e| e.to_string())?;
        writer.write_all(&bytes).await.map_err(|e| e.to_string())?;
        downloaded += bytes.len() as u64;

        if downloaded - last_emit > 1_048_576 || downloaded == total {
            let _ = app.emit(
                "download:progress",
                DownloadProgress {
                    repo: repo.clone(),
                    file: file.clone(),
                    downloaded,
                    total,
                },
            );
            last_emit = downloaded;
        }
    }

    writer.flush().await.map_err(|e| e.to_string())?;
    drop(writer);
    fs::rename(&tmp, &dest).await.map_err(|e| e.to_string())?;

    Ok(dest)
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
        if !repo_entry.file_type().await.map_err(|e| e.to_string())?.is_dir() {
            continue;
        }
        let repo_name = repo_entry.file_name().to_string_lossy().replace("__", "/");
        let mut files = fs::read_dir(repo_entry.path()).await.map_err(|e| e.to_string())?;
        while let Some(f) = files.next_entry().await.map_err(|e| e.to_string())? {
            let path = f.path();
            if path.extension().and_then(|s| s.to_str()) != Some("gguf") {
                continue;
            }
            let meta = f.metadata().await.map_err(|e| e.to_string())?;
            out.push(LocalModel {
                repo: repo_name.clone(),
                file: f.file_name().to_string_lossy().into_owned(),
                path: path.clone(),
                size: meta.len(),
            });
        }
    }
    Ok(out)
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
