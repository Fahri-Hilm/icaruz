use crate::ClientKind;
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ServerWorkspace {
    pub server_id: String,
    pub address: String,
    pub port: u16,
    pub nickname: String,
    pub client: ClientKind,
    pub profile: String,
    pub updated_at: String,
}

pub fn server_id(address: &str, port: u16) -> String {
    let mut s: String = format!("{address}_{port}")
        .chars()
        .map(|c| {
            if c.is_alphanumeric() || c == '_' || c == '-' {
                c
            } else {
                '_'
            }
        })
        .collect();
    s.truncate(64);
    s.to_lowercase()
}

pub fn workspace_dir(base: &Path, address: &str, port: u16) -> PathBuf {
    base.join("servers").join(server_id(address, port))
}

pub fn workspace_file(base: &Path, address: &str, port: u16) -> PathBuf {
    workspace_dir(base, address, port).join("workspace.json")
}

pub fn cache_dir(base: &Path, address: &str, port: u16) -> PathBuf {
    workspace_dir(base, address, port).join("cache")
}

fn now_stamp() -> String {
    use std::time::{SystemTime, UNIX_EPOCH};
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs().to_string())
        .unwrap_or_else(|_| "0".to_string())
}

pub fn default_workspace(address: &str, port: u16, nickname: &str) -> ServerWorkspace {
    ServerWorkspace {
        server_id: server_id(address, port),
        address: address.to_string(),
        port,
        nickname: nickname.to_string(),
        client: ClientKind::Samp037R5,
        profile: "default".to_string(),
        updated_at: now_stamp(),
    }
}

pub fn save_workspace(base: &Path, ws: &ServerWorkspace) -> Result<PathBuf, String> {
    let dir = workspace_dir(base, &ws.address, ws.port);
    std::fs::create_dir_all(&dir).map_err(|e| format!("Cannot create workspace: {e}"))?;
    std::fs::create_dir_all(cache_dir(base, &ws.address, ws.port))
        .map_err(|e| format!("Cannot create cache: {e}"))?;
    let mut stamped = ws.clone();
    stamped.updated_at = now_stamp();
    let f = workspace_file(base, &ws.address, ws.port);
    let json = serde_json::to_string_pretty(&stamped)
        .map_err(|e| format!("Workspace encode failed: {e}"))?;
    std::fs::write(&f, json).map_err(|e| format!("Workspace write failed: {e}"))?;
    Ok(f)
}

pub fn load_workspace(
    base: &Path,
    address: &str,
    port: u16,
) -> Result<Option<ServerWorkspace>, String> {
    let f = workspace_file(base, address, port);
    if !f.exists() {
        return Ok(None);
    }
    let text = std::fs::read_to_string(&f).map_err(|e| format!("Workspace read failed: {e}"))?;
    let ws: ServerWorkspace =
        serde_json::from_str(&text).map_err(|e| format!("Workspace parse failed: {e}"))?;
    Ok(Some(ws))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn id_slug_stable() {
        assert_eq!(server_id("127.0.0.1", 7777), "127_0_0_1_7777");
        assert_eq!(server_id("Play.Example.com", 7777), "play_example_com_7777");
    }

    #[test]
    fn save_load_roundtrip() {
        let base = std::env::temp_dir().join(format!("icaruz-ws-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&base);
        let ws = ServerWorkspace {
            server_id: server_id("127.0.0.1", 7777),
            address: "127.0.0.1".into(),
            port: 7777,
            nickname: "Fajuu_01".into(),
            client: ClientKind::OpenMp1x,
            profile: "main".into(),
            updated_at: "0".into(),
        };
        save_workspace(&base, &ws).unwrap();
        assert!(cache_dir(&base, "127.0.0.1", 7777).exists());
        let back = load_workspace(&base, "127.0.0.1", 7777).unwrap().unwrap();
        assert_eq!(back.nickname, "Fajuu_01");
        assert_eq!(back.client, ClientKind::OpenMp1x);
        assert!(load_workspace(&base, "10.0.0.9", 7777).unwrap().is_none());
        let _ = std::fs::remove_dir_all(&base);
    }
}
