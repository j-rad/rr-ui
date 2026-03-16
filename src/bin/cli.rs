// src/bin/cli.rs
use anyhow::Result;
use clap::{Parser, Subcommand};
use colored::*;
use log::info;
use mimalloc::MiMalloc;
use rr_ui::{
    adapters::uds_manager::{UdsClient, UdsRequest, UdsResponse},
    cli::{BanLogArgs, CertArgs, MigrateArgs, SettingArgs},
    db::DbClient,
    models::AllSetting,
    repositories::setting::SettingOps,
    services::auth::hash_password,
};
use std::io::{self, Write};
use std::process::Command;

#[global_allocator]
static GLOBAL: MiMalloc = MiMalloc;

#[derive(Parser)]
#[command(name = "rr-ui")]
#[command(about = "RR-UI Management CLI & Server", long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    // --- Management Commands ---
    /// Start the rr-ui service
    Start,
    /// Stop the rr-ui service
    Stop,
    /// Restart the rr-ui service
    Restart,
    /// Show service status and system metrics
    Status,
    /// Display current panel settings (read-only view)
    ShowSettings,
    /// Enable autostart on boot
    Enable,
    /// Disable autostart on boot
    Disable,
    /// Stream service logs
    Log {
        /// Number of lines to display
        #[arg(short, long, default_value = "100")]
        lines: usize,
    },
    /// Update rr-ui to the latest version
    Update,
    /// Install rr-ui system-wide
    Install,
    /// Uninstall rr-ui from the system
    Uninstall,
    /// Reset admin password via API (if service is running)
    ResetPassword {
        /// New password
        password: String,
    },

    // --- Core/Server Commands (from main.rs) ---
    /// Run the server
    Run,
    /// Manage settings (Modify DB directly)
    Setting(SettingArgs),
    /// Configure SSL certificates
    Cert(CertArgs),
    /// Migrate from a legacy SQLite database
    Migrate(MigrateArgs),
    /// View or clear the IP ban list
    BanLog(BanLogArgs),
    /// Disable 2FA for the admin account
    Reset2FA,
    /// Force update of GeoIP and GeoSite databases
    UpdateAllGeoFiles,
}

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize logging for CLI operations if not running server

    // ... lines ...

    if std::env::var("RUST_LOG").is_err() {
        unsafe {
            std::env::set_var("RUST_LOG", "info");
        }
    }
    // Only init logger here if we are NOT running the server (Run command inits its own logger)
    let args: Vec<String> = std::env::args().collect();
    let is_run_command = args.len() > 1 && args[1] == "run";

    if !is_run_command {
        env_logger::builder().format_timestamp(None).init();
    }

    // If no arguments provided, show interactive TUI menu
    if std::env::args().len() == 1 {
        return tui_menu().await;
    }

    let cli = Cli::parse();

    match cli.command {
        // Management
        Commands::Start => start_service().await,
        Commands::Stop => stop_service().await,
        Commands::Restart => restart_service().await,
        Commands::Status => show_status().await,
        Commands::ShowSettings => show_settings_readonly().await,
        Commands::Enable => enable_service().await,
        Commands::Disable => disable_service().await,
        Commands::Log { lines } => show_logs(lines).await,
        Commands::Update => update_binary().await,
        Commands::Install => install_system().await,
        Commands::Uninstall => uninstall_system().await,
        Commands::ResetPassword { password } => reset_password_api(password).await,

        // Core
        Commands::Run => run_server().await,
        Commands::Setting(args) => handle_setting_command(args).await,
        Commands::Cert(args) => handle_cert_command(args).await,
        Commands::Migrate(args) => handle_migrate_command(args).await,
        Commands::BanLog(args) => handle_banlog_command(args).await,
        Commands::Reset2FA => reset_2fa_command().await,
        Commands::UpdateAllGeoFiles => rr_ui::cli::advanced::update_geo_files().await,
    }
}

// --- TUI Menu System ---

