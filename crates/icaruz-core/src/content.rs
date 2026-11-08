use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::fs::File;
use std::io::{BufReader, Read};
use std::path::{Path, PathBuf};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DownloadSpec {
    pub url: String,
    pub filename: String,
    pub sha256: String,
    pub size_bytes: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DownloadProgress {
    pub downloaded: u64,
    pub total: u64,
    pub percent: f32,
}

pub fn default_trusted_hosts() -> Vec<&'static str> {
    vec![
        "sa-mp.com",
        "www.sa-mp.com",
        "open.mp",
        "www.open.mp",
        "github.com",
        "objects.githubusercontent.com",
    ]
}

pub fn progress_for(downloaded: u64, total: u64) -> DownloadProgress {
    if total == 0 {
        return DownloadProgress {
            downloaded,
            total,
            percent: 0.0,
        };
    }
    let done = downloaded.min(total) as f64 / total as f64 * 100.0;
    DownloadProgress {
        downloaded,
        total,
        percent: done as f32,
    }
}

fn extract_host(url: &str) -> Result<String, String> {
    let rest = url
        .strip_prefix("https://")
        .ok_or_else(|| "URL must use https.".to_string())?;
    let host = rest
        .split(['/', '?', '#'])
        .next()
        .unwrap_or("")
        .split('@')
        .next_back()
        .unwrap_or("");
    let host = host.split(':').next().unwrap_or("").trim().to_lowercase();
    if host.is_empty() {
        return Err("URL host is empty.".to_string());
    }
    Ok(host)
}

pub fn is_trusted_source(url: &str, allowed_hosts: &[&str]) -> Result<String, String> {
    if url.trim().is_empty() {
        return Err("URL is empty.".to_string());
    }
    if url.starts_with("http://") {
        return Err("Refused plain http. Trusted source needs https.".to_string());
    }
    let host = extract_host(url)?;
    let ok = allowed_hosts
        .iter()
        .any(|h| host == h.to_lowercase() || host.ends_with(&format!(".{}", h.to_lowercase())));
    if !ok {
        return Err(format!(
            "Untrusted host {host}. Download blocked before any bytes."
        ));
    }
    Ok(host)
}

fn check_filename(filename: &str) -> Result<(), String> {
    if filename.trim().is_empty() {
        return Err("Filename is empty.".to_string());
    }
    if filename.contains('/') || filename.contains('\\') || filename.contains("..") {
        return Err("Filename must be a plain name without folders.".to_string());
    }
    if filename.len() > 128 {
        return Err("Filename too long.".to_string());
    }
    Ok(())
}

fn check_sha256(hex: &str) -> Result<(), String> {
    let s = hex.trim().to_lowercase();
    if s.len() != 64 || !s.chars().all(|c| c.is_ascii_hexdigit()) {
        return Err("sha256 must be 64 hex chars.".to_string());
    }
    Ok(())
}

pub fn validate_spec(spec: &DownloadSpec, allowed_hosts: &[&str]) -> Result<(), String> {
    check_filename(&spec.filename)?;
    check_sha256(&spec.sha256)?;
    if spec.size_bytes == 0 {
        return Err("Size must be above zero.".to_string());
    }
    is_trusted_source(&spec.url, allowed_hosts)?;
    Ok(())
}

pub fn staging_path(data_dir: &Path, filename: &str) -> PathBuf {
    data_dir.join("staging").join(format!("{filename}.part"))
}

pub fn final_path(data_dir: &Path, filename: &str) -> PathBuf {
    data_dir.join("downloads").join(filename)
}

pub fn resume_offset(staging: &Path) -> u64 {
    std::fs::metadata(staging).map(|m| m.len()).unwrap_or(0)
}

pub fn hash_file_sha256(path: &Path) -> Result<String, String> {
    let f = File::open(path).map_err(|e| format!("Cannot open file: {e}"))?;
    let mut reader = BufReader::new(f);
    let mut hasher = Sha256::new();
    let mut buf = [0u8; 65536];
    loop {
        let n = reader
            .read(&mut buf)
            .map_err(|e| format!("Cannot read file: {e}"))?;
        if n == 0 {
            break;
        }
        hasher.update(&buf[..n]);
    }
    Ok(hasher
        .finalize()
        .iter()
        .map(|b| format!("{b:02x}"))
        .collect())
}

