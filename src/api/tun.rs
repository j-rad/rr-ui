// src/api/tun.rs
//! API endpoints for TUN device management

use actix_web::{get, post, web, HttpResponse, Responder};
use serde::{Deserialize, Serialize};
use std::net::{Ipv4Addr, SocketAddr};

use crate::models::GeneralResponse;
use crate::tun_device::{TunConfig, TunState};
use crate::AppState;

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TunConfigRequest {
    /// Name of the TUN device
    pub name: Option<String>,
    /// IP address for the TUN interface
    pub address: Option<String>,
    /// Netmask for the TUN interface
    pub netmask: Option<String>,
    /// MTU size
    pub mtu: Option<u16>,
    /// Whether to set as default route
    pub set_default_route: Option<bool>,
    /// Bypass addresses (won't go through tunnel)
    pub bypass: Option<Vec<String>>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TunStatusResponse {
    pub state: String,
    pub config: TunConfigResponse,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TunConfigResponse {
    pub name: String,
    pub address: String,
    pub netmask: String,
    pub mtu: u16,
    pub set_default_route: bool,
    pub bypass: Vec<String>,
}

impl From<&TunConfig> for TunConfigResponse {
    fn from(config: &TunConfig) -> Self {
        Self {
            name: config.name.clone(),
            address: config.address.to_string(),
            netmask: config.netmask.to_string(),
            mtu: config.mtu(),
            set_default_route: config.set_default_route,
            bypass: config.bypass.iter().map(|ip| ip.to_string()).collect(),
        }
    }
}

impl From<&TunState> for String {
    fn from(state: &TunState) -> Self {
        match state {
            TunState::Stopped => "stopped".to_string(),
            TunState::Starting => "starting".to_string(),
            TunState::Running => "running".to_string(),
            TunState::Error(e) => format!("error: {}", e),
        }
    }
}

/// Get TUN device status
#[get("/status")]
pub async fn get_status(data: web::Data<AppState>) -> impl Responder {
    match &data.tun {
        Some(tun) => {
            let manager = tun.read().await;
            let state = manager.get_status().await;
            let config = manager.get_config().await;

            let response = TunStatusResponse {
                state: (&state).into(),
                config: (&config).into(),
            };

            HttpResponse::Ok().json(GeneralResponse::success(
                "TUN status retrieved",
                Some(serde_json::to_value(response).unwrap()),
            ))
        }
        None => HttpResponse::Ok().json(GeneralResponse::success(
            "TUN not initialized",
            Some(serde_json::json!({
                "state": "not_initialized",
                "config": null
            })),
        )),
    }
}

/// Start TUN device
#[post("/start")]
pub async fn start(
    data: web::Data<AppState>,
    config_req: Option<web::Json<TunConfigRequest>>,
) -> impl Responder {
    // Parse configuration from request or use defaults
    let config = if let Some(req) = config_req {
        let mut cfg = TunConfig::default();

        if let Some(name) = &req.name {
            cfg.name = name.clone();
        }
        if let Some(addr) = &req.address {
            if let Ok(ip) = addr.parse::<Ipv4Addr>() {
                cfg.address = ip;
            }
        }
        if let Some(mask) = &req.netmask {
            if let Ok(ip) = mask.parse::<Ipv4Addr>() {
                cfg.netmask = ip;
            }
        }
        if let Some(mtu) = req.mtu {
            cfg.mtu_profile = mtu.into();
        }
        if let Some(default_route) = req.set_default_route {
            cfg.set_default_route = default_route;
        }
        if let Some(bypass) = &req.bypass {
            cfg.bypass = bypass.iter().filter_map(|s| s.parse().ok()).collect();
        }
        cfg
    } else {
        TunConfig::default()
    };

    // Create or get TUN manager
    let tun_manager = match &data.tun {
        Some(tun) => tun.clone(),
        None => {
            return HttpResponse::InternalServerError().json(GeneralResponse::error(
                "TUN manager not initialized. Restart the server with TUN support enabled.",
            ));
        }
    };

    // Update config and start
    {
        let manager = tun_manager.read().await;
        manager.update_config(config).await;
    }

    let proxy_addr: SocketAddr = "127.0.0.1:10808".parse().unwrap(); // Default SOCKS port

    {
        let mut manager = tun_manager.write().await;
        match manager.start(proxy_addr).await {
            Ok(_) => HttpResponse::Ok().json(GeneralResponse::success("TUN device started", None)),
            Err(e) => HttpResponse::InternalServerError().json(GeneralResponse::error(&format!(
                "Failed to start TUN device: {}",
                e
            ))),
        }
    }
}

/// Stop TUN device
#[post("/stop")]
pub async fn stop(data: web::Data<AppState>) -> impl Responder {
    match &data.tun {
        Some(tun) => {
            let mut manager = tun.write().await;
            match manager.stop().await {
                Ok(_) => {
                    HttpResponse::Ok().json(GeneralResponse::success("TUN device stopped", None))
                }
                Err(e) => HttpResponse::InternalServerError().json(GeneralResponse::error(
                    &format!("Failed to stop TUN device: {}", e),
                )),
            }
        }
        None => HttpResponse::BadRequest().json(GeneralResponse::error("TUN not initialized")),
    }
}

/// Update TUN configuration (requires restart to take effect)
#[post("/config")]
pub async fn update_config(
    data: web::Data<AppState>,
    config_req: web::Json<TunConfigRequest>,
) -> impl Responder {
    match &data.tun {
        Some(tun) => {
            let manager = tun.read().await;
            let mut cfg = manager.get_config().await;

            if let Some(name) = &config_req.name {
                cfg.name = name.clone();
            }
            if let Some(addr) = &config_req.address {
                if let Ok(ip) = addr.parse::<Ipv4Addr>() {
                    cfg.address = ip;
                }
            }
            if let Some(mask) = &config_req.netmask {
                if let Ok(ip) = mask.parse::<Ipv4Addr>() {
                    cfg.netmask = ip;
                }
            }
            if let Some(mtu) = config_req.mtu {
                cfg.mtu_profile = mtu.into();
            }
            if let Some(default_route) = config_req.set_default_route {
                cfg.set_default_route = default_route;
            }
            if let Some(bypass) = &config_req.bypass {
                cfg.bypass = bypass.iter().filter_map(|s| s.parse().ok()).collect();
            }

            manager.update_config(cfg.clone()).await;

            HttpResponse::Ok().json(GeneralResponse::success(
                "TUN configuration updated. Restart TUN device for changes to take effect.",
                Some(serde_json::to_value(TunConfigResponse::from(&cfg)).unwrap()),
            ))
        }
        None => HttpResponse::BadRequest().json(GeneralResponse::error("TUN not initialized")),
    }
}
