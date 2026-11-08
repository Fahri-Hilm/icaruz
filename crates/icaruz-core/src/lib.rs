pub mod adapters;
pub mod detector;
pub mod launch;
pub mod storage;

use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum ClientKind {
    Samp037R5,
    Samp03DL,
    OpenMp1x,
}

impl ClientKind {
    pub fn label(&self) -> &'static str {
        match self {
            ClientKind::Samp037R5 => "SA-MP 0.3.7-R5",
            ClientKind::Samp03DL => "SA-MP 0.3DL",
            ClientKind::OpenMp1x => "open.mp 1.x",
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GameInstall {
    pub gta_path: PathBuf,
    pub exe_version: String,
    pub exe_sha256: String,
    pub valid: bool,
    pub detail: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LauncherStatus {
    pub gta: Option<GameInstall>,
    pub clients: Vec<ClientStatus>,
    pub nickname: String,
    pub needs_attention: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClientStatus {
    pub kind: ClientKind,
    pub installed: bool,
    pub version: String,
    pub path: Option<PathBuf>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LaunchRequest {
    pub client: ClientKind,
    pub gta_path: PathBuf,
    pub server: String,
    pub port: u16,
    pub nick: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum LaunchPhase {
    Checking,
    Preparing,
    Launching,
    Running { pid: u32 },
}

pub fn data_dir() -> PathBuf {
    #[cfg(target_os = "windows")]
    {
        if let Ok(local) = std::env::var("LOCALAPPDATA") {
            return PathBuf::from(local).join("Icaruz").join("data");
        }
    }
    PathBuf::from("./data")
}

pub fn db_path() -> PathBuf {
    data_dir().join("icaruz.db")
}

pub fn validate_nickname(nick: &str) -> Result<(), String> {
    let len = nick.chars().count();
    if !(3..=24).contains(&len) {
        return Err("Nickname must be 3 to 24 characters.".to_string());
    }
    if !nick
        .chars()
        .all(|c| c.is_alphanumeric() || c == '_' || c == '[' || c == ']')
    {
        return Err("Nickname allows letters, numbers, underscore and brackets only.".to_string());
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn nickname_rules() {
        assert!(validate_nickname("Fajuu_01").is_ok());
        assert!(validate_nickname("ab").is_err());
        assert!(validate_nickname("bad nick!").is_err());
    }
}
