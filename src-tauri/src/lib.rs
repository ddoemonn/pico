mod runtime;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .invoke_handler(tauri::generate_handler![
            runtime::detect_llama_server,
            runtime::install_llama_cpp,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
