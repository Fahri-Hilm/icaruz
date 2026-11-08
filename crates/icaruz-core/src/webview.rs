use serde::{Deserialize, Serialize};
#[cfg(target_os = "windows")]
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WebviewStatus {
    pub installed: bool,
    pub detail: String,
}

// Installer sengaja skip WebView2 agar setup kecil dan cepat.
// Check ini dipakai banner Home, bukan untuk auto-download diam-diam.
pub fn webview_status() -> WebviewStatus {
    #[cfg(target_os = "windows")]
    {
        for dir in candidate_dirs() {
            let exe = dir.join("msedgewebview2.exe");
            if exe.exists() {
                return WebviewStatus {
                    installed: true,
                    detail: format!("WebView2 found at {}.", exe.display()),
                };
            }
        }
        WebviewStatus {
            installed: false,
            detail: "WebView2 Runtime not found. Install it from Microsoft, then reopen ICARUZ."
                .to_string(),
        }
    }
    #[cfg(not(target_os = "windows"))]
    {
        WebviewStatus {
            installed: false,
            detail: "WebView2 check only runs on Windows.".to_string(),
        }
    }
}

#[cfg(target_os = "windows")]
fn candidate_dirs() -> Vec<PathBuf> {
    let mut out = Vec::new();
    let mut push_app_dir = |base: &str| {
        let app = PathBuf::from(base)
            .join("Microsoft")
            .join("EdgeWebView")
            .join("Application");
        if let Ok(rd) = std::fs::read_dir(&app) {
            for e in rd.flatten() {
                let p = e.path();
                if p.is_dir() {
                    out.push(p);
                }
            }
        }
    };
    if let Ok(pf) = std::env::var("ProgramFiles") {
        push_app_dir(&pf);
    }
    if let Ok(pf86) = std::env::var("ProgramFiles(x86)") {
        push_app_dir(&pf86);
    }
    if let Ok(local) = std::env::var("LOCALAPPDATA") {
        push_app_dir(&local);
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn status_has_detail() {
        let s = webview_status();
        assert!(!s.detail.is_empty());
        #[cfg(not(target_os = "windows"))]
        assert!(!s.installed);
    }
}
