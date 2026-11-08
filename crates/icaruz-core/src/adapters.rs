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

pub fn auto_install_supported(_kind: &ClientKind) -> bool {
    false
}

pub fn install_hint(kind: &ClientKind) -> &'static str {
    match kind {
        ClientKind::Samp037R5 => "Auto-install not supported. Install SA-MP 0.3.7-R5 manually into the GTA folder, then press Play.",
        ClientKind::Samp03DL => "Auto-install not supported. Install SA-MP 0.3DL manually into the GTA folder, then press Play.",
        ClientKind::OpenMp1x => "Auto-install not supported. Install open.mp 1.x manually into the GTA folder, then press Play.",
    }
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
        ClientKind::Samp03DL => {
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
                note: "SA-MP 0.3DL classic CLI, same flags as 0.3.7. Custom models load from server. Auto-install not supported, install client manually.".to_string(),
            })
        }
        ClientKind::OpenMp1x => {
            let exe = gta_path.join("openmp.exe");
            let target = format!("{host}:{p}");
            Ok(AdapterPlan {
                kind: kind.clone(),
                exe,
                args: vec![
                    "--server".to_string(),
                    target,
                    "--name".to_string(),
                    nick.to_string(),
                ],
                note: "open.mp 1.x CLI. If flags are rejected by your build, connect from open.mp UI manually. Auto-install not supported, install client manually.".to_string(),
            })
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn plan_037_classic_flags() {
        let p = plan_launch(
            &ClientKind::Samp037R5,
            Path::new("/gta"),
            "127.0.0.1:7777",
            7777,
            "Fajuu_01",
        )
        .unwrap();
        assert!(p.exe.ends_with("samp.exe"));
        assert_eq!(p.args[1], "-h");
        assert_eq!(p.args[3], "-p");
    }

    #[test]
    fn plan_03dl_wired() {
        let p = plan_launch(
            &ClientKind::Samp03DL,
            Path::new("/gta"),
            "play.example.com",
            7777,
            "Fajuu_01",
        )
        .unwrap();
        assert!(p.exe.ends_with("samp.exe"));
        assert!(p.note.contains("Auto-install not supported"));
    }

    #[test]
    fn plan_openmp_wired() {
        let p = plan_launch(
            &ClientKind::OpenMp1x,
            Path::new("/gta"),
            "127.0.0.1:7777",
            7777,
            "Fajuu_01",
        )
        .unwrap();
        assert!(p.exe.ends_with("openmp.exe"));
        assert!(p.args.contains(&"127.0.0.1:7777".to_string()));
        assert!(p.note.contains("Auto-install not supported"));
    }

    #[test]
    fn plan_rejects_bad_nick_and_server() {
        assert!(plan_launch(
            &ClientKind::Samp037R5,
            Path::new("/gta"),
            "127.0.0.1:7777",
            7777,
            "ab"
        )
        .is_err());
        assert!(plan_launch(
            &ClientKind::OpenMp1x,
            Path::new("/gta"),
            "",
            7777,
            "Fajuu_01"
        )
        .is_err());
    }

    #[test]
    fn auto_install_honest() {
        for k in [
            ClientKind::Samp037R5,
            ClientKind::Samp03DL,
            ClientKind::OpenMp1x,
        ] {
            assert!(!auto_install_supported(&k));
            assert!(install_hint(&k).contains("not supported"));
        }
    }
}
