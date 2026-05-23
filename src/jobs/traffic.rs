// No change yet, just viewing

use crate::{
    AppState,
    models::{AllSetting, Inbound, NetIO, RealtimeTelemetry},
    repositories::setting::SettingOps,
};
use actix_web::web::Bytes;
use chrono::Utc;
use cron::Schedule;
use log::{debug, error, info};
use regex::Regex;
use std::collections::HashMap;
use std::str::FromStr;
use std::sync::{Arc, Mutex};
use std::time::Duration;
use tokio::sync::broadcast::Sender;

/// Represents the change in traffic (upload and download).
#[derive(Default, Clone, Copy)]
struct TrafficDelta {
    up: i64,
    down: i64,
}

/// Helper to accumulate traffic in memory
#[derive(Default, Clone)]
struct TrafficAccumulator {
    inbound_map: HashMap<String, TrafficDelta>,
    client_map: HashMap<String, TrafficDelta>,
}

impl TrafficAccumulator {
    fn new() -> Self {
        Self::default()
    }

    fn add_inbound(&mut self, tag: String, delta: TrafficDelta) {
        let entry = self.inbound_map.entry(tag).or_default();
        entry.up += delta.up;
        entry.down += delta.down;
    }

    fn add_client(&mut self, email: String, delta: TrafficDelta) {
        let entry = self.client_map.entry(email).or_default();
        entry.up += delta.up;
        entry.down += delta.down;
    }

    fn clear(&mut self) {
        self.inbound_map.clear();
        self.client_map.clear();
    }
}

/// Starts a background job to periodically poll for traffic stats and update the database.
///
/// This job runs two loops:
/// 1. Fast Loop (1s): Polls RustRay for live traffic, broadcasts to UI, and accumulates deltas.
/// 2. Slow Loop (30s): Flushes accumulated deltas to the database and checks expiration.
pub async fn start_traffic_job(state: Arc<AppState>, tx: Sender<Bytes>) {
    // Shared accumulator between the two loops
    let accumulator = Arc::new(Mutex::new(TrafficAccumulator::new()));

    // Spawn the fast loop (1s polling & broadcast)
    let state_fast = state.clone();
    let acc_fast = accumulator.clone();
    let tx_fast = tx;

    tokio::spawn(async move {
        let mut interval = tokio::time::interval(Duration::from_secs(1));
        interval.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Skip);

        let traffic_regex =
            Regex::new(r"(inbound|outbound)>>>([^>]+)>>>traffic>>>(downlink|uplink)").unwrap();
        let user_regex = Regex::new(r"user>>>([^>]+)>>>traffic>>>(downlink|uplink)").unwrap();

        let mut backoff = Duration::from_secs(1);
        let max_backoff = Duration::from_secs(60);

        loop {
            interval.tick().await;
            let mut rustray = state_fast.rustray.clone();

            // Health-check guard: check if rustray is connected/healthy
            if !rustray.is_healthy() {
                debug!("Fast Loop: RustRay is not healthy, skipping poll. Backing off...");
                tokio::time::sleep(backoff).await;
                backoff = std::cmp::min(backoff * 2, max_backoff);
                continue;
            }

            // Fetch traffic stats
            match rustray.get_traffic_stats(true).await {
                Ok(stats) => {
                    // Reset backoff on success
                    backoff = Duration::from_secs(1);

                    let mut total_up = 0u64;
                    let mut total_down = 0u64;

                    if !stats.is_empty() {
                        let mut acc = acc_fast.lock().unwrap();

                        for stat in &stats {
                            if stat.value <= 0 {
                                continue;
                            }

                            let mut delta = TrafficDelta::default();

                            if let Some(caps) = traffic_regex.captures(&stat.name) {
                                if &caps[1] == "inbound" {
                                    if &caps[3] == "uplink" {
                                        delta.up = stat.value;
                                        total_up += stat.value as u64;
                                    } else {
                                        delta.down = stat.value;
                                        total_down += stat.value as u64;
                                    }
                                    acc.add_inbound(caps[2].to_string(), delta);
                                }
                            } else if let Some(caps) = user_regex.captures(&stat.name) {
                                if &caps[2] == "uplink" {
                                    delta.up = stat.value;
                                } else {
                                    delta.down = stat.value;
                                }
                                acc.add_client(caps[1].to_string(), delta);
                            }
                        }
                    }

                    // Broadcast telemetry
                    let telemetry = RealtimeTelemetry {
                        system: None,
                        traffic: NetIO {
                            up: total_up,
                            down: total_down,
                        },
                        server_status: None,
                    };

                    if let Ok(json) = serde_json::to_string(&telemetry) {
                        let _ = tx_fast.send(Bytes::from(json));
                    }
                }
                Err(e) => {
                    error!(
                        "Fast Loop: Failed to fetch traffic stats: {}. Backing off...",
                        e
                    );
                    tokio::time::sleep(backoff).await;
                    backoff = std::cmp::min(backoff * 2, max_backoff);
                }
            }
        }
    });

    // Slow Loop (30s) - Database Persistence
    let mut interval = tokio::time::interval(Duration::from_secs(30));
    interval.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Skip);

    loop {
        interval.tick().await;

        // Flush accumulator to DB
        // Flush accumulator to DB
        let snapshot = {
            let mut acc = accumulator.lock().unwrap();
            if !acc.inbound_map.is_empty() || !acc.client_map.is_empty() {
                Some(std::mem::take(&mut *acc))
            } else {
                None
            }
        };

        if let Some(acc) = snapshot {
            debug!("Flushing traffic stats to database...");
            flush_traffic_deltas(&state, &acc).await;
        }

        // Check for client traffic resets
        check_and_reset_client_traffic(&state).await;
    }
}

