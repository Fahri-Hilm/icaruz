use serde::{Deserialize, Serialize};
use std::path::Path;

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
pub enum CheckStatus {
    Pass,
    Warn,
    Fail,
    Unknown,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RepairItem {
    pub id: String,
    pub label: String,
    pub status: CheckStatus,
    pub detail: String,
    pub fix_hint: String,
    pub needs_approval: bool,
    pub backup_before_fix: bool,
    pub can_rollback: bool,
}

fn item(
    id: &str,
    label: &str,
    status: CheckStatus,
    detail: String,
    fix_hint: String,
    can_rollback: bool,
) -> RepairItem {
    RepairItem {
        id: id.to_string(),
        label: label.to_string(),
        status,
        detail,
        fix_hint,
        needs_approval: true,
        backup_before_fix: true,
        can_rollback,
    }
}

pub fn av_policy_text() -> String {
    "Launcher never disables antivirus or firewall. If a file is flagged, review it by hand first.".to_string()
}

#[cfg(target_os = "windows")]
fn system_file(name: &str) -> bool {
    let root = std::env::var("SystemRoot").unwrap_or_else(|_| "C:\\Windows".to_string());
    Path::new(&root).join("System32").join(name).exists()
}

#[cfg(not(target_os = "windows"))]
fn system_file(_name: &str) -> bool {
    false
}

pub fn repair_check(gta_path: &Path) -> Vec<RepairItem> {
    let mut out = Vec::new();

    if !gta_path.exists() {
        out.push(item(
            "gta_folder",
            "GTA folder",
            CheckStatus::Fail,
            "Folder not found. Launcher changed nothing.".to_string(),
            "Pick the folder that holds gta_sa.exe, then run check again. No files touched until you approve.".to_string(),
            false,
        ));
        return out;
    }

    let exe = gta_path.join("gta_sa.exe");
    out.push(if exe.exists() {
        item(
            "gta_exe",
            "gta_sa.exe present",
            CheckStatus::Pass,
            "Main exe found. No change needed.".to_string(),
            "No fix needed.".to_string(),
            false,
        )
    } else {
        item(
            "gta_exe",
            "gta_sa.exe present",
            CheckStatus::Fail,
            "gta_sa.exe missing in this folder. Launcher changed nothing.".to_string(),
            "Point the launcher to the real GTA San Andreas folder. No auto download, needs your approval.".to_string(),
            false,
        )
    });

    let d3d9 = gta_path.join("d3d9.dll");
    out.push(if d3d9.exists() {
        item(
            "d3d9",
            "d3d9.dll in GTA folder",
            CheckStatus::Pass,
            "d3d9.dll found next to the game. Leave it as is.".to_string(),
            "No fix needed.".to_string(),
            true,
        )
    } else {
        item(
            "d3d9",
            "d3d9.dll in GTA folder",
            CheckStatus::Warn,
            "d3d9.dll not in GTA folder. Game can still run if DirectX9 runtime is installed system wide.".to_string(),
            "Install DirectX9 June 2010 runtime from Microsoft by hand, then recheck. Backup GTA folder first. Needs your approval.".to_string(),
            true,
        )
    });

    #[cfg(target_os = "windows")]
    {
        let dx = system_file("d3dx9_43.dll");
        out.push(if dx {
            item(
                "directx9",
                "DirectX9 runtime",
                CheckStatus::Pass,
                "d3dx9_43.dll found in System32.".to_string(),
                "No fix needed.".to_string(),
                false,
            )
        } else {
            item(
                "directx9",
                "DirectX9 runtime",
                CheckStatus::Warn,
                "d3dx9_43.dll not seen. Many SA installs need the June 2010 redist.".to_string(),
                "Download DirectX End-User Runtime June 2010 from Microsoft and run it yourself. Reboot if asked. Needs your approval.".to_string(),
                false,
            )
        });

        let vc = system_file("vcruntime140.dll") || system_file("msvcp140.dll");
        out.push(if vc {
            item(
                "vcpp",
                "Visual C++ runtime",
                CheckStatus::Pass,
                "VC++ 2015 to 2022 runtime dll found.".to_string(),
                "No fix needed.".to_string(),
                false,
            )
        } else {
            item(
                "vcpp",
                "Visual C++ runtime",
                CheckStatus::Warn,
                "VC++ runtime dll not seen. Some clients fail to start without it.".to_string(),
                "Install Microsoft Visual C++ Redistributable x86 by hand, then recheck. Needs your approval.".to_string(),
                false,
            )
        });

        out.push(item(
            "admin",
            "Admin rights",
            CheckStatus::Unknown,
            "Some fixes write to Program Files or System32 and ask for admin. Launcher does not elevate by itself.".to_string(),
            "If a step asks for admin, you approve first, then relaunch as admin by hand. Launcher never forces elevation.".to_string(),
            false,
        ));
    }

    #[cfg(not(target_os = "windows"))]
    {
        for (id, label, detail) in [
            (
                "directx9",
                "DirectX9 runtime",
                "System check runs on Windows only. On Linux this stays Unknown and blocks no flow.",
            ),
            (
                "vcpp",
                "Visual C++ runtime",
                "System check runs on Windows only. On Linux this stays Unknown and blocks no flow.",
            ),
            (
                "admin",
                "Admin rights",
                "Admin check runs on Windows only. Launcher never elevates by itself.",
            ),
        ] {
            out.push(item(
                id,
                label,
                CheckStatus::Unknown,
                detail.to_string(),
                "Recheck on Windows. No change made on this OS.".to_string(),
                false,
            ));
        }
    }

    out.push(item(
        "dep",
        "DEP note",
        CheckStatus::Unknown,
        "Data Execution Prevention can crash old GTA builds. Launcher never edits DEP by itself.".to_string(),
        "If GTA exits right away, ask user before any DEP edit. Manual path: System Properties, Performance, DEP, add exception, reboot. Needs your approval and a restore point.".to_string(),
        true,
    ));

    out.push(item(
        "antivirus",
        "Antivirus policy",
        CheckStatus::Unknown,
        av_policy_text(),
        "If antivirus flags a file, keep it quarantined and verify hash by hand. Launcher offers verify and backup only.".to_string(),
        false,
    ));

    out
}

pub fn needs_attention(items: &[RepairItem]) -> bool {
    items
        .iter()
        .any(|i| i.status == CheckStatus::Fail || i.status == CheckStatus::Warn)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn missing_folder_is_fail() {
        let items = repair_check(Path::new("/definitely/not/here-icaruz"));
        assert!(items.iter().any(|i| i.status == CheckStatus::Fail));
        assert!(items.iter().all(|i| i.needs_approval));
    }

    #[test]
    fn d3d9_warn_needs_approval_and_backup() {
        let dir = std::env::temp_dir().join("icaruz-repair-test");
        let _ = std::fs::create_dir_all(&dir);
        let items = repair_check(&dir);
        let d3d9 = items.iter().find(|i| i.id == "d3d9").unwrap();
        assert_eq!(d3d9.status, CheckStatus::Warn);
        assert!(d3d9.needs_approval);
        assert!(d3d9.backup_before_fix);
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn av_policy_never_disables() {
        let t = av_policy_text().to_lowercase();
        assert!(t.contains("never disables") || t.contains("never disable"));
    }

    #[test]
    fn dep_item_asks_approval() {
        let dir = std::env::temp_dir().join("icaruz-repair-dep");
        let _ = std::fs::create_dir_all(&dir);
        let items = repair_check(&dir);
        let dep = items.iter().find(|i| i.id == "dep").unwrap();
        assert!(dep.needs_approval);
        assert!(dep.can_rollback);
        let _ = std::fs::remove_dir_all(&dir);
    }
}
