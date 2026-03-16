use crate::AppState;
use crate::domain::ports::InboundRepository;
use crate::models::GeneralResponse;
use actix_web::{HttpResponse, Responder, delete, get, web};
use serde_json::json;
use std::net::IpAddr;
use std::sync::{Arc, Mutex};
use sysinfo::{CpuExt, DiskExt, NetworkExt, System, SystemExt};

#[derive(Clone)]
pub struct SystemState {
    pub sys: Arc<Mutex<System>>,
}

impl Default for SystemState {
    fn default() -> Self {
        Self::new()
    }
}

impl SystemState {
    pub fn new() -> Self {
        let mut sys = System::new_all();
        sys.refresh_all();
        Self {
            sys: Arc::new(Mutex::new(sys)),
        }
    }
}

#[get("/status")]
pub async fn get_status(
    sys_data: actix_web::web::Data<SystemState>,
    app_state: actix_web::web::Data<AppState>,
) -> impl Responder {
    let mut sys = sys_data.sys.lock().unwrap();
    sys.refresh_all();

    let cpu_usage = sys.global_cpu_info().cpu_usage();
    let total_mem = sys.total_memory();
    let used_mem = sys.used_memory();

    let mut disk_total = 0;
    let mut disk_used = 0;
    for disk in sys.disks() {
        if disk.mount_point().to_str() == Some("/") {
            disk_total = disk.total_space();
            disk_used = disk.total_space() - disk.available_space();
            break;
        }
    }

    let uptime = sys.uptime();
    let loads = sys.load_average();

    // Fetch active protocols
    let active_protocols = {
        #[cfg(feature = "server")]
        {
            let result = app_state.inbound_repo.find_all().await;
            match result {
                Ok(inbounds) => {
                    let mut protocols: Vec<String> = inbounds
                        .into_iter()
                        .filter(|i| i.enable)
                        .map(|i| i.settings.protocol_name().to_string())
                        .collect();
                    protocols.sort();
                    protocols.dedup();
                    protocols
                }
                Err(_) => vec![],
            }
        }
        #[cfg(not(feature = "server"))]
        {
            vec![]
        }
    };

    let mut net_up = 0;
    let mut net_down = 0;

    // Sum network usage across all interfaces except loopback
    for (_name, data) in sys.networks() {
        net_up += data.transmitted();
        net_down += data.received();
    }

    let status = json!({
        "cpu": cpu_usage,
        "cpuCores": sys.physical_core_count().unwrap_or(1),
        "logicalPro": sys.cpus().len(),
        "cpuSpeedMhz": sys.global_cpu_info().frequency(),
        "mem": { "current": used_mem, "total": total_mem },
        "swap": { "current": sys.used_swap(), "total": sys.total_swap() },
        "disk": { "current": disk_used, "total": disk_total },
        "loads": [loads.one, loads.five, loads.fifteen],
        "netIO": { "up": net_up, "down": net_down },
        "netTraffic": { "sent": net_up, "recv": net_down },
        "publicIP": { "ipv4": "127.0.0.1", "ipv6": "::1" },
        "tcpCount": 0,
        "udpCount": 0,
        "uptime": uptime,
        "appStats": { "threads": 0, "mem": 0, "uptime": 0 },
        "rustray": { "state": "running", "errorMsg": "", "version": "1.8.4" },
        "activeProtocols": active_protocols,
        "isOpenWrt": std::path::Path::new("/etc/openwrt_release").exists()
    });

    HttpResponse::Ok().json(GeneralResponse::success("success", Some(status)))
}

#[get("/mode")]
pub async fn get_mode() -> impl Responder {
    #[cfg(feature = "server")]
    let mode = "server";
    #[cfg(not(feature = "server"))]
    let mode = "client";

    HttpResponse::Ok().json(GeneralResponse::success(
        "success",
        Some(json!({ "mode": mode })),
    ))
}

#[get("/banned-ips")]
pub async fn get_banned_ips(data: web::Data<AppState>) -> impl Responder {
    let banned = data.shield.list_banned();
    HttpResponse::Ok().json(GeneralResponse::success("success", Some(json!(banned))))
}

#[delete("/banned-ips/{ip}")]
pub async fn unban_ip(data: web::Data<AppState>, path: web::Path<String>) -> impl Responder {
    let ip_str = path.into_inner();
    match ip_str.parse::<IpAddr>() {
        Ok(ip) => {
            data.shield.reset(ip);
            HttpResponse::Ok().json(GeneralResponse::success("IP unbanned", None))
        }
        Err(_) => HttpResponse::BadRequest().json(GeneralResponse::error("Invalid IP address")),
    }
}

#[delete("/banned-ips")]
pub async fn clear_banned_ips(data: web::Data<AppState>) -> impl Responder {
    data.shield.list_banned().into_iter().for_each(|b| {
        data.shield.reset(b.ip);
    });
    HttpResponse::Ok().json(GeneralResponse::success("All bans cleared", None))
}
