// src/cli/advanced.rs
use crate::repositories::setting::SettingOps;
use anyhow::{Context, Result};
use colored::*;
use std::fs;
use std::io::{self, Write};
use std::process::Command;

/// Enable TCP BBR congestion control
pub async fn enable_bbr() -> Result<()> {
    println!("{}", "Enabling TCP BBR Congestion Control...".cyan().bold());

    // Check if BBR is already enabled
    let current_cc = fs::read_to_string("/proc/sys/net/ipv4/tcp_congestion_control")
        .unwrap_or_default()
        .trim()
        .to_string();

    if current_cc == "bbr" {
        println!("{}", "✓ BBR is already enabled".green());
        return Ok(());
    }

    // Check if BBR module is available
    let output = Command::new("modprobe")
        .arg("tcp_bbr")
        .output()
        .context("Failed to load BBR module")?;

    if !output.status.success() {
        println!("{}", "✗ BBR module not available on this kernel".red());
        return Ok(());
    }

    // Create sysctl configuration
    let sysctl_conf = r#"# TCP BBR Congestion Control
net.core.default_qdisc=fq
net.ipv4.tcp_congestion_control=bbr
"#;

    fs::write("/etc/sysctl.d/99-bbr.conf", sysctl_conf)
        .context("Failed to write sysctl configuration")?;

    // Apply settings
    let output = Command::new("sysctl")
        .arg("-p")
        .arg("/etc/sysctl.d/99-bbr.conf")
        .output()
        .context("Failed to apply sysctl settings")?;

    if output.status.success() {
        println!("{}", "✓ BBR enabled successfully".green());
        println!("{}", "Current congestion control: bbr".dimmed());
    } else {
        println!("{}", "✗ Failed to apply BBR settings".red());
        eprintln!("{}", String::from_utf8_lossy(&output.stderr));
    }

    Ok(())
}

/// Run network speedtest
pub async fn run_speedtest() -> Result<()> {
    println!("{}", "Running Network Speedtest...".cyan().bold());
    println!("{}", "This may take a moment...".dimmed());
    println!();

    // Check if speedtest-cli is installed
    let speedtest_cmd = if which::which("speedtest-cli").is_ok() {
        "speedtest-cli"
    } else if which::which("speedtest").is_ok() {
        "speedtest"
    } else {
        println!("{}", "Installing speedtest-cli...".yellow());
        let install = Command::new("pip3")
            .args(&["install", "speedtest-cli"])
            .output();

        if install.is_err() {
            println!(
                "{}",
                "✗ speedtest-cli not found. Install with: pip3 install speedtest-cli".red()
            );
            return Ok(());
        }
        "speedtest-cli"
    };

    // Run speedtest
    let output = Command::new(speedtest_cmd)
        .arg("--simple")
        .output()
        .context("Failed to run speedtest")?;

    if output.status.success() {
        let result = String::from_utf8_lossy(&output.stdout);
        println!("{}", "Speedtest Results:".green().bold());
        println!("{}", result);
    } else {
        println!("{}", "✗ Speedtest failed".red());
        eprintln!("{}", String::from_utf8_lossy(&output.stderr));
    }

    Ok(())
}

