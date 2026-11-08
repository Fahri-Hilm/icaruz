use crate::ClientKind;
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct RollbackMarker {
    pub client: ClientKind,
    pub profile: String,
    pub committed_at: String,
    pub had_previous: bool,
}

pub fn client_slug(kind: &ClientKind) -> &'static str {
    match kind {
        ClientKind::Samp037R5 => "samp037r5",
        ClientKind::Samp03DL => "samp03dl",
        ClientKind::OpenMp1x => "openmp1x",
    }
}

pub fn sanitize_profile(profile: &str) -> String {
    let mut s: String = profile
        .chars()
        .map(|c| {
            if c.is_alphanumeric() || c == '_' || c == '-' {
                c
            } else {
                '_'
            }
        })
        .collect();
    s.truncate(32);
    if s.is_empty() {
        s.push_str("default");
    }
    s
}

pub fn runtime_dir(base: &Path, kind: &ClientKind, profile: &str) -> PathBuf {
    base.join("runtimes")
        .join(client_slug(kind))
        .join(sanitize_profile(profile))
}

pub fn staging_dir(base: &Path, kind: &ClientKind, profile: &str) -> PathBuf {
    let mut p = runtime_dir(base, kind, profile).into_os_string();
    p.push(".staging");
    PathBuf::from(p)
}

fn backup_dir(base: &Path, kind: &ClientKind, profile: &str) -> PathBuf {
    let mut p = runtime_dir(base, kind, profile).into_os_string();
    p.push(".backup");
    PathBuf::from(p)
}

pub fn rollback_file(base: &Path, kind: &ClientKind, profile: &str) -> PathBuf {
    let mut p = runtime_dir(base, kind, profile).into_os_string();
    p.push(".rollback.json");
    PathBuf::from(p)
}

fn ensure_under_base(base: &Path, path: &Path) -> Result<(), String> {
    if path.starts_with(base) {
        Ok(())
    } else {
        Err("Refusing path outside runtime base.".to_string())
    }
}

// Only immutable vendor files may use hardlink. Mutable configs always copy.
pub fn is_immutable_file(path: &Path) -> bool {
    match path.extension().and_then(|e| e.to_str()) {
        Some(ext) => matches!(
            ext.to_lowercase().as_str(),
            "exe" | "dll" | "asi" | "flt"
        ),
        None => false,
    }
}

pub fn link_or_copy_immutable(src: &Path, dst: &Path) -> Result<(), String> {
    if let Some(parent) = dst.parent() {
        std::fs::create_dir_all(parent).map_err(|e| format!("Cannot create dir: {e}"))?;
    }
    if is_immutable_file(src) {
        if std::fs::hard_link(src, dst).is_ok() {
            return Ok(());
        }
    }
    std::fs::copy(src, dst).map_err(|e| format!("Cannot copy file: {e}"))?;
    Ok(())
}

pub fn prepare_staging(base: &Path, kind: &ClientKind, profile: &str) -> Result<PathBuf, String> {
    let st = staging_dir(base, kind, profile);
    ensure_under_base(base, &st)?;
    if st.exists() {
        std::fs::remove_dir_all(&st).map_err(|e| format!("Cannot clear staging: {e}"))?;
    }
    std::fs::create_dir_all(&st).map_err(|e| format!("Cannot create staging: {e}"))?;
    Ok(st)
}