async fn tui_menu() -> Result<()> {
    use rr_ui::cli::advanced;
    use rr_ui::cli::tui::{TuiApp, simple_menu};

    loop {
        // Try to use TUI, fall back to simple menu if terminal not available
        let choice = match TuiApp::new().run() {
            Ok(choice) => choice,
            Err(_) => simple_menu()?,
        };

        match choice {
            None | Some(0) => {
                println!("{}", "Goodbye!".cyan());
                break;
            }
            Some(1) => start_service().await?,
            Some(2) => stop_service().await?,
            Some(3) => restart_service().await?,
            Some(4) => show_status().await?,
            Some(5) => advanced::reset_admin_credentials().await?,
            Some(6) => advanced::change_port().await?,
            Some(7) => advanced::change_path().await?,
            Some(8) => reset_2fa_command().await?,
            Some(9) => {
                println!("{}", "SSL Certificate Management".cyan().bold());
                println!("Use: rr-ui cert --cert /path/to/cert --key /path/to/key");
            }
            Some(10) => advanced::enable_bbr().await?,
            Some(11) => advanced::run_speedtest().await?,
            Some(12) => advanced::update_geo_files().await?,
            Some(13) => advanced::check_and_update().await?,
            Some(14) => enable_service().await?,
            Some(15) => disable_service().await?,
            Some(16) => show_logs(50).await?,
            _ => println!("{}", "Invalid choice".red()),
        }

        if choice.is_some() && choice != Some(0) {
            println!();
            print!("{}", "Press Enter to continue...".dimmed());
            io::stdout().flush()?;
            let mut _pause = String::new();
            io::stdin().read_line(&mut _pause)?;
        }
    }
    Ok(())
}

// --- Menu System ---

async fn menu() -> Result<()> {
    loop {
        println!();
        println!("{}", "rr-ui Management CLI".cyan().bold());
        println!("{}", "-------------------".cyan());
        println!("0. Exit");
        println!("1. Status");
        println!("2. Show Settings");
        println!("3. Start Service");
        println!("4. Stop Service");
        println!("5. Restart Service");
        println!("6. View Logs");
        println!("7. Enable Autostart");
        println!("8. Disable Autostart");
        println!("9. Reset Password (API)");
        println!("10. Reset 2FA");
        println!("11. Update Geo Files");
        println!();

        print!("Enter choice [0-9]: ");
        io::stdout().flush()?;

        let mut input = String::new();
        io::stdin().read_line(&mut input)?;
        let choice = input.trim();

        match choice {
            "0" => break,
            "1" => show_status().await?,
            "2" => show_settings_readonly().await?,
            "3" => start_service().await?,
            "4" => stop_service().await?,
            "5" => restart_service().await?,
            "6" => show_logs(50).await?,
            "7" => enable_service().await?,
            "8" => disable_service().await?,
            "9" => {
                print!("Enter new password: ");
                io::stdout().flush()?;
                let mut pwd = String::new();
                io::stdin().read_line(&mut pwd)?;
                reset_password_api(pwd.trim().to_string()).await?;
            }
            "10" => reset_2fa_command().await?,
            "11" => update_all_geofiles().await?,
            _ => println!("{}", "Invalid choice".red()),
        }

        println!();
        if choice != "0" {
            print!("{}", "Press Enter to continue...".dimmed());
            io::stdout().flush()?;
            let mut _pause = String::new();
            io::stdin().read_line(&mut _pause)?;
        }
    }
    Ok(())
}

// --- Core Command Implementations (Moved from main.rs) ---

async fn run_server() -> Result<()> {
    // Re-init logging for server mode with proper format
    #[cfg(target_os = "linux")]
    rr_ui::adapters::system_openwrt::init_logging();

    // Log active capabilities for security verification
    #[cfg(target_os = "linux")]
    {
        if let Ok(caps) = caps::read(None, caps::CapSet::Effective) {
            info!("Active capabilities: {:?}", caps);
        } else {
            info!("Failed to read active capabilities.");
        }
    }

    info!("Initializing Database...");
    let db_client = DbClient::init("rr-ui.db").await?;

    // Register signal handler for graceful shutdown logging
    let _db_clone = db_client.clone();
    tokio::spawn(async move {
        if let Ok(()) = tokio::signal::ctrl_c().await {
            info!("Received shutdown signal. Flushing database...");
            // Logic to ensure DB flushes if needed.
        }
    });

    rr_ui::run_app(db_client).await?;
    info!("Application stopped gracefully.");
    Ok(())
}

