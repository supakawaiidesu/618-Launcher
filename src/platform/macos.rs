// macOS-specific functionality

use std::path::PathBuf;

/// Get the Application Support directory
pub fn application_support_dir() -> Option<PathBuf> {
    std::env::var("HOME")
        .ok()
        .map(|home| PathBuf::from(home).join("Library/Application Support"))
}

/// Get the user's Applications directory
pub fn applications_dir() -> PathBuf {
    PathBuf::from("/Applications")
}

/// Check if an app bundle exists
pub fn app_bundle_exists(name: &str) -> bool {
    let path = applications_dir().join(format!("{}.app", name));
    path.exists()
}

/// Get the executable path from an app bundle
pub fn get_app_executable(app_path: &PathBuf) -> Option<PathBuf> {
    // macOS app bundles have executables in Contents/MacOS/
    let contents = app_path.join("Contents/MacOS");
    if contents.exists() {
        // Usually the executable has the same name as the app
        if let Some(app_name) = app_path.file_stem() {
            let exe = contents.join(app_name);
            if exe.exists() {
                return Some(exe);
            }
        }
        // Otherwise, look for any executable
        if let Ok(entries) = std::fs::read_dir(&contents) {
            for entry in entries.flatten() {
                let path = entry.path();
                if path.is_file() {
                    return Some(path);
                }
            }
        }
    }
    None
}
