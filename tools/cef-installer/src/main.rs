use anyhow::{Context, Result};
use serde::Deserialize;
use download_cef::{CefIndex, DEFAULT_TARGET};
use std::path::{Path, PathBuf};

fn default_install_dir() -> PathBuf {
    dirs::home_dir().expect("no home dir").join(".local/share/cef")
}

fn find_workspace_lock() -> Result<PathBuf> {
    let mut dir = std::env::current_dir()?;

    loop {
        let candidate = dir.join("Cargo.lock");
        if candidate.exists() {
            return Ok(candidate);
        }

        if !dir.pop() {
            anyhow::bail!("Could not find Cargo.lock in this project");
        }
    }
}

#[derive(Deserialize)]
struct Lockfile {
    package: Vec<Package>,
}

#[derive(Deserialize)]
struct Package {
    name: String,
    version: String,
}

fn required_cef_version(lock_path: &Path) -> Result<String> {
    let content = std::fs::read_to_string(lock_path)?;
    let parsed: Lockfile = toml::from_str(&content)?;

    let pkg = parsed
        .package
        .into_iter()
        .find(|p| p.name == "cef-dll-sys")
        .context("cef-dll-sys not found in Cargo.lock (did you build once?)")?;

    // version format i.e. 145.2.0+145.0.24
    let cef_version = pkg
        .version
        .split('+')
        .nth(1)
        .context("invalid cef-dll-sys version format")?;

    Ok(cef_version.to_string())
}

fn main() -> Result<()> {
    println!("rust-cef-runtime installer");

    let lock = find_workspace_lock()?;
    println!("Using lockfile: {}", lock.display());

    let cef_version = required_cef_version(&lock)?;
    println!("Required CEF version: {}", cef_version);

    let install_dir = default_install_dir();
    std::fs::create_dir_all(&install_dir)?;

    let index = CefIndex::download()?;
    let platform = index.platform(DEFAULT_TARGET)?;
    let version = platform.version(&cef_version)?;

    println!("Downloading matching CEF build...");

    let archive = version.download_archive_with_retry(
        &install_dir,
        true,
        std::time::Duration::from_secs(10),
        3,
    )?;

    download_cef::extract_target_archive(DEFAULT_TARGET, &archive, &install_dir, true)?;

    println!("\n[+] CEF installed successfully");
    println!("Location: {}", install_dir.display());
    println!("You can now run: cargo run --example demo");

    Ok(())
}
