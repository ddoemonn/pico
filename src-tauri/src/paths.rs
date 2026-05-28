use std::path::PathBuf;

pub fn models_dir() -> PathBuf {
    let base = dirs::cache_dir().unwrap_or_else(|| PathBuf::from("."));
    base.join("pico").join("models")
}

pub fn model_file_path(repo: &str, file: &str) -> PathBuf {
    let safe_repo = repo.replace('/', "__");
    models_dir().join(safe_repo).join(file)
}
