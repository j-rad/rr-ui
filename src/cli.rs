use clap::{Args, Parser, Subcommand};

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Subcommand, Debug)]
pub enum Commands {
    /// Run the rr-ui web server
    Run,
    /// Manage settings
    Setting(SettingArgs),
    /// Manage SSL certificates
    Cert(CertArgs),
    /// Migrate from a legacy SQLite database
    Migrate(MigrateArgs),
    /// View or clear the IP ban list
    BanLog(BanLogArgs),
}

#[derive(Args, Debug)]
#[command(about = "Manage the IP ban list")]
pub struct BanLogArgs {
    /// Clear the ban list
    #[arg(long, short)]
    pub clear: bool,
}

#[derive(Args, Debug)]
#[command(about = "Update settings. Run with arguments to update, or --reset to reset.")]
pub struct SettingArgs {
    /// Update admin username.
    #[arg(long)]
    pub username: Option<String>,

    /// Update admin password.
    #[arg(long)]
    pub password: Option<String>,

    /// Update the binding port
    #[arg(long)]
    pub port: Option<u16>,

    /// Update the web base path
    #[arg(long)]
    pub web_base_path: Option<String>,

    /// Reset all settings to default. Conflicts with all other setting flags.
    #[arg(long, conflicts_with_all = &["username", "password", "port", "web_base_path", "disable_mfa"])]
    pub reset: bool,

    /// Disable Multi-Factor Authentication (MFA)
    #[arg(long)]
    pub disable_mfa: bool,

    /// Set the decoy site path (directory containing index.html)
    #[arg(long)]
    pub set_decoy: Option<String>,

    /// Set the secret path for the panel (e.g. /my-secret-panel)
    #[arg(long)]
    pub set_secret_path: Option<String>,
}

#[derive(Args, Debug)]
#[command(about = "Update SSL certificate paths")]
pub struct CertArgs {
    /// Path to the certificate file
    #[arg(long)]
    pub cert: String,
    /// Path to the key file
    #[arg(long)]
    pub key: String,
}

#[derive(Args, Debug)]
#[command(about = "Migrate data from a legacy SQLite database file")]
pub struct MigrateArgs {
    /// Path to the SQLite database file
    #[arg(long)]
    pub path: String,
}

pub mod advanced;
pub mod tui;
