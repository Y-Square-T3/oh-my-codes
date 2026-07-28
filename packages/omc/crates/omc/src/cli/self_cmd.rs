use std::env;
use std::fs;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::process::Command;

use console::style;
use indicatif::{ProgressBar, ProgressStyle};
use serde::Deserialize;

use super::{SelfAction, SelfCmd};
use crate::cli::ui::{print_error, print_warning};

const REPO: &str = "Y-Square-T3/oh-my-codes";

#[derive(Debug, Deserialize)]
struct GitHubRelease {
    tag_name: String,
}

enum InstallMethod {
    Npm,
    ShellScript,
    Source,
}

pub async fn run(cmd: SelfCmd) -> Result<(), Box<dyn std::error::Error>> {
    match cmd.action {
        SelfAction::Upgrade { check } => upgrade(check).await,
    }
}

async fn upgrade(check_only: bool) -> Result<(), Box<dyn std::error::Error>> {
    let current_version = env!("CARGO_PKG_VERSION");
    println!(
        "  {} Current version: {}",
        style("ℹ").blue().bold(),
        style(current_version).cyan()
    );

    let install_method = detect_install_method();
    let latest_version = check_latest_version().await?;

    println!(
        "  {} Latest version: {}",
        style("ℹ").blue().bold(),
        style(&latest_version).cyan()
    );

    if is_up_to_date(current_version, &latest_version) {
        println!(
            "  {} Already up to date",
            style("✓").green().bold()
        );
        return Ok(());
    }

    if check_only {
        println!(
            "  {} Update available: {} → {}",
            style("↑").yellow().bold(),
            current_version,
            latest_version
        );
        return Ok(());
    }

    println!(
        "  {} Upgrading to {}...",
        style("→").blue().bold(),
        latest_version
    );

    match install_method {
        InstallMethod::Npm => upgrade_npm().await,
        InstallMethod::ShellScript => upgrade_binary(&latest_version).await,
        InstallMethod::Source => {
            print_warning("Installed from source. Please rebuild manually:");
            println!("    cargo build --release");
            Ok(())
        }
    }
}

fn detect_install_method() -> InstallMethod {
    let exe_path = env::current_exe().unwrap_or_default();
    let path_str = exe_path.to_string_lossy();

    if path_str.contains("node_modules") && path_str.contains("@y-square-t3/oh-my-codes-") {
        InstallMethod::Npm
    } else if path_str.contains("target/") {
        InstallMethod::Source
    } else {
        InstallMethod::ShellScript
    }
}

async fn check_latest_version() -> Result<String, Box<dyn std::error::Error>> {
    let url = format!("https://api.github.com/repos/{}/releases/latest", REPO);
    let client = reqwest::Client::new();
    let resp = client
        .get(&url)
        .header("User-Agent", "omc-cli")
        .send()
        .await?;

    if !resp.status().is_success() {
        return Err(format!("Failed to fetch latest version: HTTP {}", resp.status()).into());
    }

    let release: GitHubRelease = resp.json().await?;
    let version = release.tag_name.trim_start_matches('v').to_string();
    Ok(version)
}

fn is_up_to_date(current: &str, latest: &str) -> bool {
    current == latest
}

async fn upgrade_npm() -> Result<(), Box<dyn std::error::Error>> {
    let package_manager = detect_package_manager()?;
    println!(
        "  {} Detected package manager: {}",
        style("ℹ").blue().bold(),
        style(&package_manager).cyan()
    );

    let install_cmd = match package_manager.as_str() {
        "yarn" => vec!["yarn", "add", "oh-my-codes@latest"],
        "pnpm" => vec!["pnpm", "add", "oh-my-codes@latest"],
        _ => vec!["npm", "install", "oh-my-codes@latest"],
    };

    println!(
        "  {} Running: {}",
        style("→").blue().bold(),
        install_cmd.join(" ")
    );

    let status = Command::new(install_cmd[0])
        .args(&install_cmd[1..])
        .status()?;

    if !status.success() {
        print_error("Upgrade failed");
        return Err("Package manager command failed".into());
    }

    println!(
        "  {} Upgrade complete! Restart your shell to use the new version.",
        style("✓").green().bold()
    );
    Ok(())
}

fn detect_package_manager() -> Result<String, Box<dyn std::error::Error>> {
    let exe_path = env::current_exe()?;
    let mut dir = exe_path.parent();

    while let Some(d) = dir {
        if d.join("yarn.lock").exists() {
            return Ok("yarn".to_string());
        }
        if d.join("pnpm-lock.yaml").exists() {
            return Ok("pnpm".to_string());
        }
        if d.join("package-lock.json").exists() {
            return Ok("npm".to_string());
        }
        dir = d.parent();
    }

    Ok("npm".to_string())
}

