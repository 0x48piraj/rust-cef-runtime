use cef::{args::Args, *};
use std::sync::{Arc, Mutex};
use std::path::PathBuf;
use std::sync::OnceLock;

use crate::app::DemoApp;
use crate::error::RuntimeError;

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
    pub fn run(start_url: CefString) -> Result<(), RuntimeError> {
        Self::validate_asset_root()?;

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

        let settings = Settings {
            no_sandbox: 1,
            browser_subprocess_path: CefString::from(exe_str.as_ref()),
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

    pub fn set_asset_root(path: PathBuf) {
        let canonical = path
            .canonicalize()
            .expect("asset root must exist and be absolute");

        println!("Asset root set to: {:?}", canonical);

        ASSET_ROOT.set(canonical).expect("asset root already set");
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
