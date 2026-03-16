// src/api/log.rs
use crate::models::GeneralResponse;
use actix_web::{HttpResponse, Responder, get, post, web};
use chrono::Utc;
use futures_util::stream::StreamExt;
use serde_json::json;
use std::fs::File;
use std::io::{BufRead, BufReader};
use tokio::time::{Duration, interval};
use tokio_stream::wrappers::IntervalStream;

/// Reads the last `n` lines from a file.
///
/// # Arguments
///
/// * `path` - The path to the file.
/// * `_n` - The number of lines to read. Currently unused.
///
fn read_last_lines(path: &str, n: usize) -> Vec<String> {
    let file = match File::open(path) {
        Ok(f) => f,
        Err(_) => return vec![],
    };
    let reader = BufReader::new(file);
    let lines: Vec<String> = reader.lines().map_while(Result::ok).collect();

    // Return last n lines
    let start = lines.len().saturating_sub(n);
    lines[start..].to_vec()
}

/// Handles the GET request to fetch access logs.
///
/// Reads the last 100 lines from "access.log" and returns them as a JSON response.
#[get("/access")]
pub async fn get_access_log() -> impl Responder {
    let logs = read_last_lines("access.log", 100);
    HttpResponse::Ok().json(GeneralResponse::success("success", Some(json!(logs))))
}

/// Handles the GET request to fetch error logs.
///
/// Reads the last 100 lines from "error.log" and returns them as a JSON response.
#[get("/error")]
pub async fn get_error_log() -> impl Responder {
    let logs = read_last_lines("error.log", 100);
    HttpResponse::Ok().json(GeneralResponse::success("success", Some(json!(logs))))
}

/// Handles the POST request to clear log files.
///
/// Truncates both "access.log" and "error.log".
#[post("/clear")]
pub async fn clear_logs() -> impl Responder {
    let _ = File::create("access.log");
    let _ = File::create("error.log");
    HttpResponse::Ok().json(GeneralResponse::success("Logs cleared", None))
}

/// Handles the GET request for a live log stream (Server-Sent Events).
///
/// Streams simulated log entries every second.
#[get("/panel/log/live")]
pub async fn live_log_stream() -> impl Responder {
    let stream = IntervalStream::new(interval(Duration::from_secs(1))).map(|_| {
        let timestamp = Utc::now().to_rfc3339();
        let log_line = format!("[{}] INFO: Simulated log entry.", timestamp);
        Ok(web::Bytes::from(format!("data: {}\n\n", log_line)))
            as Result<web::Bytes, std::io::Error>
    });

    HttpResponse::Ok()
        .insert_header(("Content-Type", "text/event-stream"))
        .streaming(stream)
}
