use icaruz_core::{validate_nickname, ClientKind, ClientStatus, GameInstall, LauncherStatus};
use std::path::PathBuf;

fn default_clients() -> Vec<ClientStatus> {
    vec![
        ClientStatus {
            kind: ClientKind::Samp037R5,
            installed: false,
            version: "not installed".to_string(),
            path: None,
        },
        ClientStatus {
            kind: ClientKind::Samp03DL,
            installed: false,
            version: "not installed".to_string(),
            path: None,
        },
        ClientStatus {
            kind: ClientKind::OpenMp1x,
            installed: false,
            version: "not installed".to_string(),
            path: None,
        },
    ]
}

#[tauri::command]
fn get_status() -> LauncherStatus {
    LauncherStatus {
        gta: None,
        clients: default_clients(),
        nickname: "Player".to_string(),
        needs_attention: Some("GTA San Andreas folder not set yet.".to_string()),
    }
}

#[tauri::command]
fn detect_gta(path: String) -> Result<GameInstall, String> {
    let p = PathBuf::from(&path);
    let exe = p.join("gta_sa.exe");
    if !exe.exists() {
        return Err("gta_sa.exe not found in that folder.".to_string());
    }
    Ok(GameInstall {
        gta_path: p,
        exe_version: "unknown (hash check pending)".to_string(),
        exe_sha256: "pending".to_string(),
        valid: false,
        detail: "File found. Hash validation runs next.".to_string(),
    })
}

#[tauri::command]
fn set_nickname(nick: String) -> Result<String, String> {
    validate_nickname(&nick)?;
    Ok(nick)
}

#[tauri::command]
fn build_launch_args(ip: String, port: u16, nick: String) -> Result<Vec<String>, String> {
    validate_nickname(&nick)?;
    if ip.trim().is_empty() {
        return Err("Server address is empty.".to_string());
    }
    Ok(vec![
        "-c".to_string(),
        "-h".to_string(),
        ip,
        "-p".to_string(),
        port.to_string(),
        "-n".to_string(),
        nick,
    ])
}

fn main() {
    tracing_subscriber::fmt()
        .with_env_filter("icaruz=debug")
        .init();
    tauri::Builder::default()
        .invoke_handler(tauri::generate_handler![
            get_status,
            detect_gta,
            set_nickname,
            build_launch_args
        ])
        .run(tauri::generate_context!())
        .expect("ICARUZ failed to start");
}
