// Windows-specific functionality

#[cfg(target_os = "windows")]
use winreg::enums::*;
#[cfg(target_os = "windows")]
use winreg::RegKey;

/// Get a string value from the Windows registry
#[cfg(target_os = "windows")]
pub fn get_registry_string(hive: &str, path: &str, key: &str) -> Option<String> {
    let hkey = match hive {
        "HKEY_LOCAL_MACHINE" | "HKLM" => RegKey::predef(HKEY_LOCAL_MACHINE),
        "HKEY_CURRENT_USER" | "HKCU" => RegKey::predef(HKEY_CURRENT_USER),
        _ => return None,
    };

    hkey.open_subkey(path)
        .ok()
        .and_then(|k| k.get_value::<String, _>(key).ok())
}

/// Check if a program is installed by looking for its registry entry
#[cfg(target_os = "windows")]
pub fn is_program_installed(name: &str) -> bool {
    let paths = [
        format!("SOFTWARE\\Microsoft\\Windows\\CurrentVersion\\Uninstall\\{}", name),
        format!(
            "SOFTWARE\\WOW6432Node\\Microsoft\\Windows\\CurrentVersion\\Uninstall\\{}",
            name
        ),
    ];

    let hklm = RegKey::predef(HKEY_LOCAL_MACHINE);

    for path in &paths {
        if hklm.open_subkey(path).is_ok() {
            return true;
        }
    }

    false
}
