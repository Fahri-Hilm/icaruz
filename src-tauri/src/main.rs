use icaruz_core::{
    adapters, content, crash, detector, launch, repair, storage, validate_nickname, ClientKind,
    ClientStatus, GameInstall, LaunchPhase, LaunchRequest, LauncherStatus,
};
use std::path::PathBuf;
use std::sync::Mutex;
use tauri::State;

struct RunState(Mutex<Option<u32>>);

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
    LauncherStatus {
        gta: None,
        clients: default_clients(None),
        nickname: "Player".to_string(),
        needs_attention: Some("GTA San Andreas folder not set yet.".to_string()),
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

fn main() {
    tracing_subscriber::fmt()
        .with_env_filter("icaruz=debug")
        .init();
    tauri::Builder::default()
        .manage(RunState(Mutex::new(None)))
        .invoke_handler(tauri::generate_handler![
            get_status,
            detect_gta,
            set_nickname,
            build_launch_args,
            plan_launch,
            launch_game,
            download_client,
            repair_check,
            crash_report
        ])
        .run(tauri::generate_context!())
        .expect("ICARUZ failed to start");
}