// Atomic replace via rename. Never touches GTA folder, only runtime base.
pub fn commit_staging(base: &Path, kind: &ClientKind, profile: &str) -> Result<RollbackMarker, String> {
    let st = staging_dir(base, kind, profile);
    let target = runtime_dir(base, kind, profile);
    let backup = backup_dir(base, kind, profile);
    ensure_under_base(base, &st)?;
    ensure_under_base(base, &target)?;
    if !st.exists() {
        return Err("Staging dir missing, nothing to commit.".to_string());
    }
    if backup.exists() {
        std::fs::remove_dir_all(&backup).map_err(|e| format!("Cannot clear backup: {e}"))?;
    }
    let had_previous = target.exists();
    if had_previous {
        std::fs::rename(&target, &backup).map_err(|e| format!("Cannot backup current: {e}"))?;
    }
    let rename_ok = std::fs::rename(&st, &target);
    if let Err(e) = rename_ok {
        if had_previous && backup.exists() {
            let _ = std::fs::rename(&backup, &target);
        }
        return Err(format!("Atomic replace failed: {e}"));
    }
    if backup.exists() {
        std::fs::remove_dir_all(&backup).map_err(|e| format!("Cannot drop backup: {e}"))?;
    }
    let marker = RollbackMarker {
        client: kind.clone(),
        profile: sanitize_profile(profile),
        committed_at: now_stamp(),
        had_previous,
    };
    write_rollback(base, kind, profile, &marker)?;
    Ok(marker)
}

fn now_stamp() -> String {
    use std::time::{SystemTime, UNIX_EPOCH};
    let s = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0);
    s.to_string()
}

pub fn write_rollback(
    base: &Path,
    kind: &ClientKind,
    profile: &str,
    marker: &RollbackMarker,
) -> Result<(), String> {
    let f = rollback_file(base, kind, profile);
    ensure_under_base(base, &f)?;
    let json = serde_json::to_string_pretty(marker).map_err(|e| format!("Rollback encode failed: {e}"))?;
    std::fs::write(&f, json).map_err(|e| format!("Rollback write failed: {e}"))?;
    Ok(())
}

pub fn read_rollback(
    base: &Path,
    kind: &ClientKind,
    profile: &str,
) -> Result<Option<RollbackMarker>, String> {
    let f = rollback_file(base, kind, profile);
    if !f.exists() {
        return Ok(None);
    }
    let text = std::fs::read_to_string(&f).map_err(|e| format!("Rollback read failed: {e}"))?;
    let m: RollbackMarker =
        serde_json::from_str(&text).map_err(|e| format!("Rollback parse failed: {e}"))?;
    Ok(Some(m))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn profile_slug_safe() {
        assert_eq!(sanitize_profile("Main RP!"), "Main_RP_");
        assert_eq!(sanitize_profile(""), "default");
        assert_eq!(client_slug(&ClientKind::OpenMp1x), "openmp1x");
    }

    #[test]
    fn immutable_gate() {
        assert!(is_immutable_file(Path::new("samp.dll")));
        assert!(is_immutable_file(Path::new("a.EXE")));
        assert!(!is_immutable_file(Path::new("settings.ini")));
        assert!(!is_immutable_file(Path::new("noext")));
    }

    #[test]
    fn stage_commit_rollback_roundtrip() {
        let base = std::env::temp_dir().join(format!("icaruz-rt-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&base);
        let kind = ClientKind::Samp037R5;
        let st = prepare_staging(&base, &kind, "main").unwrap();
        std::fs::write(st.join("settings.ini"), "nick=x").unwrap();
        let m1 = commit_staging(&base, &kind, "main").unwrap();
        assert!(!m1.had_previous);
        assert!(runtime_dir(&base, &kind, "main").join("settings.ini").exists());
        let st2 = prepare_staging(&base, &kind, "main").unwrap();
        std::fs::write(st2.join("settings.ini"), "nick=y").unwrap();
        let m2 = commit_staging(&base, &kind, "main").unwrap();
        assert!(m2.had_previous);
        let back = read_rollback(&base, &kind, "main").unwrap().unwrap();
        assert_eq!(back.profile, "main");
        let _ = std::fs::remove_dir_all(&base);
    }

    #[test]
    fn commit_without_staging_fails() {
        let base = std::env::temp_dir().join(format!("icaruz-rt-no-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&base);
        std::fs::create_dir_all(&base).unwrap();
        assert!(commit_staging(&base, &ClientKind::OpenMp1x, "p").is_err());
        let _ = std::fs::remove_dir_all(&base);
    }
}