/// Download and update geo files using GeoOrchestrator with progress tracking
pub async fn update_geo_files() -> Result<()> {
    use crate::services::geo_orchestrator::GeoOrchestrator;
    use indicatif::{ProgressBar, ProgressStyle};

    println!(
        "{}",
        "Updating Geo Files (GeoOrchestrator)...".cyan().bold()
    );
    println!();

    let assets_dir = std::env::var("RUSTRAY_ASSET_LOCATION")
        .unwrap_or_else(|_| "/usr/share/rustray".to_string());

    // Initialize GeoOrchestrator
    let orchestrator = GeoOrchestrator::new(&assets_dir)?;
    orchestrator.initialize().await?;

    // Display current state
    let state = orchestrator.get_state().await;

    if let Some(geoip_info) = &state.geoip_info {
        println!("Current geoip.dat:");
        println!("  Size: {} bytes", geoip_info.file_size);
        println!(
            "  Last Updated: {}",
            geoip_info.last_update.format("%Y-%m-%d %H:%M:%S UTC")
        );
        println!("  Checksum: {}...", &geoip_info.checksum[..16]);
    } else {
        println!("{}", "geoip.dat: Not found".yellow());
    }

    if let Some(geosite_info) = &state.geosite_info {
        println!("Current geosite.dat:");
        println!("  Size: {} bytes", geosite_info.file_size);
        println!(
            "  Last Updated: {}",
            geosite_info.last_update.format("%Y-%m-%d %H:%M:%S UTC")
        );
        println!("  Checksum: {}...", &geosite_info.checksum[..16]);
    } else {
        println!("{}", "geosite.dat: Not found".yellow());
    }

    println!();
    print!("{}", "Download latest versions? [y/N]: ".yellow().bold());
    io::stdout().flush()?;

    let mut input = String::new();
    io::stdin().read_line(&mut input)?;

    if !input.trim().eq_ignore_ascii_case("y") {
        println!("{}", "Sync cancelled".dimmed());
        return Ok(());
    }

    // Create progress bar
    let pb = ProgressBar::new(100);
    pb.set_style(
        ProgressStyle::default_bar()
            .template("{msg} [{bar:40.cyan/blue}] {bytes}/{total_bytes} ({eta})")
            .unwrap()
            .progress_chars("#>-"),
    );

    let mut last_filename = String::new();
    let pb_clone = pb.clone();
    let progress_callback = move |current: u64, total: u64| {
        let filename = if current < total / 2 {
            "geoip.dat"
        } else {
            "geosite.dat"
        };

        if filename != last_filename {
            last_filename = filename.to_string();
            pb_clone.set_message(format!("Downloading {}", filename));
        }

        if total > 0 {
            pb_clone.set_length(total);
            pb_clone.set_position(current);
        }
    };

    // Perform sync with progress tracking
    println!();
    pb.set_message("Initializing download...");

    match orchestrator.sync_with_progress(progress_callback).await {
        Ok(()) => {
            pb.finish_with_message("Download complete");
            println!();
            println!("{}", "✓ Geo files updated successfully".green().bold());

            // Display new state
            let new_state = orchestrator.get_state().await;
            if let Some(geoip) = &new_state.geoip_info {
                println!("  geoip.dat: {} bytes ({})", geoip.file_size, geoip.version);
            }
            if let Some(geosite) = &new_state.geosite_info {
                println!(
                    "  geosite.dat: {} bytes ({})",
                    geosite.file_size, geosite.version
                );
            }
        }
        Err(e) => {
            pb.abandon_with_message("Download failed");
            println!();
            println!("{}", format!("✗ Failed to update geo files: {}", e).red());
        }
    }

    Ok(())
}

/// Check for updates and perform self-update
pub async fn check_and_update() -> Result<()> {
    println!("{}", "Checking for updates...".cyan().bold());

    let client = reqwest::Client::builder().user_agent("rr-ui-cli").build()?;

    let response = client
        .get("https://api.github.com/repos/FaezBarghasa/rr-ui/releases/latest")
        .send()
        .await
        .context("Failed to check for updates")?;

    let release: serde_json::Value = response.json().await?;

    let latest_version = release["tag_name"]
        .as_str()
        .unwrap_or("unknown")
        .trim_start_matches('v');

    let current_version = env!("CARGO_PKG_VERSION");

    println!("Current version: {}", current_version.yellow());
    println!("Latest version:  {}", latest_version.green());

    if latest_version == current_version {
        println!();
        println!("{}", "✓ You are running the latest version".green().bold());
        return Ok(());
    }

    println!();
    print!(
        "{}",
        "Update available! Download and install? [y/N]: "
            .yellow()
            .bold()
    );
    io::stdout().flush()?;

    let mut input = String::new();
    io::stdin().read_line(&mut input)?;

    if !input.trim().eq_ignore_ascii_case("y") {
        println!("{}", "Update cancelled".dimmed());
        return Ok(());
    }

    // Determine architecture
    let arch = std::env::consts::ARCH;
    let arch_name = match arch {
        "x86_64" => "x86_64",
        "aarch64" => "aarch64",
        _ => {
            println!("{}", format!("✗ Unsupported architecture: {}", arch).red());
            return Ok(());
        }
    };

    // Find download URL
    let assets = release["assets"]
        .as_array()
        .context("No assets found in release")?;

    let asset = assets
        .iter()
        .find(|a| {
            a["name"]
                .as_str()
                .map(|n| n.contains(arch_name) && n.ends_with(".tar.gz") && !n.ends_with(".sha256"))
                .unwrap_or(false)
        })
        .context("No matching binary found for your architecture")?;

    let download_url = asset["browser_download_url"]
        .as_str()
        .context("Invalid download URL")?;

    let checksum_url = format!("{}.sha256", download_url);

    println!("Downloading update from: {}", download_url.dimmed());

    // Download binary
    let binary_response = client.get(download_url).send().await?;
    let binary_bytes = binary_response.bytes().await?;

    // Download checksum
    let checksum_response = client.get(&checksum_url).send().await?;
    let checksum_text = checksum_response.text().await?;
    let expected_checksum = checksum_text
        .split_whitespace()
        .next()
        .context("Invalid checksum format")?;

    // Verify checksum
    use sha2::{Digest, Sha256};
    let mut hasher = Sha256::new();
    hasher.update(&binary_bytes);
    let actual_checksum = format!("{:x}", hasher.finalize());

    if actual_checksum != expected_checksum {
        println!("{}", "✗ Checksum verification failed!".red().bold());
        println!("Expected: {}", expected_checksum);
        println!("Actual:   {}", actual_checksum);
        return Ok(());
    }

    println!("{}", "✓ Checksum verified".green());

    // Extract and install
    let temp_dir = std::env::temp_dir().join("rr-ui-update");
    fs::create_dir_all(&temp_dir)?;

    let archive_path = temp_dir.join("rr-ui.tar.gz");
    fs::write(&archive_path, binary_bytes)?;

    // Extract
    let output = Command::new("tar")
        .args(&[
            "-xzf",
            archive_path.to_str().unwrap(),
            "-C",
            temp_dir.to_str().unwrap(),
        ])
        .output()?;

    if !output.status.success() {
        println!("{}", "✗ Failed to extract archive".red());
        return Ok(());
    }

    // Install binary
    let new_binary = temp_dir.join("rr-ui");
    let current_exe = std::env::current_exe()?;

    // Backup current binary
    let backup = format!("{}.backup", current_exe.display());
    fs::copy(&current_exe, &backup)?;

    // Replace binary
    fs::copy(&new_binary, &current_exe)?;

    // Make executable
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let mut perms = fs::metadata(&current_exe)?.permissions();
        perms.set_mode(0o755);
        fs::set_permissions(&current_exe, perms)?;
    }

    // Cleanup
    fs::remove_dir_all(&temp_dir)?;

    println!();
    println!("{}", "✓ Update installed successfully!".green().bold());
    println!("{}", format!("Backup saved to: {}", backup).dimmed());
    println!(
        "{}",
        "Please restart the service for changes to take effect.".yellow()
    );

    Ok(())
}

