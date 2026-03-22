use anyhow::Result;
use std::process::Command;

pub fn run() -> Result<()> {
    println!("Starting dev mode...");

    let cef = dirs::home_dir()
        .expect("no home dir")
        .join(".local/share/cef");

    if !cef.exists() {
        println!("CEF not found. Installing...");
        crate::install::run()?;
    }

    // Pass env to build step
    let mut cmd = Command::new("cargo");
    cmd.arg("run");

    cmd.env("CEF_PATH", &cef);

    //
    // OS-specific runtime linking
    //
    #[cfg(target_os = "linux")]
    {
        let mut ld = std::env::var("LD_LIBRARY_PATH").unwrap_or_default();
        ld = format!("{}:{}", cef.display(), ld);
        cmd.env("LD_LIBRARY_PATH", ld);
    }

    #[cfg(target_os = "windows")]
    {
        let mut path = std::env::var("PATH").unwrap_or_default();
        path = format!("{};{}", cef.display(), path);
        cmd.env("PATH", path);
    }

    #[cfg(target_os = "macos")]
    {
        let mut dyld =
            std::env::var("DYLD_FALLBACK_LIBRARY_PATH").unwrap_or_default();
        dyld = format!("{}:{}", cef.display(), dyld);
        cmd.env("DYLD_FALLBACK_LIBRARY_PATH", dyld);
    }

    let status = cmd.status()?;

    if !status.success() {
        anyhow::bail!("Application failed");
    }

    Ok(())
}
