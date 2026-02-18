use cef::{args::Args, *};
use std::sync::{Arc, Mutex};
use std::path::PathBuf;
use std::sync::OnceLock;

use crate::cef_app::DemoApp;
use crate::error::RuntimeError;
use crate::fs_pool;

static ASSET_ROOT: OnceLock<PathBuf> = OnceLock::new();

/// Public entry point for launching a CEF application.
///
/// Responsible for:
/// - Initializing platform-specific CEF requirements
/// - Spawning CEF subprocesses
/// - Starting the browser process
/// - Running the CEF message loop
pub struct Runtime;

impl Runtime {
    /// Launches the CEF runtime and blocks until shutdown.
    ///
    /// start_url determines what the browser loads on startup.
    pub fn run(start_url: CefString, require_assets: bool) -> Result<(), RuntimeError> {
        Self::auto_configure_cef()?;

        if require_assets {
            Self::validate_asset_root()?;
        }

        fs_pool::init_worker_pool();

        #[cfg(target_os = "macos")]
        crate::platform::macos::init_ns_app();

        let _ = api_hash(sys::CEF_API_VERSION_LAST, 0);

        let args = Args::new();

        let window = Arc::new(Mutex::new(None));
        let mut app = DemoApp::new(window.clone(), start_url);

        let ret = execute_process(
            Some(args.as_main_args()),
            Some(&mut app),
            std::ptr::null_mut(),
        );

        // Subprocesses exit immediately
        if ret >= 0 {
            std::process::exit(ret);
        }

        let exe = std::env::current_exe()
            .expect("failed to get current exe path");
        let exe_str = exe.to_string_lossy();

        let cache_dir = std::env::temp_dir().join("rust_cef_runtime");
        std::fs::create_dir_all(&cache_dir).ok();

        // Use a persistent profile instead of CEF's default incognito mode.
        // This enables cookies, storage APIs and service workers.
        let settings = Settings {
            no_sandbox: 1,
            browser_subprocess_path: CefString::from(exe_str.as_ref()),
            cache_path: CefString::from(cache_dir.to_string_lossy().as_ref()),
            root_cache_path: CefString::from(cache_dir.to_string_lossy().as_ref()),
            persist_session_cookies: 1,
            ..Default::default()
        };

        if initialize(
            Some(args.as_main_args()),
            Some(&settings),
            Some(&mut app),
            std::ptr::null_mut(),
        ) != 1 {
            return Err(RuntimeError::CefInitializeFailed);
        }

        run_message_loop();
        shutdown();
        Ok(())
    }

    fn auto_configure_cef() -> Result<(), RuntimeError> {
        use std::env;

        if env::var("CEF_PATH").is_ok() {
            return Ok(());
        }

        let home = dirs::home_dir().ok_or(RuntimeError::CefNotInstalled)?;

        #[cfg(target_os = "windows")]
        let cef = home.join(".local/share/cef/cef_windows_x86_64/libcef.dll");

        #[cfg(target_os = "linux")]
        let cef = home.join(".local/share/cef/cef_linux_x86_64/libcef.so");

        #[cfg(target_os = "macos")]
        let cef = home.join(".local/share/cef/cef_macos_x86_64/Chromium Embedded Framework.framework");

        if !cef.exists() {
            return Err(RuntimeError::CefNotInstalled);
        }

        let root = cef.parent().unwrap();
        env::set_var("CEF_PATH", root);

        Ok(())
    }

    pub fn set_asset_root(path: PathBuf) -> Result<(), RuntimeError> {
        let canonical = path
            .canonicalize()
            .map_err(|_| RuntimeError::AssetRootMissing(path.clone()))?;

        ASSET_ROOT
            .set(canonical)
            .map_err(|_| RuntimeError::AssetRootNotSet)?;

        Ok(())
    }

    pub fn asset_root() -> PathBuf {
        ASSET_ROOT.get().expect("asset root not set").clone()
    }

    fn validate_asset_root() -> Result<(), RuntimeError> {
        let root = ASSET_ROOT.get().ok_or(RuntimeError::AssetRootNotSet)?;

        if !root.exists() {
            return Err(RuntimeError::AssetRootMissing(root.clone()));
        }

        Ok(())
    }
}
