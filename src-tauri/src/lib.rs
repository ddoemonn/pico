mod hf;
mod paths;
mod runtime;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .invoke_handler(tauri::generate_handler![
            runtime::detect_llama_server,
            runtime::install_llama_cpp,
            hf::search_hf_models,
            hf::list_hf_files,
            hf::download_model,
            hf::list_local_models,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
