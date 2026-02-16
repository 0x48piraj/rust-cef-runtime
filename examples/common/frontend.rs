//! Frontend resolution helpers for examples.

use cef::CefString;
use std::path::PathBuf;

/// Resolve the frontend entrypoint.
///
/// Priority:
/// 1. CEF_DEV_URL (live dev server)
/// 2. CEF_APP_PATH (custom frontend directory)
/// 3. examples/<name>/index.html (cargo run)
/// 4. assets/index.html next to the executable (release)
pub fn resolve(default_example: &str) -> (PathBuf, CefString) {

    // Dev server override
    if let Ok(url) = std::env::var("CEF_DEV_URL") {
        return (std::env::current_dir().unwrap(), CefString::from(url.as_str()));
    }

    // Custom frontend directory override
    let app_path = std::env::var("CEF_APP_PATH")
        .unwrap_or_else(|_| format!("examples/{default_example}"));

    // Cargo dev: use project root
    if let Ok(manifest_dir) = std::env::var("CARGO_MANIFEST_DIR") {
        let dir = PathBuf::from(manifest_dir).join(app_path);

        if dir.join("index.html").exists() {
            return (dir, CefString::from("app://app/index.html"));
        }
    }

    // Release: assets next to executable
    let exe = std::env::current_exe().unwrap();
    let dir = exe.parent().unwrap().join("assets");

    (dir, CefString::from("app://app/index.html"))
}
