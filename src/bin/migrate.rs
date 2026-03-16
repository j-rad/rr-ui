use anyhow::{Context, Result};
use clap::Parser;
use rr_ui::models::{
    Client, ClientTraffic, Inbound, InboundProtocol, ProtocolSettings, Sniffing, StreamSettings,
    TrojanSettings, VlessSettings, VmessSettings,
};
use rusqlite::Connection;
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
use std::collections::HashSet;
use std::path::PathBuf;

use surrealdb::Surreal;
use surrealdb::engine::remote::ws::Ws;

#[derive(Parser)]
#[command(name = "migrate")]
#[command(about = "Migrate from legacy X-UI SQLite DB to RustRay SurrealDB")]
struct Cli {
    #[arg(long, help = "Path to legacy x-ui.db")]
    from_x_ui: PathBuf,

    #[arg(long, help = "URL of target SurrealDB (e.g. ws://localhost:8000)")]
    to_surreal: String,

    #[arg(long, default_value = "xui", help = "SurrealDB namespace")]
    namespace: String,

    #[arg(long, default_value = "panel", help = "SurrealDB database name")]
    database: String,

    #[arg(long, default_value_t = false, help = "Dry-run mode (no writes)")]
    dry_run: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct LegacyInbound {
    id: i64,
    up: i64,
    down: i64,
    total: i64,
    remark: String,
    enable: bool,
    expiry_time: i64,
    port: i64,
    protocol: String,
    settings: String,
    stream_settings: String,
    tag: String,
    sniffing: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct LegacyClientTraffic {
    id: i64,
    inbound_id: i64,
    enable: bool,
    email: String,
    up: i64,
    down: i64,
    expiry_time: i64,
    total: i64,
}

/// Tracks migration progress for reporting and streaming to the UI wizard.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MigrationProgress {
    pub phase: String,
    pub current: usize,
    pub total: usize,
    pub success_count: usize,
    pub fail_count: usize,
    pub skipped_duplicates: usize,
    pub total_users: usize,
    pub errors: Vec<String>,
}

impl MigrationProgress {
    fn new(phase: &str, total: usize) -> Self {
        Self {
            phase: phase.to_string(),
            current: 0,
            total,
            success_count: 0,
            fail_count: 0,
            skipped_duplicates: 0,
            total_users: 0,
            errors: Vec::new(),
        }
    }

    fn percent(&self) -> f64 {
        if self.total == 0 {
            100.0
        } else {
            (self.current as f64 / self.total as f64) * 100.0
        }
    }
}

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();

    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    println!("🔄 RustRay Migration Tool");
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    println!("  Source:    {:?}", cli.from_x_ui);
    println!("  Target:    {}", cli.to_surreal);
    println!("  Namespace: {}", cli.namespace);
    println!("  Database:  {}", cli.database);
    if cli.dry_run {
        println!("  ⚠ DRY-RUN MODE — no data will be written");
    }
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━\n");

    // Connect to SQLite
    let conn = Connection::open(&cli.from_x_ui).context("Failed to open SQLite database")?;

    // Connect to SurrealDB
    let db = Surreal::new::<Ws>(&cli.to_surreal)
        .await
        .context("Failed to connect to SurrealDB")?;

    db.use_ns(&cli.namespace)
        .use_db(&cli.database)
        .await
        .context("Failed to select DB namespace")?;

    // ── Phase 1: Migrate inbounds ──────────────────────────────────
    let inbound_progress = migrate_inbounds(&conn, &db, cli.dry_run).await?;

    // ── Phase 2: Migrate client_traffics ───────────────────────────
    let traffic_progress = migrate_client_traffics(&conn, &db, cli.dry_run).await?;

