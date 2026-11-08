use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::path::{Path, PathBuf};

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
pub enum Confidence {
    Low,
    Medium,
    High,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CrashInfo {
    pub exception: String,
    pub address: String,
    pub modules: Vec<String>,
    pub raw_lines: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CrashReport {
    pub exit_code: i32,
    pub exit_meaning: String,
    pub crashinfo: Option<CrashInfo>,
    pub wer_files: Vec<PathBuf>,
    pub signature: String,
    pub confidence: Confidence,
    pub note: String,
}

pub fn exit_code_meaning(code: i32) -> &'static str {
    match code {
        0 => "Clean exit.",
        1 => "Generic error. See logs.",
        -1073741819 => "Access violation (0xC0000005). Often bad memory read or conflicting mod.",
        -1073741795 => "Illegal instruction (0xC000001D). Often broken exe or bad hook.",
        -1073741571 => "Stack overflow (0xC00000FD). Often deep recursion or heavy mod.",
        -1073741825 => "Privileged instruction (0xC0000096). Often compat or overlay hook issue.",
        -532462766 => "Unhandled .NET exception (0xE0434352). Check client logs.",
        _ => "Unknown code. Needs crashinfo.txt and logs to group.",
    }
}

fn short_module_name(line: &str) -> Option<String> {
    let low = line.to_lowercase();
    for ext in [".dll", ".asi", ".exe", ".flt", ".cs"] {
        if let Some(pos) = low.find(ext) {
            let end = pos + ext.len();
            let start = line[..pos]
                .rfind(|c: char| c == '\\' || c == '/' || c == ' ' || c == '"' || c == '\'')
                .map(|i| i + 1)
                .unwrap_or(0);
            let name = line[start..end.min(line.len())].trim().to_string();
            if !name.is_empty() && name.len() <= 80 {
                return Some(name);
            }
        }
    }
    None
}

pub fn parse_modules(text: &str) -> Vec<String> {
    let mut mods = Vec::new();
    for line in text.lines() {
        if let Some(m) = short_module_name(line) {
            let key = m.to_lowercase();
            if !mods.iter().any(|x: &String| x.to_lowercase() == key) {
                mods.push(m);
            }
            if mods.len() >= 16 {
                break;
            }
        }
    }
    mods
}

pub fn parse_crashinfo(text: &str) -> Option<CrashInfo> {
    if text.trim().is_empty() {
        return None;
    }
    let mut exception = String::new();
    let mut address = String::new();
    for line in text.lines() {
        let low = line.to_lowercase();
        if exception.is_empty() && low.contains("exception") {
            exception = line.trim().chars().take(160).collect();
        }
        if address.is_empty() && low.contains("address") {
            address = line.trim().chars().take(160).collect();
        }
    }
    let modules = parse_modules(text);
    if exception.is_empty() && address.is_empty() && modules.is_empty() {
        return None;
    }
    Some(CrashInfo {
        exception: if exception.is_empty() {
            "not stated".to_string()
        } else {
            exception
        },
        address: if address.is_empty() {
            "not stated".to_string()
        } else {
            address
        },
        modules,
        raw_lines: text.lines().count(),
    })
}

pub fn find_wer_files(gta_path: &Path) -> Vec<PathBuf> {
    let mut out = Vec::new();
    let push_if = |dir: &Path, out: &mut Vec<PathBuf>| {
        let rd = std::fs::read_dir(dir);
        if let Ok(rd) = rd {
            for e in rd.flatten().take(40) {
                let p = e.path();
                let low = p.to_string_lossy().to_lowercase();
                if low.ends_with(".dmp")
                    || low.ends_with("crashinfo.txt")
                    || low.ends_with("crash.log")
                {
                    out.push(p);
                    if out.len() >= 10 {
                        break;
                    }
                }
            }
        }
    };
    push_if(gta_path, &mut out);
    push_if(&gta_path.join("crashes"), &mut out);
    #[cfg(target_os = "windows")]
    {
        if let Ok(local) = std::env::var("LOCALAPPDATA") {
            push_if(Path::new(&local).join("CrashDumps").as_path(), &mut out);
        }
    }
    out
}

pub fn build_signature(exit_code: i32, info: Option<&CrashInfo>) -> String {
    let mut base = format!("exit={exit_code}");
    if let Some(i) = info {
        let top = i.modules.first().cloned().unwrap_or_default().to_lowercase();
        base.push_str(&format!("|exc={}|mod={}", i.exception.to_lowercase(), top));
    }
    let mut h = Sha256::new();
    h.update(base.as_bytes());
    let hex: String = h.finalize().iter().map(|b| format!("{b:02x}")).collect();
    hex[..16].to_string()
}

fn confidence_for(exit_known: bool, info: &Option<CrashInfo>) -> Confidence {
    match (exit_known, info) {
        (true, Some(i)) if !i.modules.is_empty() => Confidence::Medium,
        (true, Some(_)) => Confidence::Medium,
        (false, Some(i)) if !i.modules.is_empty() => Confidence::Medium,
        (false, Some(_)) => Confidence::Low,
        (_, None) => Confidence::Low,
    }
}

pub fn build_report(gta_path: &Path, exit_code: i32) -> CrashReport {
    let meaning = exit_code_meaning(exit_code).to_string();
    let exit_known = meaning != "Unknown code. Needs crashinfo.txt and logs to group.";
    let crash_path = gta_path.join("crashinfo.txt");
    let crashinfo = std::fs::read_to_string(&crash_path)
        .ok()
        .and_then(|t| parse_crashinfo(&t));
    let wer_files = find_wer_files(gta_path);
    let signature = build_signature(exit_code, crashinfo.as_ref());
    let confidence = confidence_for(exit_known, &crashinfo);
    CrashReport {
        exit_code,
        exit_meaning: meaning,
        crashinfo,
        wer_files,
        signature,
        confidence,
        note: "Module listed is possibly related, not proven cause. Verify with clean install test.".to_string(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const SAMPLE: &str = "SA-MP CrashInfo\nException At Address: 0x007F0A2B\nModule: samp.dll\nLoaded: gta_sa.exe d3d9.dll mod.asi\n";

    #[test]
    fn exit_map_known() {
        assert!(exit_code_meaning(0).contains("Clean"));
        assert!(exit_code_meaning(-1073741819).contains("Access violation"));
        assert!(exit_code_meaning(999999).contains("Unknown"));
    }

    #[test]
    fn parse_sample() {
        let info = parse_crashinfo(SAMPLE).unwrap();
        assert!(info.exception.to_lowercase().contains("exception"));
        assert!(!info.modules.is_empty());
    }

    #[test]
    fn signature_stable() {
        let a = build_signature(-1073741819, parse_crashinfo(SAMPLE).as_ref());
        let b = build_signature(-1073741819, parse_crashinfo(SAMPLE).as_ref());
        assert_eq!(a, b);
        assert_eq!(a.len(), 16);
        let c = build_signature(0, None);
        assert_ne!(a, c);
    }

    #[test]
    fn low_confidence_without_file() {
        let r = build_report(Path::new("/definitely/not/here-icaruz"), 12345);
        assert_eq!(r.confidence, Confidence::Low);
        assert!(r.note.contains("not proven"));
    }

    #[test]
    fn modules_cap() {
        let mods = parse_modules(SAMPLE);
        assert!(mods.len() <= 16);
    }
}
