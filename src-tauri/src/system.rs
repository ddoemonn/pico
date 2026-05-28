use serde::Serialize;
use std::process::Command;

#[derive(Serialize, Clone)]
pub struct SystemInfo {
    pub ram_gb: u64,
    pub cpu: String,
    pub os: String,
}

#[tauri::command]
pub fn system_info() -> SystemInfo {
    SystemInfo {
        ram_gb: total_ram_gb(),
        cpu: cpu_brand(),
        os: std::env::consts::OS.to_string(),
    }
}

fn total_ram_gb() -> u64 {
    if cfg!(target_os = "macos") {
        let out = Command::new("sysctl")
            .args(["-n", "hw.memsize"])
            .output()
            .ok();
        if let Some(o) = out {
            if let Ok(s) = String::from_utf8(o.stdout) {
                if let Ok(bytes) = s.trim().parse::<u64>() {
                    return bytes / 1024 / 1024 / 1024;
                }
            }
        }
    }
    16
}

fn cpu_brand() -> String {
    if cfg!(target_os = "macos") {
        let out = Command::new("sysctl")
            .args(["-n", "machdep.cpu.brand_string"])
            .output()
            .ok();
        if let Some(o) = out {
            if let Ok(s) = String::from_utf8(o.stdout) {
                return s.trim().to_string();
            }
        }
    }
    "unknown".into()
}