/// Change panel port
pub async fn change_port() -> Result<()> {
    use crate::{db::DbClient, models::AllSetting};

    println!("{}", "Change Panel Port".cyan().bold());
    println!();

    print!("Enter new port [1-65535]: ");
    io::stdout().flush()?;

    let mut input = String::new();
    io::stdin().read_line(&mut input)?;

    let port: u16 = input.trim().parse().context("Invalid port number")?;

    if port < 1 || port > 65535 {
        println!("{}", "✗ Port must be between 1 and 65535".red());
        return Ok(());
    }

    let db = DbClient::init("rr-ui.db").await?;
    let mut settings: AllSetting = <AllSetting as SettingOps>::get(&db)
        .await?
        .unwrap_or_default();

    let old_port = settings.web_port;
    settings.web_port = port;
    settings.save(&db).await?;

    println!();
    println!(
        "{}",
        format!("✓ Port changed from {} to {}", old_port, port)
            .green()
            .bold()
    );
    println!(
        "{}",
        "Please restart the service for changes to take effect.".yellow()
    );

    Ok(())
}

/// Change panel path
pub async fn change_path() -> Result<()> {
    use crate::{db::DbClient, models::AllSetting};

    println!("{}", "Change Panel Path".cyan().bold());
    println!();

    print!("Enter new panel path (e.g., /admin): ");
    io::stdout().flush()?;

    let mut input = String::new();
    io::stdin().read_line(&mut input)?;

    let path = input.trim();

    if !path.starts_with('/') {
        println!("{}", "✗ Path must start with /".red());
        return Ok(());
    }

    let db = DbClient::init("rr-ui.db").await?;
    let mut settings: AllSetting = <AllSetting as SettingOps>::get(&db)
        .await?
        .unwrap_or_default();

    let old_path = settings.panel_secret_path.clone();
    settings.panel_secret_path = path.to_string();
    settings.save(&db).await?;

    println!();
    println!(
        "{}",
        format!("✓ Panel path changed from {} to {}", old_path, path)
            .green()
            .bold()
    );
    println!(
        "{}",
        "Please restart the service for changes to take effect.".yellow()
    );

    Ok(())
}

