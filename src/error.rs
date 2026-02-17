use std::fmt::{Display, Formatter};

#[derive(Debug)]
pub enum RuntimeError {
    AssetRootNotSet,
    AssetRootMissing(std::path::PathBuf),
    CefInitializeFailed,
}

impl Display for RuntimeError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            RuntimeError::AssetRootNotSet =>
                write!(f, "Runtime::set_asset_root() was not called before Runtime::run()"),

            RuntimeError::AssetRootMissing(p) =>
                write!(f, "Asset root does not exist: {}", p.display()),

            RuntimeError::CefInitializeFailed =>
                write!(f, "CEF failed to initialize"),
        }
    }
}

impl std::error::Error for RuntimeError {}