/// Flushes accumulated traffic deltas to the database.
async fn flush_traffic_deltas(state: &Arc<AppState>, acc: &TrafficAccumulator) {
    // 1. Bulk Update Inbound Traffic (Optimized)
    let inbound_updates: Vec<_> = acc
        .inbound_map
        .iter()
        .filter(|(_, traffic)| traffic.up > 0 || traffic.down > 0)
        .map(|(tag, traffic)| {
            serde_json::json!({
                "tag": tag,
                "up": traffic.up,
                "down": traffic.down,
            })
        })
        .collect();

    if !inbound_updates.is_empty() {
        let query = "FOR $u IN $updates {
            UPDATE inbound SET up_bytes += $u.up, down_bytes += $u.down WHERE tag = $u.tag;
        };";
        if let Err(e) = state
            .db
            .client
            .query(query)
            .bind(("updates", inbound_updates))
            .await
        {
            error!("Failed to batch update inbound traffic: {}", e);
        }
    }

    // 2. Client Traffic Updates and Policy Enforcement (Optimized)
    if !acc.client_map.is_empty() {
        // Collect only client emails that have traffic
        let active_emails: Vec<_> = acc.client_map.keys().cloned().collect();

        // Fetch only inbounds that contain at least one of the active clients
        // This avoids loading all inbounds into memory if only a few are active
        let query = "SELECT * FROM inbound WHERE settings.clients[WHERE email IN $emails]";
        let inbounds: Vec<Inbound> = match state
            .db
            .client
            .query(query)
            .bind(("emails", active_emails))
            .await
        {
            Ok(mut res) => res.take(0).unwrap_or_default(),
            Err(e) => {
                error!("Failed to fetch active inbounds: {}", e);
                return;
            }
        };

        for mut inbound in inbounds {
            let mut modified = false;

            if let Some(clients) = inbound.settings.clients_mut() {
                for client in clients.iter_mut() {
                    if let Some(email) = client.email.clone() {
                        if let Some(delta) = acc.client_map.get(&email) {
                            if delta.up > 0 || delta.down > 0 {
                                client.up += delta.up;
                                client.down += delta.down;
                                modified = true;

                                // Policy Enforcement
                                enforce_client_policies(client, &email, delta, state, &mut modified);
                            }
                        }
                    }
                }
            }

            if modified {
                if let Some(record_id) = &inbound.id {
                    let _: Result<Option<Inbound>, _> = state
                        .db
                        .client
                        .update(("inbound", record_id.id.to_string()))
                        .content(inbound)
                        .await;
                }
            }
        }
    }
}