    // ── Summary ────────────────────────────────────────────────────
    println!("\n━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    println!("📊 Migration Summary");
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    println!(
        "  Inbounds:        {} migrated, {} failed, {} duplicate-skipped",
        inbound_progress.success_count,
        inbound_progress.fail_count,
        inbound_progress.skipped_duplicates
    );
    println!("  Users in config: {}", inbound_progress.total_users);
    println!(
        "  Client Traffic:  {} migrated, {} failed, {} duplicate-skipped",
        traffic_progress.success_count,
        traffic_progress.fail_count,
        traffic_progress.skipped_duplicates
    );
    if !inbound_progress.errors.is_empty() || !traffic_progress.errors.is_empty() {
        println!("\n⚠ Errors:");
        for e in &inbound_progress.errors {
            println!("  • {e}");
        }
        for e in &traffic_progress.errors {
            println!("  • {e}");
        }
    }
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━\n");

    Ok(())
}

// ═══════════════════════════════════════════════════════════════════
// Phase 1 — Inbounds
// ═══════════════════════════════════════════════════════════════════

async fn migrate_inbounds(
    conn: &Connection,
    db: &Surreal<surrealdb::engine::remote::ws::Client>,
    dry_run: bool,
) -> Result<MigrationProgress> {
    println!("📦 Phase 1: Migrating inbounds…");

    let mut stmt = conn.prepare(
        "SELECT id, up, down, total, remark, enable, expiry_time, \
         port, protocol, settings, stream_settings, tag, sniffing \
         FROM inbounds",
    )?;

    let rows: Vec<LegacyInbound> = stmt
        .query_map([], |row| {
            Ok(LegacyInbound {
                id: row.get(0)?,
                up: row.get(1)?,
                down: row.get(2)?,
                total: row.get(3)?,
                remark: row.get(4)?,
                enable: row.get(5)?,
                expiry_time: row.get(6)?,
                port: row.get(7)?,
                protocol: row.get(8)?,
                settings: row.get(9)?,
                stream_settings: row.get(10)?,
                tag: row.get(11)?,
                sniffing: row.get(12)?,
            })
        })?
        .filter_map(|r| r.ok())
        .collect();

    let mut progress = MigrationProgress::new("inbounds", rows.len());

    // Collect existing tags to detect duplicates
    let mut seen_tags: HashSet<String> = HashSet::new();

    // Query existing tags from SurrealDB to detect cross-run duplicates
    let existing: Vec<serde_json::Value> = db
        .query("SELECT tag FROM inbound")
        .await
        .ok()
        .and_then(|mut r| r.take::<Vec<serde_json::Value>>(0).ok())
        .unwrap_or_default();

    for entry in &existing {
        if let Some(tag) = entry.get("tag").and_then(|v| v.as_str()) {
            seen_tags.insert(tag.to_string());
        }
    }

    for legacy in rows {
        progress.current += 1;

        // Duplicate UUID / tag guard
        if seen_tags.contains(&legacy.tag) {
            println!(
                "  ⏭ Skipping duplicate inbound tag='{}' (remark='{}')",
                legacy.tag, legacy.remark
            );
            progress.skipped_duplicates += 1;
            continue;
        }

        match convert_inbound(legacy.clone()) {
            Ok(model) => {
                if let Some(clients) = model.settings.clients() {
                    progress.total_users += clients.len();

                    // Check for duplicate client UUIDs within this inbound
                    let mut client_ids: HashSet<String> = HashSet::new();
                    for client in clients {
                        if let Some(ref id) = client.id {
                            if !client_ids.insert(id.clone()) {
                                let msg = format!(
                                    "Duplicate client UUID {id} in inbound '{}'",
                                    legacy.remark
                                );
                                println!("  ⚠ {msg}");
                                progress.errors.push(msg);
                            }
                        }
                    }
                }

                if dry_run {
                    println!("  ✓ [dry-run] Would migrate inbound: {}", legacy.remark);
                    progress.success_count += 1;
                } else {
                    let result: surrealdb::Result<Option<Inbound<'static>>> =
                        db.create("inbound").content(model).await;
                    match result {
                        Ok(_) => {
                            println!("  ✓ Migrated inbound: {}", legacy.remark);
                            progress.success_count += 1;
                            seen_tags.insert(legacy.tag);
                        }
                        Err(e) => {
                            let msg = format!("Failed to insert inbound '{}': {e}", legacy.remark);
                            eprintln!("  ❌ {msg}");
                            progress.errors.push(msg);
                            progress.fail_count += 1;
                        }
                    }
                }
            }
            Err(e) => {
                let msg = format!("Failed to convert inbound '{}': {e}", legacy.remark);
                eprintln!("  ❌ {msg}");
                progress.errors.push(msg);
                progress.fail_count += 1;
            }
        }

        if progress.current % 10 == 0 {
            println!(
                "  … {:.0}% ({}/{})",
                progress.percent(),
                progress.current,
                progress.total
            );
        }
    }

    println!(
        "  ✅ Phase 1 complete: {} ok, {} failed, {} skipped\n",
        progress.success_count, progress.fail_count, progress.skipped_duplicates
    );
    Ok(progress)
}

// ═══════════════════════════════════════════════════════════════════
// Phase 2 — Client Traffics
// ═══════════════════════════════════════════════════════════════════

async fn migrate_client_traffics(
    conn: &Connection,
    db: &Surreal<surrealdb::engine::remote::ws::Client>,
    dry_run: bool,
) -> Result<MigrationProgress> {
    println!("📦 Phase 2: Migrating client_traffics…");

    // Check if table exists in the legacy DB
    let table_exists: bool = conn
        .prepare("SELECT name FROM sqlite_master WHERE type='table' AND name='client_traffics'")
        .and_then(|mut s| s.query_row([], |_| Ok(true)))
        .unwrap_or(false);

    if !table_exists {
        println!("  ℹ No client_traffics table found, skipping phase 2.");
        return Ok(MigrationProgress::new("client_traffics", 0));
    }

    let mut stmt = conn.prepare(
        "SELECT id, inbound_id, enable, email, up, down, expiry_time, total \
         FROM client_traffics",
    )?;

    let rows: Vec<LegacyClientTraffic> = stmt
        .query_map([], |row| {
            Ok(LegacyClientTraffic {
                id: row.get(0)?,
                inbound_id: row.get(1)?,
                enable: row.get(2)?,
                email: row.get(3)?,
                up: row.get(4)?,
                down: row.get(5)?,
                expiry_time: row.get(6)?,
                total: row.get(7)?,
            })
        })?
        .filter_map(|r| r.ok())
        .collect();

    let mut progress = MigrationProgress::new("client_traffics", rows.len());

    // Query existing emails to detect duplicates
    let mut seen_emails: HashSet<String> = HashSet::new();
    let existing: Vec<serde_json::Value> = db
        .query("SELECT email FROM client_traffic")
        .await
        .ok()
        .and_then(|mut r| r.take::<Vec<serde_json::Value>>(0).ok())
        .unwrap_or_default();

    for entry in &existing {
        if let Some(email) = entry.get("email").and_then(|v| v.as_str()) {
            seen_emails.insert(email.to_string());
        }
    }

    for legacy in rows {
        progress.current += 1;

        // Duplicate email guard — client_traffics keyed on email
        if seen_emails.contains(&legacy.email) {
            println!(
                "  ⏭ Skipping duplicate client_traffic email='{}'",
                legacy.email
            );
            progress.skipped_duplicates += 1;
            continue;
        }

        let traffic = ClientTraffic {
            id: Some(legacy.id),
            inbound_id: Some(legacy.inbound_id),
            enable: legacy.enable,
            email: legacy.email.clone(),
            up: legacy.up,
            down: legacy.down,
            expiry_time: legacy.expiry_time,
            total: legacy.total,
        };

        if dry_run {
            println!("  ✓ [dry-run] Would migrate traffic for: {}", legacy.email);
            progress.success_count += 1;
        } else {
            let result: surrealdb::Result<Option<ClientTraffic>> =
                db.create("client_traffic").content(traffic).await;
            match result {
                Ok(_) => {
                    println!("  ✓ Migrated traffic for: {}", legacy.email);
                    progress.success_count += 1;
                    seen_emails.insert(legacy.email);
                }
                Err(e) => {
                    let msg = format!("Failed to insert client_traffic '{}': {e}", legacy.email);
                    eprintln!("  ❌ {msg}");
                    progress.errors.push(msg);
                    progress.fail_count += 1;
                }
            }
        }

        if progress.current % 25 == 0 {
            println!(
                "  … {:.0}% ({}/{})",
                progress.percent(),
                progress.current,
                progress.total
            );
        }
    }

    println!(
        "  ✅ Phase 2 complete: {} ok, {} failed, {} skipped\n",
        progress.success_count, progress.fail_count, progress.skipped_duplicates
    );
    Ok(progress)
}

// ═══════════════════════════════════════════════════════════════════
// Inbound Conversion
// ═══════════════════════════════════════════════════════════════════

fn convert_inbound(legacy: LegacyInbound) -> Result<Inbound<'static>> {
    let settings_json: Value =
        serde_json::from_str(&legacy.settings).context("Failed to parse settings JSON")?;
    let stream_json: Value = serde_json::from_str(&legacy.stream_settings)
        .context("Failed to parse stream_settings JSON")?;
    let sniffing_json: Value = serde_json::from_str(&legacy.sniffing)
        .unwrap_or(json!({ "enabled": true, "destOverride": ["http", "tls"] }));