/// Reset admin credentials to defaults
pub async fn reset_admin_credentials() -> Result<()> {
    use crate::{db::DbClient, models::AllSetting, services::auth::hash_password};

    println!("{}", "Reset Admin Credentials".cyan().bold());
    println!();
    println!(
        "{}",
        "This will reset username to 'admin' and password to 'admin'".yellow()
    );
    print!("Continue? [y/N]: ");
    io::stdout().flush()?;

    let mut input = String::new();
    io::stdin().read_line(&mut input)?;

    if !input.trim().eq_ignore_ascii_case("y") {
        println!("{}", "Cancelled".dimmed());
        return Ok(());
    }

    let db = DbClient::init("rr-ui.db").await?;
    let mut settings: AllSetting = <AllSetting as SettingOps>::get(&db)
        .await?
        .unwrap_or_default();

    settings.username = "admin".to_string();
    settings.password_hash = hash_password("admin")?;
    settings.save(&db).await?;

    println!();
    println!(
        "{}",
        "✓ Admin credentials reset successfully".green().bold()
    );
    println!("Username: {}", "admin".yellow());
    println!("Password: {}", "admin".yellow());

    Ok(())
}

/// Interactive Certificate Management Menu
pub async fn cert_menu() -> Result<()> {
    use crate::db::DbClient;
    use crate::models::AllSetting;
    use crate::repositories::setting::SettingOps;
    use colored::Colorize;
    use std::io::{self, Write};
    use std::process::Command;

    loop {
        println!();
        println!("{}", "SSL Certificate Management".cyan().bold());
        println!("{}", "--------------------------".cyan());
        println!("1. Renew Certificates (Certbot)");
        println!("2. Renew Certificates (Dry-Run)");
        println!("3. Update Certificate Paths");
        println!("4. Revoke/Delete Certificates");
        println!("0. Back");
        println!();

        print!("Select option: ");
        io::stdout().flush()?;

        let mut input = String::new();
        io::stdin().read_line(&mut input)?;
        let choice = input.trim();

        match choice {
            "1" => {
                println!("{}", "Running Certbot Renewal...".cyan());
                let output = Command::new("certbot").arg("renew").output();
                match output {
                    Ok(out) => {
                        println!("{}", String::from_utf8_lossy(&out.stdout));
                        if !out.status.success() {
                            eprintln!("{}", String::from_utf8_lossy(&out.stderr).red());
                        }
                    }
                    Err(e) => eprintln!("{}", format!("Failed to run certbot: {}", e).red()),
                }
            }
            "2" => {
                println!("{}", "Running Certbot Renewal (Dry Run)...".cyan());
                let output = Command::new("certbot")
                    .args(&["renew", "--dry-run"])
                    .output();
                match output {
                    Ok(out) => {
                        println!("{}", String::from_utf8_lossy(&out.stdout));
                        if !out.status.success() {
                            eprintln!("{}", String::from_utf8_lossy(&out.stderr).red());
                        }
                    }
                    Err(e) => eprintln!("{}", format!("Failed to run certbot: {}", e).red()),
                }
            }
            "3" => {
                println!("{}", "Update Certificate Paths".cyan());
                print!("Enter absolute path to Certificate (.crt/.pem): ");
                io::stdout().flush()?;
                let mut cert = String::new();
                io::stdin().read_line(&mut cert)?;

                print!("Enter absolute path to Key (.key): ");
                io::stdout().flush()?;
                let mut key = String::new();
                io::stdin().read_line(&mut key)?;

                let cert = cert.trim();
                let key = key.trim();

                if !cert.is_empty() && !key.is_empty() {
                    let db = DbClient::init("rr-ui.db").await?;
                    let mut settings = <AllSetting as SettingOps>::get(&db)
                        .await?
                        .unwrap_or_default();
                    settings.web_cert_file = Some(cert.to_string());
                    settings.web_key_file = Some(key.to_string());
                    settings.save(&db).await?;
                    println!("{}", "✓ Paths updated. Restart required.".green());
                }
            }
            "4" => {
                println!(
                    "{}",
                    "WARNING: This will delete the certificate configuration from the panel."
                        .red()
                        .bold()
                );
                print!("Are you sure? [y/N]: ");
                io::stdout().flush()?;
                let mut confirm = String::new();
                io::stdin().read_line(&mut confirm)?;
                if confirm.trim().eq_ignore_ascii_case("y") {
                    let db = DbClient::init("rr-ui.db").await?;
                    let mut settings = <AllSetting as SettingOps>::get(&db)
                        .await?
                        .unwrap_or_default();
                    settings.web_cert_file = None;
                    settings.web_key_file = None;
                    settings.save(&db).await?;
                    println!("{}", "✓ Certificate configuration removed.".green());
                }
            }
            "0" => break,
            _ => println!("{}", "Invalid choice".red()),
        }
    }
    Ok(())
}