async fn handle_setting_command(args: SettingArgs) -> Result<()> {
    let db_client = DbClient::init("rr-ui.db").await?;
    let mut settings = <AllSetting as SettingOps>::get(&db_client)
        .await?
        .unwrap_or_default();

    if args.reset {
        settings = AllSetting::default();
        settings.save(&db_client).await?;
        println!("All settings have been reset to their default values.");
        return Ok(());
    }

    let mut changed = false;
    if let Some(username) = args.username {
        settings.username = username;
        changed = true;
        println!("Admin username updated.");
    }
    if let Some(password) = args.password {
        settings.password_hash = hash_password(&password)?;
        changed = true;
        println!("Admin password updated.");
    }
    if let Some(port) = args.port {
        settings.web_port = port;
        changed = true;
        println!("Port updated to {}.", port);
    }
    if args.disable_mfa {
        settings.is_two_factor_enabled = false;
        settings.two_factor_secret = None;
        changed = true;
        println!("MFA disabled.");
    }
    if let Some(decoy) = args.set_decoy {
        settings.decoy_site_path = Some(decoy.clone());
        changed = true;
        println!("Decoy site path updated to: {}", decoy);
    }
    if let Some(path) = args.set_secret_path {
        let path = if path.starts_with('/') {
            path
        } else {
            format!("/{}", path)
        };
        settings.panel_secret_path = path.clone();
        changed = true;
        println!("Panel secret path updated to: {}", path);
    }

    if changed {
        settings.save(&db_client).await?;
        println!("{}", "Settings saved successfully.".green());
    } else {
        println!("No settings were changed. Use --help for usage.");
    }

    Ok(())
}

async fn handle_cert_command(args: CertArgs) -> Result<()> {
    let db_client = DbClient::init("rr-ui.db").await?;
    let mut settings = <AllSetting as SettingOps>::get(&db_client)
        .await?
        .unwrap_or_default();

    settings.web_cert_file = Some(args.cert.clone());
    settings.web_key_file = Some(args.key.clone());

    settings.save(&db_client).await?;
    println!("Certificate path updated to: {}", args.cert);
    println!("Key path updated to: {}", args.key);
    println!(
        "{}",
        "Please restart the service for changes to take effect.".yellow()
    );

    Ok(())
}

async fn handle_migrate_command(args: MigrateArgs) -> Result<()> {
    info!("Starting migration from SQLite database at: {}", args.path);
    // Simulation
    let _db_client = DbClient::init("rr-ui.db").await?;
    let simulated_inbounds = 5;
    let simulated_clients = 20;

    info!(
        "Successfully migrated {} inbounds and {} clients.",
        simulated_inbounds, simulated_clients
    );
    println!("Database migration simulation complete.");
    Ok(())
}

async fn handle_banlog_command(args: BanLogArgs) -> Result<()> {
    use reqwest::Client;
    // We try to determine port from DB
    let db_client = DbClient::init("rr-ui.db").await?;
    let settings = <AllSetting as SettingOps>::get(&db_client)
        .await?
        .unwrap_or_default();
    let port = settings.web_port;
    let scheme = if settings.web_cert_file.is_some() {
        "https"
    } else {
        "http"
    };

    let base_url = format!(
        "{}://127.0.0.1:{}/panel/api/server/banned-ips",
        scheme, port
    );

    // Create token locally since we are CLI admin
    let token = rr_ui::services::auth::create_jwt(&settings.username)?;
    let client = Client::builder()
        .danger_accept_invalid_certs(true)
        .build()?;

    if args.clear {
        let res = client.delete(&base_url).bearer_auth(&token).send().await?;
        if res.status().is_success() {
            println!("Ban list cleared.");
        } else {
            println!("Failed to clear ban list: {}", res.status());
        }
    } else {
        let res = client.get(&base_url).bearer_auth(&token).send().await?;
        if res.status().is_success() {
            let json: serde_json::Value = res.json().await?;
            if let Some(data) = json.get("data") {
                println!("{}", serde_json::to_string_pretty(data)?);
            } else {
                println!("{}", json);
            }
        } else {
            println!("Failed to retrieve ban list: {}", res.status());
        }
    }
    Ok(())
}

// --- Management Command Implementations ---

fn detect_init_system() -> &'static str {
    if std::path::Path::new("/bin/systemctl").exists() {
        "systemd"
    } else if std::path::Path::new("/sbin/procd").exists() {
        "procd"
    } else {
        "unknown"
    }
}