    let protocol = legacy.protocol.as_str();

    let settings = match protocol {
        "vless" => {
            let clients_val = settings_json.get("clients").cloned().unwrap_or(json!([]));
            let clients: Vec<Client> = serde_json::from_value(clients_val).unwrap_or_default();
            ProtocolSettings::Vless(VlessSettings {
                clients,
                decryption: Some(
                    settings_json
                        .get("decryption")
                        .and_then(|v| v.as_str())
                        .unwrap_or("none")
                        .to_string()
                        .into(),
                ),
                fallbacks: settings_json.get("fallbacks").cloned(),
                extra: std::collections::HashMap::new(),
            })
        }
        "vmess" => {
            let clients_val = settings_json.get("clients").cloned().unwrap_or(json!([]));
            let clients: Vec<Client> = serde_json::from_value(clients_val).unwrap_or_default();
            ProtocolSettings::Vmess(VmessSettings {
                clients,
                fallbacks: settings_json.get("fallbacks").cloned(),
                password: None,
                extra: std::collections::HashMap::new(),
            })
        }
        "trojan" => {
            let clients_val = settings_json.get("clients").cloned().unwrap_or(json!([]));
            let clients: Vec<Client> = serde_json::from_value(clients_val).unwrap_or_default();
            ProtocolSettings::Trojan(TrojanSettings {
                clients,
                fallbacks: settings_json.get("fallbacks").cloned(),
                password: None,
                extra: std::collections::HashMap::new(),
            })
        }
        "shadowsocks" => {
            ProtocolSettings::Shadowsocks(serde_json::from_value(settings_json).unwrap_or_default())
        }
        _ => return Err(anyhow::anyhow!("Unsupported protocol: {}", protocol)),
    };

