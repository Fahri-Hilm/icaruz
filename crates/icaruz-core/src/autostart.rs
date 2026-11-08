use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct AutostartStatus {
    pub enabled: bool,
    pub detail: String,
}

#[cfg(target_os = "windows")]
const RUN_KEY: &str = "Software\\Microsoft\\Windows\\CurrentVersion\\Run";
#[cfg(target_os = "windows")]
const VALUE_NAME: &str = "ICARUZ";

#[cfg(target_os = "windows")]
fn current_exe() -> Result<String, String> {
    std::env::current_exe()
        .map(|p| p.to_string_lossy().to_string())
        .map_err(|e| format!("Cannot find launcher path: {e}"))
}

#[cfg(target_os = "windows")]
pub fn get_autostart() -> Result<AutostartStatus, String> {
    use winreg::enums::HKEY_CURRENT_USER;
    use winreg::RegKey;
    let hkcu = RegKey::predef(HKEY_CURRENT_USER);
    let run = hkcu
        .open_subkey(RUN_KEY)
        .map_err(|e| format!("Cannot read startup key: {e}"))?;
    let val: Result<String, _> = run.get_value(VALUE_NAME);
    Ok(AutostartStatus {
        enabled: val.is_ok(),
        detail: match val {
            Ok(v) => format!("Starts at login: {v}"),
            Err(_) => "Not set to start at login.".to_string(),
        },
    })
}

#[cfg(target_os = "windows")]
pub fn set_autostart(enabled: bool) -> Result<AutostartStatus, String> {
    use winreg::enums::HKEY_CURRENT_USER;
    use winreg::RegKey;
    let exe = current_exe()?;
    let hkcu = RegKey::predef(HKEY_CURRENT_USER);
    let (run, _) = hkcu
        .create_subkey(RUN_KEY)
        .map_err(|e| format!("Cannot open startup key: {e}"))?;
    if enabled {
        run.set_value(VALUE_NAME, &format!("\"{exe}\""))
            .map_err(|e| format!("Cannot enable startup: {e}"))?;
    } else {
        let _ = run.delete_value(VALUE_NAME);
    }
    get_autostart()
}

#[cfg(not(target_os = "windows"))]
pub fn get_autostart() -> Result<AutostartStatus, String> {
    Ok(AutostartStatus {
        enabled: false,
        detail: "Startup toggle only applies on Windows.".to_string(),
    })
}

#[cfg(not(target_os = "windows"))]
pub fn set_autostart(_enabled: bool) -> Result<AutostartStatus, String> {
    get_autostart()
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn status_has_detail() {
        let s = get_autostart().unwrap();
        assert!(!s.detail.is_empty());
        #[cfg(not(target_os = "windows"))]
        assert!(!s.enabled);
    }
}