async fn upgrade_binary(latest_version: &str) -> Result<(), Box<dyn std::error::Error>> {
    let platform = detect_platform()?;
    println!(
        "  {} Detected platform: {}",
        style("ℹ").blue().bold(),
        style(&platform).cyan()
    );

    let archive_name = if cfg!(windows) {
        format!("omc-{}.zip", platform)
    } else {
        format!("omc-{}.tar.gz", platform)
    };

    let download_url = format!(
        "https://github.com/{}/releases/download/v{}/{}",
        REPO, latest_version, archive_name
    );

    println!(
        "  {} Downloading...",
        style("→").blue().bold()
    );

    let temp_dir = env::temp_dir().join(format!("omc-upgrade-{}", latest_version));
    fs::create_dir_all(&temp_dir)?;

    let archive_path = temp_dir.join(&archive_name);
    download_with_progress(&download_url, &archive_path).await?;

    println!(
        "  {} Extracting...",
        style("→").blue().bold()
    );

    extract_archive(&archive_path, &temp_dir)?;

    println!(
        "  {} Installing...",
        style("→").blue().bold()
    );

    replace_binaries(&temp_dir)?;

    fs::remove_dir_all(&temp_dir)?;

    println!(
        "  {} Upgrade complete!",
        style("✓").green().bold()
    );
    Ok(())
}

fn detect_platform() -> Result<String, Box<dyn std::error::Error>> {
    let os = if cfg!(target_os = "macos") {
        "darwin"
    } else if cfg!(target_os = "linux") {
        "linux"
    } else if cfg!(target_os = "windows") {
        "win32"
    } else {
        return Err("Unsupported OS".into());
    };

    let arch = if cfg!(target_arch = "aarch64") {
        "arm64"
    } else if cfg!(target_arch = "x86_64") {
        "x64"
    } else {
        return Err("Unsupported architecture".into());
    };

    Ok(format!("{}-{}", os, arch))
}

async fn download_with_progress(
    url: &str,
    dest: &PathBuf,
) -> Result<(), Box<dyn std::error::Error>> {
    let client = reqwest::Client::new();
    let resp = client
        .get(url)
        .header("User-Agent", "omc-cli")
        .send()
        .await?;

    if !resp.status().is_success() {
        return Err(format!("Download failed: HTTP {}", resp.status()).into());
    }

    let total_size = resp.content_length().unwrap_or(0);
    let pb = ProgressBar::new(total_size);
    pb.set_style(
        ProgressStyle::default_bar()
            .template("{spinner:.green} [{bar:40.cyan/blue}] {bytes}/{total_bytes} ({eta})")?
            .progress_chars("█▉▊▋▌▍▎▏  "),
    );

    let bytes = resp.bytes().await?;
    pb.set_position(bytes.len() as u64);

    let mut file = fs::File::create(dest)?;
    file.write_all(&bytes)?;

    pb.finish_with_message("Downloaded");
    Ok(())
}

fn extract_archive(archive: &Path, dest: &Path) -> Result<(), Box<dyn std::error::Error>> {
    let archive_str = archive.to_string_lossy();

    if archive_str.ends_with(".tar.gz") {
        let status = Command::new("tar")
            .args(["-xzf", &archive_str, "-C", &dest.to_string_lossy()])
            .status()?;

        if !status.success() {
            return Err("Failed to extract tar.gz".into());
        }
    } else if archive_str.ends_with(".zip") {
        let status = Command::new("unzip")
            .args(["-o", &archive_str, "-d", &dest.to_string_lossy()])
            .status()?;

        if !status.success() {
            return Err("Failed to extract zip".into());
        }
    } else {
        return Err("Unknown archive format".into());
    }

    Ok(())
}

fn replace_binaries(temp_dir: &Path) -> Result<(), Box<dyn std::error::Error>> {
    let current_exe = env::current_exe()?;
    let install_dir = current_exe.parent().ok_or("Cannot determine install directory")?;

    let exe_ext = if cfg!(windows) { ".exe" } else { "" };

    let new_omc = temp_dir.join(format!("omc{}", exe_ext));
    let new_omcd = temp_dir.join(format!("omcd{}", exe_ext));

    let target_omc = install_dir.join(format!("omc{}", exe_ext));
    let target_omcd = install_dir.join(format!("omcd{}", exe_ext));

    replace_binary(&new_omc, &target_omc)?;
    replace_binary(&new_omcd, &target_omcd)?;

    Ok(())
}

fn replace_binary(new: &PathBuf, target: &PathBuf) -> Result<(), Box<dyn std::error::Error>> {
    if cfg!(windows) {
        let old_backup = target.with_extension("old");
        if old_backup.exists() {
            fs::remove_file(&old_backup)?;
        }
        if target.exists() {
            fs::rename(target, &old_backup)?;
        }
        fs::rename(new, target)?;
    } else {
        if target.exists() {
            fs::remove_file(target)?;
        }
        fs::copy(new, target)?;

        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let perms = fs::Permissions::from_mode(0o755);
            fs::set_permissions(target, perms)?;
        }
    }

    Ok(())
}