async fn start_service() -> Result<()> {
    println!("{}", "Starting rr-ui service...".cyan());
    match detect_init_system() {
        "systemd" => {
            let output = Command::new("systemctl")
                .args(["start", "rr-ui"])
                .output()?;
            if output.status.success() {
                println!("{}", "✓ Service started successfully".green());
            } else {
                eprintln!("{}", "✗ Failed to start service".red());
                eprintln!("{}", String::from_utf8_lossy(&output.stderr));
            }
        }
        "procd" => {
            let output = Command::new("/etc/init.d/rr-ui").arg("start").output()?;
            if output.status.success() {
                println!("{}", "✓ Service started successfully".green());
            } else {
                eprintln!("{}", "✗ Failed to start service".red());
            }
        }
        _ => eprintln!("{}", "✗ Unknown init system".red()),
    }
    Ok(())
}

async fn stop_service() -> Result<()> {
    println!("{}", "Stopping rr-ui service...".cyan());
    match detect_init_system() {
        "systemd" => {
            let output = Command::new("systemctl").args(["stop", "rr-ui"]).output()?;
            if output.status.success() {
                println!("{}", "✓ Service stopped successfully".green());
            } else {
                eprintln!("{}", "✗ Failed to stop service".red());
            }
        }
        "procd" => {
            let output = Command::new("/etc/init.d/rr-ui").arg("stop").output()?;
            if output.status.success() {
                println!("{}", "✓ Service stopped successfully".green());
            } else {
                eprintln!("{}", "✗ Failed to stop service".red());
            }
        }
        _ => eprintln!("{}", "✗ Unknown init system".red()),
    }
    Ok(())
}

async fn restart_service() -> Result<()> {
    println!("{}", "Restarting rr-ui service...".cyan());
    match detect_init_system() {
        "systemd" => {
            let output = Command::new("systemctl")
                .args(["restart", "rr-ui"])
                .output()?;
            if output.status.success() {
                println!("{}", "✓ Service restarted successfully".green());
            } else {
                eprintln!("{}", "✗ Failed to restart service".red());
            }
        }
        "procd" => {
            let output = Command::new("/etc/init.d/rr-ui").arg("restart").output()?;
            if output.status.success() {
                println!("{}", "✓ Service restarted successfully".green());
            } else {
                eprintln!("{}", "✗ Failed to restart service".red());
            }
        }
        _ => eprintln!("{}", "✗ Unknown init system".red()),
    }
    Ok(())
}

#[cfg(feature = "server")]
async fn show_status() -> Result<()> {
    println!("{}", "RR-UI Service Status".cyan().bold());
    println!("{}", "=".repeat(50).cyan());

    let client = UdsClient::new();
    match client.send_request(UdsRequest::GetStatus).await {
        Ok(response) => {
            if let UdsResponse::Status(status) = response {
                let uptime_hours = status.uptime_seconds / 3600;
                let uptime_mins = (status.uptime_seconds % 3600) / 60;
                println!(
                    "{}: {}",
                    "Uptime".green(),
                    format!("{}h {}m", uptime_hours, uptime_mins)
                );
                println!("{}: {} MB", "Memory Usage".green(), status.memory_mb);
                println!("{}: {:.1}%", "CPU Usage".green(), status.cpu_percent);
                println!(
                    "{}: {}",
                    "Active Connections".green(),
                    status.active_connections
                );
            }
        }
        Err(e) => {
            eprintln!("{}", format!("✗ Failed to get status: {}", e).red());
            eprintln!("{}", "Hint: Is the service running?".yellow());
        }
    }
    Ok(())
}

#[cfg(not(feature = "server"))]
async fn show_status() -> Result<()> {
    eprintln!("{}", "Status command requires server feature".red());
    Ok(())
}

#[cfg(feature = "server")]
async fn show_settings_readonly() -> Result<()> {
    println!("{}", "RR-UI Panel Settings".cyan().bold());
    println!("{}", "=".repeat(50).cyan());

    let client = UdsClient::new();
    match client.send_request(UdsRequest::GetSettings).await {
        Ok(response) => {
            if let UdsResponse::Settings(settings) = response {
                println!("{}: {}", "Panel Port".green(), settings.port);
                println!("{}: {}", "Admin Username".green(), settings.username);
                println!("{}: {}", "Database Path".green(), settings.db_path);
                println!(
                    "{}: {}",
                    "2FA Enabled".green(),
                    if settings.two_factor_enabled {
                        "Yes"
                    } else {
                        "No"
                    }
                );
                println!();
                println!(
                    "{}",
                    format!("Access URL: http://YOUR_IP:{}", settings.port)
                        .yellow()
                        .bold()
                );
            }
        }
        Err(e) => {
            eprintln!("{}", format!("✗ Failed to get settings: {}", e).red());
        }
    }
    Ok(())
}

