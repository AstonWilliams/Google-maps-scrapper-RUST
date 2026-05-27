use anyhow::{anyhow, Result};
use std::io::{self, Write};
use std::process::Command;

pub fn ensure_chromium_installed(auto_install: bool) -> Result<()> {
    if chromium_available() {
        return Ok(());
    }

    if !prompt_install(auto_install)? {
        return Err(anyhow!("Chromium is required for full mode. Install canceled."));
    }

    install_chromium()?;

    if chromium_available() {
        Ok(())
    } else {
        Err(anyhow!("Chromium install did not complete successfully."))
    }
}

fn chromium_available() -> bool {
    let candidates = [
        "chromium",
        "chromium-browser",
        "google-chrome",
        "chrome",
        "Chromium",
    ];

    candidates.iter().any(|bin| {
        Command::new(bin)
            .arg("--version")
            .output()
            .map(|o| o.status.success())
            .unwrap_or(false)
    })
}

fn prompt_install(auto_install: bool) -> Result<bool> {
    if auto_install {
        return Ok(true);
    }

    let mut stdout = io::stdout();
    stdout.write_all(b"Chromium not found. Install now? [y/N]: ")?;
    stdout.flush()?;

    let mut input = String::new();
    io::stdin().read_line(&mut input)?;
    let answer = input.trim().to_lowercase();
    Ok(answer == "y" || answer == "yes")
}

fn install_chromium() -> Result<()> {
    #[cfg(target_os = "linux")]
    {
        if Command::new("which").arg("apt-get").output().map(|o| o.status.success()).unwrap_or(false) {
            let _ = Command::new("sudo").arg("apt-get").arg("update").status();
            let status = Command::new("sudo")
                .arg("apt-get")
                .arg("install")
                .arg("-y")
                .arg("chromium")
                .status()?;
            if status.success() {
                return Ok(());
            }
            let status2 = Command::new("sudo")
                .arg("apt-get")
                .arg("install")
                .arg("-y")
                .arg("chromium-browser")
                .status()?;
            if status2.success() {
                return Ok(());
            }
        }
        return Err(anyhow!("Auto-install not supported for this Linux distro. Install Chromium manually."));
    }

    #[cfg(target_os = "macos")]
    {
        let status = Command::new("brew")
            .arg("install")
            .arg("--cask")
            .arg("chromium")
            .status()?;
        if status.success() {
            return Ok(());
        }
        return Err(anyhow!("Auto-install failed. Install Chromium manually (brew install --cask chromium)."));
    }

    #[cfg(target_os = "windows")]
    {
        let status = Command::new("winget")
            .arg("install")
            .arg("--id=Chromium.Chromium")
            .arg("-e")
            .status()?;
        if status.success() {
            return Ok(());
        }
        return Err(anyhow!("Auto-install failed. Install Chromium manually (winget install --id=Chromium.Chromium -e)."));
    }
}