    // Parse StreamSettings
    let network = stream_json
        .get("network")
        .and_then(|v| v.as_str())
        .unwrap_or("tcp")
        .to_string();
    let security = stream_json
        .get("security")
        .and_then(|v| v.as_str())
        .unwrap_or("none")
        .to_string();

    let mut stream_settings = StreamSettings {
        network: network.clone().into(),
        security: security.clone().into(),
        ..Default::default()
    };

    match network.as_str() {
        "ws" => {
            if let Some(ws_set) = stream_json.get("wsSettings") {
                stream_settings.ws_settings = serde_json::from_value(ws_set.clone()).ok();
            }
        }
        "tcp" => {
            if let Some(tcp_set) = stream_json.get("tcpSettings") {
                stream_settings.tcp_settings = serde_json::from_value(tcp_set.clone()).ok();
            }
        }
        "grpc" => {
            if let Some(grpc_set) = stream_json.get("grpcSettings") {
                stream_settings.grpc_settings = serde_json::from_value(grpc_set.clone()).ok();
            }
        }
        "http" => {
            if let Some(http_set) = stream_json.get("httpSettings") {
                stream_settings.http_settings = serde_json::from_value(http_set.clone()).ok();
            }
        }
        "kcp" => {
            if let Some(kcp_set) = stream_json.get("kcpSettings") {
                stream_settings.kcp_settings = serde_json::from_value(kcp_set.clone()).ok();
            }
        }
        _ => {}
    }

    match security.as_str() {
        "tls" => {
            if let Some(tls_set) = stream_json.get("tlsSettings") {
                stream_settings.tls_settings = serde_json::from_value(tls_set.clone()).ok();
            }
        }
        "reality" => {
            if let Some(reality_set) = stream_json.get("realitySettings") {
                stream_settings.reality_settings = serde_json::from_value(reality_set.clone()).ok();
            }
        }
        _ => {}
    }

    let sniffing: Sniffing = serde_json::from_value(sniffing_json).unwrap_or_default();

    Ok(Inbound {
        id: None,
        up_bytes: legacy.up,
        down_bytes: legacy.down,
        all_time: legacy.total,
        remark: legacy.remark.into(),
        enable: legacy.enable,
        expiry: legacy.expiry_time,
        port: legacy.port as u32,
        protocol: match legacy.protocol.as_str() {
            "vless" => InboundProtocol::Vless,
            "vmess" => InboundProtocol::Vmess,
            "trojan" => InboundProtocol::Trojan,
            "shadowsocks" => InboundProtocol::Shadowsocks,
            "socks" => InboundProtocol::Socks,
            "http" => InboundProtocol::Http,
            "wireguard" => InboundProtocol::WireGuard,
            "dokodemo-door" => InboundProtocol::Dokodemo,
            _ => InboundProtocol::Vless,
        },
        settings,
        stream_settings,
        tag: legacy.tag.into(),
        sniffing,
        traffic_reset: "never".into(),
        last_traffic_reset_time: 0,
        total_limit: 0,
        up_speed_limit: 0,
        down_speed_limit: 0,
        listen: "".into(),
        extra: std::collections::HashMap::new(),
    })
}