pub fn verify_sha256(path: &Path, expected: &str) -> Result<(), String> {
    let got = hash_file_sha256(path)?;
    if got.to_lowercase() != expected.trim().to_lowercase() {
        return Err("sha256 mismatch. File kept in staging, nothing installed.".to_string());
    }
    Ok(())
}

pub fn finalize_download(staging: &Path, dest: &Path) -> Result<(), String> {
    if !staging.exists() {
        return Err("Staging file missing. Nothing to finalize.".to_string());
    }
    if let Some(parent) = dest.parent() {
        std::fs::create_dir_all(parent).map_err(|e| format!("Cannot create dest dir: {e}"))?;
    }
    std::fs::rename(staging, dest).map_err(|e| format!("Atomic move failed: {e}"))?;
    Ok(())
}

pub fn cleanup_staging(staging: &Path) {
    let _ = std::fs::remove_file(staging);
}

pub fn gate_auto_execute(path: &Path, user_approved: bool) -> Result<(), String> {
    let ext = path
        .extension()
        .and_then(|e| e.to_str())
        .unwrap_or("")
        .to_lowercase();
    let risky = matches!(ext.as_str(), "dll" | "asi" | "flt" | "cleo" | "cs");
    if risky && !user_approved {
        return Err(format!(
            "Refused auto execute .{ext}. Needs explicit user approval first."
        ));
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;

    fn hosts() -> Vec<&'static str> {
        vec!["sa-mp.com", "open.mp"]
    }

    #[test]
    fn progress_half() {
        let p = progress_for(50, 100);
        assert!((p.percent - 50.0).abs() < 0.01);
        let z = progress_for(0, 0);
        assert_eq!(z.percent, 0.0);
    }

    #[test]
    fn trusted_ok_and_reject() {
        assert!(is_trusted_source("https://sa-mp.com/dl/x.zip", &hosts()).is_ok());
        assert!(is_trusted_source("https://sub.open.mp/x.zip", &hosts()).is_ok());
        assert!(is_trusted_source("http://sa-mp.com/x.zip", &hosts()).is_err());
        assert!(is_trusted_source("https://evil.com/x.zip", &hosts()).is_err());
    }

    #[test]
    fn spec_rejects_traversal() {
        let bad = DownloadSpec {
            url: "https://sa-mp.com/x.zip".into(),
            filename: "../evil.exe".into(),
            sha256: "a".repeat(64),
            size_bytes: 10,
        };
        assert!(validate_spec(&bad, &hosts()).is_err());
    }

    #[test]
    fn verify_roundtrip() {
        let dir = std::env::temp_dir().join("icaruz-content-test");
        let _ = std::fs::create_dir_all(&dir);
        let f = dir.join("hello.bin");
        let mut fh = File::create(&f).unwrap();
        fh.write_all(b"icaruz").unwrap();
        drop(fh);
        let h = hash_file_sha256(&f).unwrap();
        assert!(verify_sha256(&f, &h).is_ok());
        assert!(verify_sha256(&f, &"0".repeat(64)).is_err());
        let _ = std::fs::remove_file(&f);
    }

    #[test]
    fn resume_and_finalize() {
        let dir = std::env::temp_dir().join("icaruz-stage-test");
        let _ = std::fs::create_dir_all(&dir);
        let st = dir.join("a.bin.part");
        let _ = std::fs::remove_file(&st);
        assert_eq!(resume_offset(&st), 0);
        std::fs::write(&st, b"12345").unwrap();
        assert_eq!(resume_offset(&st), 5);
        let dest = dir.join("a.bin");
        let _ = std::fs::remove_file(&dest);
        assert!(finalize_download(&st, &dest).is_ok());
        assert!(!st.exists());
        assert!(dest.exists());
        let _ = std::fs::remove_file(&dest);
    }

    #[test]
    fn dll_gate_refuses() {
        assert!(gate_auto_execute(Path::new("mod.dll"), false).is_err());
        assert!(gate_auto_execute(Path::new("mod.asi"), false).is_err());
        assert!(gate_auto_execute(Path::new("mod.dll"), true).is_ok());
        assert!(gate_auto_execute(Path::new("game.exe"), false).is_ok());
    }
}
