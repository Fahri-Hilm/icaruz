use crate::ClientKind;
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AdapterPlan {
    pub kind: ClientKind,
    pub exe: PathBuf,
    pub args: Vec<String>,
    pub note: String,
}

pub fn plan_launch(
    kind: &ClientKind,
    gta_path: &Path,
    ip: &str,
    port: u16,
    nick: &str,
) -> Result<AdapterPlan, String> {
    crate::validate_nickname(nick)?;
    let (host, p) = crate::launch::parse_server(ip, port)?;
    match kind {
        ClientKind::Samp037R5 => {
            let exe = gta_path.join("samp.exe");
            Ok(AdapterPlan {
                kind: kind.clone(),
                exe,
                args: vec![
                    "-c".to_string(),
                    "-h".to_string(),
                    host,
                    "-p".to_string(),
                    p.to_string(),
                    "-n".to_string(),
                    nick.to_string(),
                ],
                note: "SA-MP 0.3.7-R5 classic CLI.".to_string(),
            })
        }
        ClientKind::Samp03DL => Err("SA-MP 0.3DL adapter not wired in P2. Ships in P3.".to_string()),
        ClientKind::OpenMp1x => Err("open.mp adapter not wired in P2. Ships in P3.".to_string()),
    }
}