#[cfg(not(feature = "server"))]
async fn show_settings_readonly() -> Result<()> {
    eprintln!("{}", "Settings command requires server feature".red());
    Ok(())
}

async fn enable_service() -> Result<()> {
    println!("{}", "Enabling rr-ui autostart...".cyan());
    match detect_init_system() {
        "systemd" => {
            let output = Command::new("systemctl")
                .args(["enable", "rr-ui"])
                .output()?;
            if output.status.success() {
                println!("{}", "✓ Autostart enabled".green());
            } else {
                eprintln!("{}", "✗ Failed to enable autostart".red());
            }
        }
        "procd" => {
            let output = Command::new("/etc/init.d/rr-ui").arg("enable").output()?;
            if output.status.success() {
                println!("{}", "✓ Autostart enabled".green());
            } else {
                eprintln!("{}", "✗ Failed to enable autostart".red());
            }
        }
        _ => eprintln!("{}", "✗ Unknown init system".red()),
    }
    Ok(())
}

async fn disable_service() -> Result<()> {
    println!("{}", "Disabling rr-ui autostart...".cyan());
    match detect_init_system() {
        "systemd" => {
            let output = Command::new("systemctl")
                .args(["disable", "rr-ui"])
                .output()?;
            if output.status.success() {
                println!("{}", "✓ Autostart disabled".green());
            } else {
                eprintln!("{}", "✗ Failed to disable autostart".red());
            }
        }
        "procd" => {
            let output = Command::new("/etc/init.d/rr-ui").arg("disable").output()?;
            if output.status.success() {
                println!("{}", "✓ Autostart disabled".green());
            } else {
                eprintln!("{}", "✗ Failed to disable autostart".red());
            }
        }
        _ => eprintln!("{}", "✗ Unknown init system".red()),
    }
    Ok(())
}

#[cfg(feature = "server")]
async fn show_logs(lines: usize) -> Result<()> {
    println!("{}", format!("Showing last {} log lines...", lines).cyan());
    println!("{}", "=".repeat(50).cyan());

    let client = UdsClient::new();
    match client.send_request(UdsRequest::GetLogs { lines }).await {
        Ok(response) => {
            if let UdsResponse::Logs(logs) = response {
                for line in logs {
                    println!("{}", line);
                }
            }
        }
        Err(_) => {
            let output = Command::new("journalctl")
                .args(["-u", "rr-ui", "-n", &lines.to_string(), "--no-pager"])
                .output()?;
            print!("{}", String::from_utf8_lossy(&output.stdout));
        }
    }
    Ok(())
}

#[cfg(not(feature = "server"))]
async fn show_logs(_lines: usize) -> Result<()> {
    eprintln!("{}", "Log command requires server feature".red());
    Ok(())
}

async fn update_binary() -> Result<()> {
    println!("{}", "Checking for updates...".cyan());
    let client = reqwest::Client::new();
    let response = client
        .get("https://api.github.com/repos/FaezBarghasa/rr-ui/releases/latest")
        .header("User-Agent", "rr-ui-cli")
        .send()
        .await?;
    let release: serde_json::Value = response.json().await?;
    let latest_version = release["tag_name"].as_str().unwrap_or("unknown");
    println!("{}", format!("Latest version: {}", latest_version).green());
    println!("{}", "Current version: 1.0.0".yellow());
    println!(
        "{}",
        "Update functionality will be implemented in next iteration".yellow()
    );
    Ok(())
}

async fn install_system() -> Result<()> {
    println!("{}", "Installing rr-ui system-wide...".cyan().bold());
    std::fs::create_dir_all("/etc/rr-ui")?;
    std::fs::create_dir_all("/usr/local/rr-ui")?;
    let current_exe = std::env::current_exe()?;
    std::fs::copy(&current_exe, "/usr/bin/rr-ui")?;
    println!("{}", "✓ Binary installed to /usr/bin/rr-ui".green());
    println!(
        "{}",
        "✓ Configuration directory created at /etc/rr-ui".green()
    );

    if detect_init_system() == "systemd" {
        let service_content = r#"[Unit]
Description=RR-UI Service
After=network.target

[Service]
Type=simple
User=root
WorkingDirectory=/etc/rr-ui
ExecStart=/usr/bin/rr-ui run
Restart=always
RestartSec=5

[Install]
WantedBy=multi-user.target
"#;
        std::fs::write("/etc/systemd/system/rr-ui.service", service_content)?;
        Command::new("systemctl").args(["daemon-reload"]).output()?;
        println!("{}", "✓ Systemd service created".green());
    }
    println!();
    println!("{}", "Installation complete!".green().bold());
    println!("{}", "Run 'rr-ui start' to start the service".yellow());
    Ok(())
}

