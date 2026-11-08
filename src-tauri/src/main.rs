use icaruz_core::{
    adapters, autostart, content, crash, detector, discovery, launch, repair, storage,
    validate_nickname, webview, ClientKind, ClientStatus, GameInstall, LaunchPhase, LaunchRequest,
    LauncherStatus,
};
use std::path::PathBuf;
use std::sync::Mutex;
use tauri::State;

struct RunState(Mutex<Option<u32>>);
struct AppState {
    cache: discovery::DiscoveryCache,
}

const OPENMP_API: &str = "https://api.open.mp/servers";

fn default_clients(gta_path: Option<&PathBuf>) -> Vec<ClientStatus> {
    vec![
        ClientStatus {
            kind: ClientKind::Samp037R5,
            installed: gta_path
                .map(|p| p.join("samp.exe").exists())
                .unwrap_or(false),
            version: "P2 adapter".to_string(),
            path: gta_path.map(|p| p.join("samp.exe")),
        },
        ClientStatus {
            kind: ClientKind::Samp03DL,
            installed: false,
            version: "ships in P3".to_string(),
            path: None,
        },
        ClientStatus {
            kind: ClientKind::OpenMp1x,
            installed: false,
            version: "ships in P3".to_string(),
            path: None,
        },
    ]
}

#[tauri::command]
fn get_status() -> LauncherStatus {
    let db = icaruz_core::db_path();
    let _ = storage::init_db(&db);
    let nickname = storage::get_setting(&db, "nickname")
        .ok()
        .flatten()
        .unwrap_or_else(|| "Player".to_string());
    let gta_path = storage::get_setting(&db, "gta_path").ok().flatten();
    let gta = gta_path
        .as_deref()
        .and_then(|p| detector::inspect_gta(std::path::Path::new(p)).ok());
    let clients = default_clients(gta.as_ref().map(|g| &g.gta_path));
    let needs_attention = if gta.is_none() {
        Some("GTA San Andreas folder not set yet.".to_string())
    } else {
        None
    };
    LauncherStatus {
        gta,
        clients,
        nickname,
        needs_attention,
    }
}

#[tauri::command]
fn detect_gta(path: String) -> Result<GameInstall, String> {
    let info = detector::inspect_gta(std::path::Path::new(&path))?;
    let db = icaruz_core::db_path();
    let _ = storage::init_db(&db);
    let _ = storage::save_setting(&db, "gta_path", &path);
    Ok(info)
}

#[tauri::command]
fn set_nickname(nick: String) -> Result<String, String> {
    validate_nickname(&nick)?;
    let db = icaruz_core::db_path();
    let _ = storage::init_db(&db);
    let _ = storage::save_setting(&db, "nickname", &nick);
    Ok(nick)
}

#[tauri::command]
fn plan_launch(req: LaunchRequest) -> Result<adapters::AdapterPlan, String> {
    let _ = storage::init_db(&icaruz_core::db_path());
    adapters::plan_launch(&req.client, &req.gta_path, &req.server, req.port, &req.nick)
}

#[tauri::command]
fn build_launch_args(ip: String, port: u16, nick: String) -> Result<Vec<String>, String> {
    validate_nickname(&nick)?;
    let (host, p) = launch::parse_server(&ip, port)?;
    Ok(vec![
        "-c".to_string(),
        "-h".to_string(),
        host,
        "-p".to_string(),
        p.to_string(),
        "-n".to_string(),
        nick,
    ])
}

#[tauri::command]
fn download_client(
    url: String,
    sha256: String,
    filename: String,
    size_bytes: u64,
) -> Result<String, String> {
    let spec = content::DownloadSpec {
        url,
        sha256,
        filename,
        size_bytes,
    };
    let hosts = content::default_trusted_hosts();
    content::validate_spec(&spec, &hosts)?;
    let data = icaruz_core::data_dir();
    let staging = content::staging_path(&data, &spec.filename);
    if let Some(parent) = staging.parent() {
        std::fs::create_dir_all(parent).map_err(|e| format!("Cannot prep staging: {e}"))?;
    }
    let dest = content::final_path(&data, &spec.filename);
    content::gate_auto_execute(&dest, false)?;
    let offset = content::resume_offset(&staging);
    let p = content::progress_for(offset, spec.size_bytes);
    Ok(format!(
        "Spec OK. Staging ready at {} resume {} of {} bytes ({:.1}%). Copy bytes then verify sha256 before install. No auto execute.",
        staging.display(),
        p.downloaded,
        p.total,
        p.percent
    ))
}

#[tauri::command]
fn repair_check(gta_path: String) -> Result<Vec<repair::RepairItem>, String> {
    Ok(repair::repair_check(std::path::Path::new(&gta_path)))
}

#[tauri::command]
fn crash_report(gta_path: String, exit_code: i32) -> Result<crash::CrashReport, String> {
    Ok(crash::build_report(
        std::path::Path::new(&gta_path),
        exit_code,
    ))
}

