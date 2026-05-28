mod chat;
mod hf;
mod paths;
mod runtime;

use runtime::InferenceState;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .manage(InferenceState::default())
        .invoke_handler(tauri::generate_handler![
            runtime::detect_llama_server,
            runtime::install_llama_cpp,
            runtime::start_inference,
            runtime::stop_inference,
            runtime::current_port,
            hf::search_hf_models,
            hf::list_hf_files,
            hf::download_model,
            hf::list_local_models,
            chat::chat_stream,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
