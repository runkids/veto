//! Upgrade command - self-update veto to the latest version

use colored::Colorize;
use semver::Version;
use std::env;
use std::fs;
use std::io::{self, Write};
use std::path::PathBuf;
use std::process::Command;

const GITHUB_API_URL: &str = "https://api.github.com/repos/runkids/veto/releases/latest";
const USER_AGENT: &str = "veto-upgrade";

/// Get the current version of veto
fn get_current_version() -> Version {
    Version::parse(env!("CARGO_PKG_VERSION")).expect("Invalid package version")
}

/// Release information from GitHub API
#[derive(Debug)]
struct ReleaseInfo {
    version: Version,
    assets: Vec<AssetInfo>,
}

/// Asset information from GitHub release
#[derive(Debug)]
struct AssetInfo {
    name: String,
    download_url: String,
}

/// Fetch the latest release information from GitHub
fn fetch_latest_release() -> Result<ReleaseInfo, Box<dyn std::error::Error>> {
    let output = Command::new("curl")
        .args(["-s", "-H", &format!("User-Agent: {}", USER_AGENT), GITHUB_API_URL])
        .output()?;

    if !output.status.success() {
        return Err("Failed to fetch release information from GitHub".into());
    }

    let json: serde_json::Value = serde_json::from_slice(&output.stdout)?;

    let tag_name = json["tag_name"]
        .as_str()
        .ok_or("No tag_name in release")?;

    // Parse version from tag (remove 'v' prefix if present)
    let version_str = tag_name.strip_prefix('v').unwrap_or(tag_name);
    let version = Version::parse(version_str)?;

    // Parse assets
    let assets = json["assets"]
        .as_array()
        .ok_or("No assets in release")?
        .iter()
        .filter_map(|asset| {
            let name = asset["name"].as_str()?.to_string();
            let download_url = asset["browser_download_url"].as_str()?.to_string();
            Some(AssetInfo { name, download_url })
        })
        .collect();

    Ok(ReleaseInfo { version, assets })
}

/// Detect the current OS and architecture
fn detect_platform() -> Result<(String, String), Box<dyn std::error::Error>> {
    let os = env::consts::OS;
    let arch = env::consts::ARCH;

    let os_name = match os {
        "macos" => "apple-darwin",
        "linux" => "unknown-linux-gnu",
        "windows" => "pc-windows-msvc",
        _ => return Err(format!("Unsupported OS: {}", os).into()),
    };

    let arch_name = match arch {
        "x86_64" => "x86_64",
        "aarch64" => "aarch64",
        _ => return Err(format!("Unsupported architecture: {}", arch).into()),
    };

    Ok((arch_name.to_string(), os_name.to_string()))
}

/// Find the appropriate asset for the current platform
fn find_asset_for_platform<'a>(
    assets: &'a [AssetInfo],
    arch: &str,
    os: &str,
) -> Option<&'a AssetInfo> {
    let target = format!("{}-{}", arch, os);
    assets.iter().find(|a| a.name.contains(&target) && a.name.ends_with(".tar.gz"))
}

/// Get the current executable path
fn get_current_exe_path() -> Result<PathBuf, Box<dyn std::error::Error>> {
    env::current_exe().map_err(|e| format!("Failed to get current executable path: {}", e).into())
}

/// Check if we have write permission to the executable
fn can_write_to_exe(path: &PathBuf) -> bool {
    // Try to open the file for writing
    fs::OpenOptions::new().write(true).open(path).is_ok()
}

/// Download a file to a temporary location
fn download_file(url: &str, dest: &PathBuf) -> Result<(), Box<dyn std::error::Error>> {
    let status = Command::new("curl")
        .args(["-sL", "-o", dest.to_str().unwrap(), url])
        .status()?;

    if !status.success() {
        return Err("Failed to download file".into());
    }

    Ok(())
}

/// Extract tarball to a directory
fn extract_tarball(tarball: &PathBuf, dest_dir: &PathBuf) -> Result<(), Box<dyn std::error::Error>> {
    let status = Command::new("tar")
        .args(["-xzf", tarball.to_str().unwrap(), "-C", dest_dir.to_str().unwrap()])
        .status()?;

    if !status.success() {
        return Err("Failed to extract tarball".into());
    }

    Ok(())
}

