use crate::db::DbClient;
use crate::rustray_client::RustRayClient;
use crate::rustray_process::SharedRustRayProcess;
use log::{error, info, warn};
use rand::Rng;
use tokio::time::{Duration, sleep};

pub async fn start_watchdog_job(
    process: SharedRustRayProcess,
    db: DbClient,
    mut client: RustRayClient,
) {
    info!("Starting core process watchdog...");
    let mut interval = tokio::time::interval(Duration::from_secs(1));
    let mut failures = 0;

    // Initial delay to let process start
    sleep(Duration::from_secs(2)).await;

    loop {
        interval.tick().await;

        let mut is_running = {
            // Minimize lock contention by keeping the critical section small
            let mut guard = process.process.lock().await;
            guard.is_running()
        };

        // Perform gRPC health check if the OS process is running
        if is_running {
            if client.is_healthy() {
                // Use standard gRPC health check
                if let Err(e) = client.check_health().await {
                    warn!("Watchdog: gRPC health check failed: {}", e);
                    // Treat as failure to trigger backoff/restart
                    is_running = false;
                }
            } else {
                // Try to reconnect once; if it fails, maybe core is still starting or is dead
                if let Err(e) = client.connect_with_retry(1).await {
                    warn!("Watchdog: could not connect to gRPC API: {}", e);
                    is_running = false;
                }
            }
        }

        if !is_running {
            failures += 1;
            warn!(
                "Core process is unresponsive or not running. Failure count: {}. Attempting restart...",
                failures
            );

            // Jitter-aware exponential backoff
            // Base: 2^failures, capped at 30s
            // Jitter: 0-1000ms
            let cap = 30u64;
            let expo = 1u64
                .checked_shl(std::cmp::min(failures, 6) as u32)
                .unwrap_or(cap);
            let backoff_secs = std::cmp::min(cap, expo);

            let jitter_ms = rand::thread_rng().gen_range(0..1000);
            let wait_time = Duration::from_secs(backoff_secs) + Duration::from_millis(jitter_ms);

            warn!(
                "Backoff: waiting {}s {}ms before restart attempt #{}",
                backoff_secs, jitter_ms, failures
            );
            sleep(wait_time).await;

            let mut guard = process.process.lock().await;
            if let Err(e) = guard.restart(&db).await {
                error!("Watchdog failed to restart core: {}", e);
            } else {
                info!("Watchdog successfully restarted core.");
                // We don't reset failures immediately to prevent rapid flap-reset cycles
                // But if it stays stable we should.
                // For simplicity here, we rely on the loop checking next time.
                // If it crashes again immediately, failures increments.

                // Also reset client so it reconnects to the new process instance
                let api_port = client.api_port();
                client = RustRayClient::new(api_port);
            }
        } else {
            // If running successfully, slowly decay failure count to reset backoff eventually
            if failures > 0 {
                failures = 0;
            }
        }
    }
}