#[tauri::command(async)]
async fn launch_game(
    req: LaunchRequest,
    state: State<'_, RunState>,
) -> Result<LaunchPhase, String> {
    let plan = adapters::plan_launch(&req.client, &req.gta_path, &req.server, req.port, &req.nick)?;
    if !plan.exe.exists() {
        return Err(format!(
            "{} not found. Install {} first. Launcher changed nothing.",
            plan.exe.display(),
            req.client.label()
        ));
    }
    #[cfg(not(target_os = "windows"))]
    {
        let _ = &plan;
        return Err("Game launch needs Windows. Dry run plan OK.".to_string());
    }
    #[cfg(target_os = "windows")]
    {
        use tokio::process::Command;
        let mut cmd = Command::new(&plan.exe);
        cmd.args(&plan.args);
        cmd.current_dir(&req.gta_path);
        let child = cmd.spawn().map_err(|e| format!("Launch failed: {e}"))?;
        let pid = child.id().unwrap_or(0);
        if let Ok(mut slot) = state.0.lock() {
            *slot = Some(pid);
        }
        // Detach: forget handle, OS owns process. Game lives if UI closes.
        std::mem::forget(child);
        Ok(LaunchPhase::Running { pid })
    }
}

#[tauri::command]
fn webview_status() -> webview::WebviewStatus {
    webview::webview_status()
}

#[tauri::command(async)]
async fn discover_servers(
    force: bool,
    state: State<'_, AppState>,
) -> Result<Vec<discovery::ServerEntry>, String> {
    if force {
        state.cache.invalidate("openmp");
    }
    if let Some(hit) = state.cache.get("openmp") {
        return Ok(hit);
    }
    let v = discovery::fetch_openmp(OPENMP_API, 9000).await?;
    state.cache.put("openmp", v.clone());
    Ok(v)
}

#[tauri::command(async)]
async fn query_server_info(
    address: String,
    port: u16,
) -> Result<discovery::ServerInfo, String> {
    discovery::query_server_info(&address, port, 2500).await
}

#[tauri::command]
fn fav_list() -> Result<Vec<storage::Favorite>, String> {
    let db = icaruz_core::db_path();
    let _ = storage::init_db(&db);
    storage::list_favorites(&db)
}

#[tauri::command]
fn fav_add(server: String, port: u16, label: String) -> Result<(), String> {
    let db = icaruz_core::db_path();
    let _ = storage::init_db(&db);
    let now = chrono_now();
    storage::add_favorite(
        &db,
        &storage::Favorite {
            server,
            port,
            label,
            added_at: now,
        },
    )
}

#[tauri::command]
fn fav_remove(server: String, port: u16) -> Result<(), String> {
    let db = icaruz_core::db_path();
    let _ = storage::init_db(&db);
    storage::remove_favorite(&db, &server, port)
}

#[tauri::command]
fn hist_list(limit: i64) -> Result<Vec<storage::HistoryEntry>, String> {
    let db = icaruz_core::db_path();
    let _ = storage::init_db(&db);
    storage::list_history(&db, limit.clamp(1, 50))
}

#[tauri::command]
fn hist_record(
    server: String,
    port: u16,
    duration_secs: u64,
    error: String,
) -> Result<(), String> {
    let db = icaruz_core::db_path();
    let _ = storage::init_db(&db);
    let _ = storage::record_history(
        &db,
        &storage::HistoryEntry {
            id: None,
            server,
            port,
            started_at: chrono_now(),
            duration_secs,
            error,
            detail: String::new(),
        },
    );
    Ok(())
}

#[tauri::command]
fn get_setting(key: String) -> Result<Option<String>, String> {
    let db = icaruz_core::db_path();
    let _ = storage::init_db(&db);
    storage::get_setting(&db, &key)
}

#[tauri::command]
fn save_setting(key: String, value: String) -> Result<(), String> {
    let db = icaruz_core::db_path();
    let _ = storage::init_db(&db);
    storage::save_setting(&db, &key, &value)
}

#[tauri::command]
fn get_autostart() -> Result<autostart::AutostartStatus, String> {
    autostart::get_autostart()
}

#[tauri::command]
fn set_autostart(enabled: bool) -> Result<autostart::AutostartStatus, String> {
    autostart::set_autostart(enabled)
}

fn chrono_now() -> String {
    use std::time::{SystemTime, UNIX_EPOCH};
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs().to_string())
        .unwrap_or_default()
}

fn main() {
    tracing_subscriber::fmt()
        .with_env_filter("icaruz=debug")
        .init();
    tauri::Builder::default()
        .manage(RunState(Mutex::new(None)))
        .manage(AppState {
            cache: discovery::DiscoveryCache::new(),
        })
        .invoke_handler(tauri::generate_handler![
            get_status,
            detect_gta,
            set_nickname,
            build_launch_args,
            plan_launch,
            launch_game,
            download_client,
            repair_check,
            crash_report,
            webview_status,
            discover_servers,
            query_server_info,
            fav_list,
            fav_add,
            fav_remove,
            hist_list,
            hist_record,
            get_setting,
            save_setting,
            get_autostart,
            set_autostart
        ])
        .run(tauri::generate_context!())
        .expect("ICARUZ failed to start");
}