/// Install the new binary, using sudo if necessary
fn install_binary(
    src: &PathBuf,
    dest: &PathBuf,
    use_sudo: bool,
) -> Result<(), Box<dyn std::error::Error>> {
    if use_sudo {
        eprintln!("{}", "Requesting elevated permissions to install...".yellow());
        let status = Command::new("sudo")
            .args(["cp", src.to_str().unwrap(), dest.to_str().unwrap()])
            .status()?;

        if !status.success() {
            return Err("Failed to install with sudo".into());
        }

        // Set executable permission
        Command::new("sudo")
            .args(["chmod", "+x", dest.to_str().unwrap()])
            .status()?;
    } else {
        fs::copy(src, dest)?;

        // Set executable permission on Unix
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let mut perms = fs::metadata(dest)?.permissions();
            perms.set_mode(0o755);
            fs::set_permissions(dest, perms)?;
        }
    }

    Ok(())
}

/// Run the upgrade process
pub fn run_upgrade(check_only: bool, force: bool) -> Result<(), Box<dyn std::error::Error>> {
    let current_version = get_current_version();
    println!("{} {}", "Current version:".cyan(), current_version);

    print!("{}", "Checking for updates...".dimmed());
    io::stdout().flush()?;

    let release = fetch_latest_release()?;
    println!(" {}", "done".green());

    println!("{} {}", "Latest version:".cyan(), release.version);

    // Compare versions
    if release.version <= current_version && !force {
        println!("{}", "✓ You are already on the latest version!".green());
        return Ok(());
    }

    if release.version > current_version {
        println!(
            "{} {} → {}",
            "Update available:".yellow().bold(),
            current_version,
            release.version.to_string().green()
        );
    } else if force {
        println!("{}", "Force reinstall requested.".yellow());
    }

    if check_only {
        println!("{}", "Run 'veto upgrade' to install the update.".dimmed());
        return Ok(());
    }

    // Detect platform
    let (arch, os) = detect_platform()?;
    println!("{} {}-{}", "Platform:".cyan(), arch, os);

    // Find appropriate asset
    let asset = find_asset_for_platform(&release.assets, &arch, &os)
        .ok_or_else(|| format!("No release asset found for {}-{}", arch, os))?;

    println!("{} {}", "Downloading:".cyan(), asset.name);

    // Create temp directory
    let temp_dir = env::temp_dir().join("veto-upgrade");
    fs::create_dir_all(&temp_dir)?;

    // Download tarball
    let tarball_path = temp_dir.join(&asset.name);
    download_file(&asset.download_url, &tarball_path)?;
    println!("{}", "✓ Download complete".green());

    // Extract
    print!("{}", "Extracting...".dimmed());
    io::stdout().flush()?;
    extract_tarball(&tarball_path, &temp_dir)?;
    println!(" {}", "done".green());

    // Find the extracted binary
    let extracted_binary = temp_dir.join("veto");
    if !extracted_binary.exists() {
        // Try finding in a subdirectory (some releases package in a folder)
        let entries: Vec<_> = fs::read_dir(&temp_dir)?
            .filter_map(|e| e.ok())
            .filter(|e| e.path().is_dir())
            .collect();

        for entry in entries {
            let candidate = entry.path().join("veto");
            if candidate.exists() {
                fs::rename(&candidate, &extracted_binary)?;
                break;
            }
        }
    }

    if !extracted_binary.exists() {
        return Err("Could not find veto binary in extracted files".into());
    }

    // Get current executable path
    let current_exe = get_current_exe_path()?;
    println!("{} {}", "Installing to:".cyan(), current_exe.display());

    // Check permissions and install
    let need_sudo = !can_write_to_exe(&current_exe);
    install_binary(&extracted_binary, &current_exe, need_sudo)?;

    // Cleanup
    let _ = fs::remove_dir_all(&temp_dir);

    println!();
    println!("{}", "✓ Successfully upgraded to version ".green().to_string() + &release.version.to_string().green().to_string() + "!");
    println!("{}", "  Run 'veto --version' to verify.".dimmed());

    Ok(())
}
