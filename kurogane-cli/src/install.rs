use anyhow::{Context, Result};
use cargo_metadata::MetadataCommand;
use download_cef::{CefIndex, DEFAULT_TARGET};
use std::time::Duration;
use std::path::{Path, PathBuf};

pub fn run() -> Result<()> {
    println!("Kurogane CEF installer");

    let install_dir = default_install_dir(); // ~/.local/share/cef

    if install_dir.exists() {
        println!("[+] CEF already installed at {}", install_dir.display());
        return Ok(());
    }

    let cef_version = match required_cef_version() {
        Ok(v) => v,
        Err(_) => {
            println!("[~] Could not detect project CEF version, falling back...");
            return Err(anyhow::anyhow!(
                "Run this inside a kurogane project or build once."
            ));
        }
    };

    println!("[!] Required CEF version: {}", cef_version);

    let parent = install_dir.parent().unwrap(); // ~/.local/share
    std::fs::create_dir_all(parent)?;

    let index = CefIndex::download()?;
    let platform = index.platform(DEFAULT_TARGET)?;
    let version = platform.version(&cef_version)?;

    println!("[*] Downloading matching CEF build...");

    let archive = version.download_archive_with_retry(
        parent,
        true,
        Duration::from_secs(15),
        3,
    )?;

    println!("[*] Extracting...");
    let extracted = download_cef::extract_target_archive(
        DEFAULT_TARGET,
        &archive,
        parent,
        true,
    )?;

    // Write archive.json
    version.minimal()?.write_archive_json(&extracted)?;

    // Replace existing install (safety, though we already early-returned)
    if install_dir.exists() {
        println!("[*] Removing old install...");
        std::fs::remove_dir_all(&install_dir)?;
    }

    println!("[*] Installing to {}", install_dir.display());
    std::fs::rename(&extracted, &install_dir)?;

    let _ = std::fs::remove_file(&archive);

    println!("\n[+] CEF installed at {}", install_dir.display());
    print_env_instructions(&install_dir);

    Ok(())
}

fn default_install_dir() -> PathBuf {
    dirs::home_dir()
        .expect("no home dir")
        .join(".local/share/cef")
}

fn required_cef_version() -> Result<String> {
    let metadata = MetadataCommand::new()
        .exec()
        .context("failed to read cargo metadata")?;

    let pkg = metadata
        .packages
        .iter()
        .find(|p| p.name == "cef-dll-sys")
        .context("cef-dll-sys not found in dependency graph")?;

    let version = pkg.version.to_string();

    let cef_version = version
        .split('+')
        .nth(1)
        .context("invalid cef-dll-sys version format")?;

    Ok(cef_version.to_string())
}

fn print_env_instructions(root: &Path) {
    println!("\nInitializing runtime (one-time setup)\n");

    #[cfg(target_os = "windows")]
    {
        println!("PowerShell:");
        println!(r#"$env:CEF_PATH="{}""#, root.display());
        println!(r#"$env:PATH="$env:PATH;$env:CEF_PATH""#);
    }

    #[cfg(target_os = "linux")]
    {
        println!(r#"export CEF_PATH="{}""#, root.display());
        println!(r#"export LD_LIBRARY_PATH="$LD_LIBRARY_PATH:$CEF_PATH""#);
        println!("Run once:");
        println!(
            " sudo chown root:root {}/chrome-sandbox",
            root.display()
        );
        println!(
            " sudo chmod 4755 {}/chrome-sandbox",
            root.display()
        );
    }

    #[cfg(target_os = "macos")]
    {
        println!(r#"export CEF_PATH="{}""#, root.display());
        println!(
            r#"export DYLD_FALLBACK_LIBRARY_PATH="$DYLD_FALLBACK_LIBRARY_PATH:$CEF_PATH:$CEF_PATH/Chromium Embedded Framework.framework/Libraries""#
        );
    }

    println!("\nRestart your terminal after running these commands.");
    println!("Then run: kurogane dev\n");
}