/// Helper to enforce client traffic policies (limits, expiry, speed).
fn enforce_client_policies(
    client: &mut crate::models::Client,
    email: &str,
    delta: &TrafficDelta,
    state: &Arc<AppState>,
    modified: &mut bool,
) {
    // Enforce Total Flow Limit
    if client.total_flow_limit > 0 {
        let used_up = client.up.max(0) as u64;
        let used_down = client.down.max(0) as u64;
        let total_used = used_up + used_down;

        if total_used >= client.total_flow_limit && client.enable {
            info!(
                "Client {} exceeded flow limit ({} >= {}). Disabling.",
                email, total_used, client.total_flow_limit
            );
            client.enable = false;
            *modified = true;
        }
    }

    // Enforce Expiry Time
    if client.expiry_time > 0 {
        let now = Utc::now().timestamp_millis();
        if now > client.expiry_time && client.enable {
            info!("Client {} expired. Disabling.", email);
            client.enable = false;
            *modified = true;
        }
    }

    // Enforce IP Limit
    if let Some(limit_ip) = client.limit_ip {
        if limit_ip > 0 && client.enable {
            let active_ips = state.log_watcher.get_active_ip_count(email);
            if active_ips > limit_ip as usize {
                info!(
                    "Client {} exceeded IP limit ({} > {}). Disabling.",
                    email, active_ips, limit_ip
                );
                client.enable = false;
                *modified = true;
            }
        }
    }

    // Speed Watchdog (Warning Only)
    if client.up_speed_limit > 0 || client.down_speed_limit > 0 {
        let interval_secs = 30; // Approximation for slow loop
        let up_speed = (delta.up as u64) / interval_secs;
        let down_speed = (delta.down as u64) / interval_secs;

        let limit_up_bps = client.up_speed_limit as u64 * 1024;
        let limit_down_bps = client.down_speed_limit as u64 * 1024;

        if client.up_speed_limit > 0 && up_speed > limit_up_bps {
            info!(
                "Client {} exceeding upload limit ({} B/s > {} B/s)",
                email, up_speed, limit_up_bps
            );
        }

        if client.down_speed_limit > 0 && down_speed > limit_down_bps {
            info!(
                "Client {} exceeding download limit ({} B/s > {} B/s)",
                email, down_speed, limit_down_bps
            );
        }
    }
}

/// Checks and resets client traffic usage based on a cron schedule defined in settings.
///
/// # Arguments
///
/// * `state` - The shared application state.
async fn check_and_reset_client_traffic(state: &Arc<AppState>) {
    let settings: Option<AllSetting> = match <AllSetting as SettingOps>::get(&state.db).await {
        Ok(Some(s)) => Some(s),
        _ => None,
    };

    if let Some(settings) = settings {
        if let Some(cron_str) = settings.traffic_reset_cron {
            if let Ok(schedule) = Schedule::from_str(&cron_str) {
                let now = Utc::now();
                let mut inbounds: Vec<Inbound> = match state.db.client.select("inbound").await {
                    Ok(result) => result,
                    Err(_) => return,
                };

                for inbound in &mut inbounds {
                    let mut needs_update = false;

                    // Only process inbounds that have clients
                    // We need to scope this block so the mutable borrow is dropped
                    {
                        if let Some(clients) = inbound.settings.clients_mut() {
                            for client in clients.iter_mut() {
                                let next_reset_date = client.next_reset_date.unwrap_or(0);
                                if next_reset_date > 0 && now.timestamp() >= next_reset_date {
                                    info!(
                                        "Resetting traffic for client: {}",
                                        client.email.as_deref().unwrap_or("")
                                    );
                                    // Reset traffic usage
                                    client.up = 0;
                                    client.down = 0;

                                    if let Some(next_time) = schedule.upcoming(Utc).next() {
                                        let next_time: chrono::DateTime<chrono::Utc> = next_time;
                                        client.next_reset_date = Some(next_time.timestamp());
                                        needs_update = true;
                                    }
                                } else if next_reset_date == 0 {
                                    // Set initial reset date for new clients
                                    if let Some(next_time) = schedule.upcoming(Utc).next() {
                                        let next_time: chrono::DateTime<chrono::Utc> = next_time;
                                        client.next_reset_date = Some(next_time.timestamp());
                                        needs_update = true;
                                    }
                                }
                            }
                        }
                    } // Mutable borrow of inbound.settings is dropped here

                    if needs_update {
                        let inbound_id = inbound.id.as_ref().unwrap().clone();
                        let inbound_clone = inbound.clone();
                        let _: Result<Option<Inbound>, _> = state
                            .db
                            .client
                            .update(("inbound", inbound_id.id.to_string()))
                            .content(inbound_clone)
                            .await;
                    }
                }
            }
        }
    }
}
