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
    for (tag, traffic) in &acc.inbound_map {
        if traffic.up > 0 || traffic.down > 0 {
            let query = format!(
                "UPDATE inbound SET up_bytes += {}, down_bytes += {} WHERE tag = '{}'",
                traffic.up, traffic.down, tag
            );
            if let Err(e) = state.db.client.query(&query).await {
                error!("Failed to update inbound traffic for {}: {}", tag, e);
            }
        }
    }

    if !acc.client_map.is_empty() {
        if let Ok(inbounds) = state.db.client.select::<Vec<Inbound>>("inbound").await {
            for mut inbound in inbounds {
                let mut modified = false;

                if let Some(clients) = inbound.settings.clients_mut() {
                    for client in clients.iter_mut() {
                        if let Some(email) = &client.email {
                            if let Some(delta) = acc.client_map.get(email) {
                                if delta.up > 0 || delta.down > 0 {
                                    client.up += delta.up;
                                    client.down += delta.down;
                                    modified = true;

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
                                            modified = true;
                                        }
                                    }

                                    // Enforce Expiry Time
                                    if client.expiry_time > 0 {
                                        let now = Utc::now().timestamp_millis();
                                        if now > client.expiry_time && client.enable {
                                            info!("Client {} expired. Disabling.", email);
                                            client.enable = false;
                                            modified = true;
                                        }
                                    }

                                    // Enforce IP Limit
                                    if let Some(limit_ip) = client.limit_ip {
                                        if limit_ip > 0 && client.enable {
                                            let active_ips =
                                                state.log_watcher.get_active_ip_count(email);
                                            if active_ips > limit_ip as usize {
                                                info!(
                                                    "Client {} exceeded IP limit ({} > {}). Disabling.",
                                                    email, active_ips, limit_ip
                                                );
                                                // Ideally we kick specific IPs, but RustRay doesn't support that easily.
                                                // We disable the user for now (strict policy).
                                                client.enable = false;
                                                modified = true;
                                            }
                                        }
                                    }

                                    // Speed Watchdog (Warning Only)
                                    // traffic job runs every 30s (Slow Loop) for DB flush, but accumulator has data.
                                    // To calculate speed properly, we need the delta time since last check.
                                    // The 'delta' here is accumulated over the slow loop interval?
                                    // Actually, traffic job has a Fast Loop (1s). We should check speed THERE.
                                    // But we are in the Slow Loop logic block here.
                                    // Let's rely on the Fast Loop for real-time speed enforcement if needed.
                                    // For now, let's just log a warning if the *average* speed over 30s is massive?
                                    // No, let's move speed check to Fast Loop.
                                    // But we don't have mutable access to Client config in Fast Loop easily without DB query.
                                    // Let's stick to IP limiting here (Slow Loop) and handle Speed in Fast Loop if feasible,
                                    // or just log aggregated violation here.

                                    // Calculating average speed over the last flush interval (approx 30s)
                                    if client.up_speed_limit > 0 || client.down_speed_limit > 0 {
                                        let interval_secs = 30; // Approximation
                                        let up_speed = (delta.up as u64) / interval_secs;
                                        let down_speed = (delta.down as u64) / interval_secs;

                                        // Speed limits are in kbps usually? Or Bps?
                                        // Models say 'up_speed_limit: u32'. Usually specificied in bytes/sec or kbps in UI.
                                        // Let's assume bytes/sec for strictness or verify UI.
                                        // UI says "kbps". So limit * 1024.

                                        let limit_up_bps = client.up_speed_limit as u64 * 1024;
                                        let limit_down_bps = client.down_speed_limit as u64 * 1024;

                                        if client.up_speed_limit > 0 && up_speed > limit_up_bps {
                                            info!(
                                                "Client {} exceeding upload limit ({} B/s > {} B/s)",
                                                email, up_speed, limit_up_bps
                                            );
                                            // Could trigger a temporary throttle or penalty here
                                        }

                                        if client.down_speed_limit > 0
                                            && down_speed > limit_down_bps
                                        {
                                            info!(
                                                "Client {} exceeding download limit ({} B/s > {} B/s)",
                                                email, down_speed, limit_down_bps
                                            );
                                        }
                                    }
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
