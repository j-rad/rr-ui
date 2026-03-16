use log::{error, info, warn};
use regex::Regex;
use std::collections::{HashMap, HashSet};
use std::fs::File;
use std::io::{BufRead, BufReader, Read, Seek, SeekFrom};
use std::path::Path;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};
use std::time::{Duration, SystemTime};
use tokio::time::interval;

/// Tracks active IPs for each user based on access logs.
#[derive(Clone)]
pub struct LogWatcher {
    /// Maps User Email -> Set of Active IPs (with timestamp)
    active_ips: Arc<Mutex<HashMap<String, HashMap<String, u64>>>>,
    log_path: String,
}

impl LogWatcher {
    pub fn new(log_path: &str) -> Self {
        Self {
            active_ips: Arc::new(Mutex::new(HashMap::new())),
            log_path: log_path.to_string(),
        }
    }

    /// Starts the log watching background task.
    pub async fn run(&self) {
        let path = Path::new(&self.log_path);
        let mut file_cursor = 0;

        // Ensure file exists or wait for it
        if !path.exists() {
            warn!("Log file {} not found. Waiting...", self.log_path);
        }

        let mut interval_timer = interval(Duration::from_secs(2));
        // Regex to parse key info: 2023/10/27 10:00:00 [Info] [email] ... source: [ip]:port
        // Example: 2023/10/27 10:00:00 [Info] [user@example.com] inbound/vless-inbound: received request from 192.168.1.1:54321
        // Simplified regex for robustness
        let log_pattern =
            Regex::new(r"\[Info\]\s+\[([^\]]+)\]\s+.*from\s+((?:\d{1,3}\.){3}\d{1,3})").unwrap();

        loop {
            interval_timer.tick().await;

            if let Ok(file) = File::open(path) {
                let mut reader = BufReader::new(&file);

                // Seek to last position
                if let Ok(metadata) = file.metadata() {
                    let len = metadata.len();
                    if len < file_cursor {
                        // File truncated/rotated
                        file_cursor = 0;
                    }
                    if let Err(e) = reader.seek(SeekFrom::Start(file_cursor)) {
                        error!("Failed to seek log file: {}", e);
                        continue;
                    }

                    let mut lines = String::new();
                    if let Ok(bytes_read) = reader.read_to_string(&mut lines) {
                        if bytes_read > 0 {
                            file_cursor += bytes_read as u64;
                            self.process_lines(&lines, &log_pattern);
                            self.cleanup_stale_ips();
                        }
                    }
                }
            } else {
                // File might not exist yet
                // warn!("Could not open log file: {}", self.log_path);
            }
        }
    }

    fn process_lines(&self, lines: &str, regex: &Regex) {
        let now = SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs();

        let mut guard = self.active_ips.lock().unwrap();

        for line in lines.lines() {
            if let Some(caps) = regex.captures(line) {
                if let (Some(email), Some(ip)) = (caps.get(1), caps.get(2)) {
                    let email_str = email.as_str().to_string();
                    let ip_str = ip.as_str().to_string();

                    // Update or insert IP with current timestamp
                    guard.entry(email_str).or_default().insert(ip_str, now);
                }
            }
        }
    }

    /// Removes IPs that haven't been seen for 5 minutes (300s).
    fn cleanup_stale_ips(&self) {
        let now = SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs();
        let timeout = 300; // 5 minutes TTL

        let mut guard = self.active_ips.lock().unwrap();

        // Retain only valid IPs
        for (_, ips) in guard.iter_mut() {
            ips.retain(|_, timestamp| now - *timestamp < timeout);
        }

        // Remove users with empty IP maps
        guard.retain(|_, ips| !ips.is_empty());
    }

    /// Returns the current active IP count for a user.
    pub fn get_active_ip_count(&self, email: &str) -> usize {
        let guard = self.active_ips.lock().unwrap();
        guard.get(email).map(|ips| ips.len()).unwrap_or(0)
    }
}