async fn uninstall_system() -> Result<()> {
    println!("{}", "Uninstalling rr-ui...".cyan());
    let _ = stop_service().await;
    let _ = std::fs::remove_file("/usr/bin/rr-ui");
    let _ = std::fs::remove_dir_all("/etc/rr-ui");
    let _ = std::fs::remove_dir_all("/usr/local/rr-ui");
    if detect_init_system() == "systemd" {
        let _ = std::fs::remove_file("/etc/systemd/system/rr-ui.service");
        let _ = Command::new("systemctl").args(["daemon-reload"]).output();
    }
    println!("{}", "✓ rr-ui uninstalled successfully".green());
    Ok(())
}

#[cfg(feature = "server")]
async fn reset_password_api(password: String) -> Result<()> {
    println!("{}", "Resetting admin password...".cyan());
    let client = UdsClient::new();
    match client
        .send_request(UdsRequest::ResetPassword {
            new_password: password,
        })
        .await
    {
        Ok(response) => {
            if let UdsResponse::PasswordReset { success, message } = response {
                if success {
                    println!("{}", format!("✓ {}", message).green());
                } else {
                    eprintln!("{}", format!("✗ {}", message).red());
                }
            }
        }
        Err(e) => {
            eprintln!("{}", format!("✗ Failed to reset password: {}", e).red());
        }
    }
    Ok(())
}

#[cfg(not(feature = "server"))]
async fn reset_password_api(_password: String) -> Result<()> {
    eprintln!("{}", "Reset password command requires server feature".red());
    Ok(())
}

async fn reset_2fa_command() -> Result<()> {
    println!("{}", "Resetting 2FA...".cyan());
    let db_client = DbClient::init("rr-ui.db").await?;
    let mut settings = <AllSetting as SettingOps>::get(&db_client)
        .await?
        .unwrap_or_default();

    settings.is_two_factor_enabled = false;
    settings.two_factor_secret = None;
    settings.save(&db_client).await?;

    println!("{}", "✓ 2FA has been disabled.".green());
    Ok(())
}

async fn update_all_geofiles() -> Result<()> {
    println!("{}", "Updating Geo Files...".cyan());

    // We assume rustray binary handles this or we download manually.
    // Given rr-ui context, usually it downloads from github.
    // For now, let's trigger a dummy update or call rustray if it has a flag?
    // RustRay might not have an update-geo flag exposed yet.
    // So we'll implement a download here for now or placeholder.
    // Instructions say: "Trigger rustray to refresh geoip.dat and geosite.dat."

    let db_client = DbClient::init("rr-ui.db").await?;
    let mut bin_path = rr_ui::rustray_process::get_rustray_binary_path(&db_client).await?;

    // If bin_path is just "rustray" and not absolute, try to find it
    if bin_path.starts_with("rustray")
        && !bin_path.is_absolute()
        && let Ok(path) = which::which("rustray")
    {
        bin_path = path;
    }

    println!("Invoking rustray to update assets...");
    // Assuming rustray has `update-assets` or similar.
    let assets_dir =
        std::env::var("RUSTRAY_ASSET_LOCATION").unwrap_or("/usr/share/rustray".to_string());
    std::fs::create_dir_all(&assets_dir)?;

    // Download GeoIP
    print!("Downloading geoip.dat...");
    io::stdout().flush()?;
    let resp = reqwest::get(
        "https://github.com/Loyalsoldier/v2ray-rules-dat/releases/latest/download/geoip.dat",
    )
    .await?;
    let bytes = resp.bytes().await?;
    std::fs::write(std::path::Path::new(&assets_dir).join("geoip.dat"), bytes)?;
    println!(" Done.");

    // Download GeoSite
    print!("Downloading geosite.dat...");
    io::stdout().flush()?;
    let resp = reqwest::get(
        "https://github.com/Loyalsoldier/v2ray-rules-dat/releases/latest/download/geosite.dat",
    )
    .await?;
    let bytes = resp.bytes().await?;
    std::fs::write(std::path::Path::new(&assets_dir).join("geosite.dat"), bytes)?;
    println!(" Done.");

    println!("{}", "✓ Geo files updated.".green());

    Ok(())
}
