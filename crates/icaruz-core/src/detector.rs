use crate::{ClientKind, GameInstall};
use sha2::{Digest, Sha256};
use std::fs::File;
use std::io::{BufReader, Read};
use std::path::{Path, PathBuf};

pub fn find_exe(gta_path: &Path) -> PathBuf {
    gta_path.join("gta_sa.exe")
}

pub fn inspect_gta(gta_path: &Path) -> Result<GameInstall, String> {
    let exe = find_exe(gta_path);
    if !exe.exists() {
        return Err("gta_sa.exe not found in that folder.".to_string());
    }
    let meta = std::fs::metadata(&exe).map_err(|e| format!("Cannot read gta_sa.exe: {e}"))?;
    let size = meta.len();
    let sha = hash_file(&exe).map_err(|e| format!("Cannot hash gta_sa.exe: {e}"))?;
    // GTA SA 1.0 US is around 14.3 MB. We report, not enforce, until hash allowlist ships.
    let plausible = (13_500_000..=15_500_000).contains(&size);
    Ok(GameInstall {
        gta_path: gta_path.to_path_buf(),
        exe_version: if plausible {
            "plausible 1.0 US (size check only)".to_string()
        } else {
            format!("unexpected size {size} bytes")
        },
        exe_sha256: sha,
        valid: plausible,
        detail: if plausible {
            "File found. Size matches 1.0 US range. Full hash allowlist pending.".to_string()
        } else {
            "File found but size differs from 1.0 US. Launcher will not modify it.".to_string()
        },
    })
}

fn hash_file(path: &Path) -> std::io::Result<String> {
    let f = File::open(path)?;
    let mut reader = BufReader::new(f);
    let mut hasher = Sha256::new();
    let mut buf = [0u8; 65536];
    loop {
        let n = reader.read(&mut buf)?;
        if n == 0 {
            break;
        }
        hasher.update(&buf[..n]);
    }
    Ok(hex::encode(hasher.finalize()))
}

pub fn client_exe_name(kind: &ClientKind) -> &'static str {
    match kind {
        ClientKind::Samp037R5 => "samp.exe",
        ClientKind::Samp03DL => "samp.exe",
        ClientKind::OpenMp1x => "openmp.exe",
    }
}

// Minimal hex helper to avoid extra dep at core level.
mod hex {
    pub fn encode(bytes: impl AsRef<[u8]>) -> String {
        bytes.as_ref().iter().map(|b| format!("{b:02x}")).collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn missing_folder_errors() {
        let r = inspect_gta(Path::new("/definitely/not/here-icaruz"));
        assert!(r.is_err());
    }
}
