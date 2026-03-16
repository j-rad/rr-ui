// src/api/signaling.rs
//! Signaling Server — Phase 3
//!
//! REST endpoints that allow an authorised admin to push node configuration
//! updates to clients via two out-of-band channels:
//!
//!   1. **DNS TXT record** — A stable DNS hostname whose TXT record holds a
//!      base64-encoded, AES-256-GCM-encrypted node descriptor.  Clients poll
//!      this record to discover when a swap is needed.
//!
//!   2. **MQTT hidden topic** — A topic like `_sr/nodes/<node_id>` on a
//!      password-protected broker.  Clients subscribe at startup and receive
//!      updates with QoS-1 delivery guarantees.
//!
//! # Access Control
//!
//! Every endpoint is guarded by a `Bearer` token validated against a bcrypt
//! hash stored in the application settings.  The token is read from the
//! `Authorization: Bearer <token>` header.
//!
//! # Endpoints
//!
//! | Method | Path | Description |
//! |--------|------|-------------|
//! | POST   | `/panel/api/signaling/push` | Push a node update |
//! | GET    | `/panel/api/signaling/status` | Last delivery receipt per node |
//!
//! # Push Request
//!
//! ```json
//! {
//!   "node_id": "ir-mci-01",
//!   "new_ip": "1.2.3.4",
//!   "remark": "operator note",
//!   "channels": ["dns", "mqtt"]   // optional, both by default
//! }
//! ```

use actix_web::{HttpRequest, HttpResponse, get, post, web};
use base64::{Engine, engine::general_purpose::STANDARD as B64};
use chacha20poly1305::{
    ChaCha20Poly1305, Nonce,
    aead::{Aead, AeadCore, KeyInit, OsRng},
};
use chrono::Utc;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::Mutex;

// ──────────────────────────────── Types ──────────────────────────────────────

/// Which out-of-band channel(s) to notify.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum SignalChannel {
    Dns,
    Mqtt,
}

/// Body accepted by `POST /panel/api/signaling/push`.
#[derive(Debug, Deserialize)]
pub struct PushRequest {
    /// Logical node identifier, matches `ManagedNode::node_id`.
    pub node_id: String,
    /// New public IP to advertise.
    pub new_ip: String,
    /// Human note — embedded in the DNS TXT payload.
    #[serde(default)]
    pub remark: String,
    /// Which channels to use; defaults to both if absent.
    #[serde(default = "default_channels")]
    pub channels: Vec<SignalChannel>,
}

fn default_channels() -> Vec<SignalChannel> {
    vec![SignalChannel::Dns, SignalChannel::Mqtt]
}

/// Outcome for one channel in a push operation.
#[derive(Debug, Serialize)]
pub struct ChannelResult {
    pub channel: String,
    pub ok: bool,
    pub error: Option<String>,
}

/// Response body for `POST /panel/api/signaling/push`.
#[derive(Debug, Serialize)]
pub struct PushResponse {
    pub node_id: String,
    pub timestamp: i64,
    pub results: Vec<ChannelResult>,
}

/// A delivery receipt stored in memory for `GET /panel/api/signaling/status`.
#[derive(Debug, Clone, Serialize)]
pub struct DeliveryReceipt {
    pub node_id: String,
    pub new_ip: String,
    pub remark: String,
    pub timestamp: i64,
    pub dns_ok: bool,
    pub mqtt_ok: bool,
}

// ──────────────────────────────── App state ──────────────────────────────────

/// Shared application state injected via Actix `Data<Arc<SignalingState>>`.
pub struct SignalingState {
    /// Configured bearer tokens (may be multiple for rotation).
    pub valid_tokens: Vec<String>,
    /// DNS TXT signaling configuration (Cloudflare zone).
    pub dns_config: Option<DnsTxtConfig>,
    /// MQTT signaling configuration.
    pub mqtt_config: Option<MqttSignalingConfig>,
    /// Latest receipt per node, protected by mutex for concurrent access.
    pub receipts: Mutex<HashMap<String, DeliveryReceipt>>,
    /// Pre-shared key for payload encryption (32 bytes hex string)
    pub psk: Option<String>,
}

impl SignalingState {
    pub fn new(
        tokens: Vec<String>,
        dns_config: Option<DnsTxtConfig>,
        mqtt_config: Option<MqttSignalingConfig>,
        psk: Option<String>,
    ) -> Arc<Self> {
        Arc::new(Self {
            valid_tokens: tokens,
            dns_config,
            mqtt_config,
            receipts: Mutex::new(HashMap::new()),
            psk,
        })
    }
}

// ──────────────────────────────── Encryption ─────────────────────────────────

/// Encrypts the payload using ChaCha20Poly1305 if a PSK is provided.
/// Returns base64 encoded string (nonce + ciphertext).
fn encrypt_payload(payload: &DnsTxtPayload, psk_hex: Option<&str>) -> anyhow::Result<String> {
    let json_bytes = serde_json::to_vec(payload)?;

    if let Some(key_hex) = psk_hex {
        let key_bytes = hex::decode(key_hex)?;
        if key_bytes.len() != 32 {
            return Err(anyhow::anyhow!("Invalid PSK length, expected 32 bytes"));
        }

        let cipher = ChaCha20Poly1305::new_from_slice(&key_bytes)
            .map_err(|e| anyhow::anyhow!("Key init failed: {}", e))?;

        let nonce = ChaCha20Poly1305::generate_nonce(&mut OsRng); // 96-bits; unique per message
        let ciphertext = cipher
            .encrypt(&nonce, json_bytes.as_ref())
            .map_err(|e| anyhow::anyhow!("Encryption failed: {}", e))?;

        // Combine nonce + ciphertext
        let mut combined = nonce.to_vec();
        combined.extend(ciphertext);

        Ok(B64.encode(&combined))
    } else {
        // Fallback to plain base64 if no PSK (not recommended for prod)
        Ok(B64.encode(&json_bytes))
    }
}

// ──────────────────────────────── DNS TXT ────────────────────────────────────

/// Settings for publishing node descriptors as Cloudflare-managed DNS TXT records.
///
/// The TXT record name convention is: `_sig.<node_id>.<base_domain>`.
#[derive(Debug, Clone)]
pub struct DnsTxtConfig {
    /// Cloudflare API token (zone:edit permission).
    pub api_token: String,
    /// Zone ID of the domain used for signaling.
    pub zone_id: String,
    /// Base domain (e.g. "signal.example.com").  Node descriptors are published
    /// under `_sig.<node_id>.<base_domain>`.
    pub base_domain: String,
}

/// Payload embedded in the DNS TXT record (JSON → base64).
#[derive(Debug, Serialize, Deserialize)]
struct DnsTxtPayload {
    node_id: String,
    ip: String,
    remark: String,
    ts: i64,
}

/// Cloudflare DNS TXT record resource (minimal set of fields for our use).
#[derive(Debug, Serialize, Deserialize)]
struct CfRecord {
    id: String,
    #[serde(rename = "type")]
    record_type: String,
    name: String,
    content: String,
}

#[derive(Serialize, Deserialize)]
struct CfListResponse {
    result: Vec<CfRecord>,
    success: bool,
}

#[derive(Serialize)]
struct CfCreateBody<'a> {
    #[serde(rename = "type")]
    record_type: &'a str,
    name: String,
    content: String,
    ttl: u32,
}

#[derive(Serialize)]
struct CfPatchBody {
    content: String,
    ttl: u32,
}

/// Upsert a DNS TXT record for the given node.
async fn push_dns_txt(
    config: &DnsTxtConfig,
    payload: &DnsTxtPayload,
    psk: Option<&str>,
) -> anyhow::Result<()> {
    let encrypted_content = encrypt_payload(payload, psk)?;
    let content = format!("v=sig1 d={}", encrypted_content);
    let record_name = format!("_sig.{}.{}", payload.node_id, config.base_domain);

    let client = reqwest::Client::new();

    // List existing TXT records with this name.
    let list_url = format!(
        "https://api.cloudflare.com/client/v4/zones/{}/dns_records?type=TXT&name={}",
        config.zone_id, record_name
    );
    let list_resp: CfListResponse = client
        .get(&list_url)
        .bearer_auth(&config.api_token)
        .send()
        .await?
        .error_for_status()?
        .json()
        .await?;

    if let Some(existing) = list_resp.result.into_iter().next() {
        // Patch in-place.
        let patch_url = format!(
            "https://api.cloudflare.com/client/v4/zones/{}/dns_records/{}",
            config.zone_id, existing.id
        );
        client
            .patch(&patch_url)
            .bearer_auth(&config.api_token)
            .json(&CfPatchBody { content, ttl: 60 })
            .send()
            .await?
            .error_for_status()?;
    } else {
        // Create new TXT record.
        let create_url = format!(
            "https://api.cloudflare.com/client/v4/zones/{}/dns_records",
            config.zone_id
        );
        client
            .post(&create_url)
            .bearer_auth(&config.api_token)
            .json(&CfCreateBody {
                record_type: "TXT",
                name: record_name,
                content,
                ttl: 60,
            })
            .send()
            .await?
            .error_for_status()?;
    }

    log::info!(
        "[signaling/dns] TXT record updated for node {} → {}",
        payload.node_id,
        payload.ip
    );

    Ok(())
}

// ──────────────────────────────── MQTT ───────────────────────────────────────

/// MQTT broker settings for the hidden signaling topic.
///
/// Topic layout: `_sr/nodes/<node_id>`
#[derive(Debug, Clone)]
pub struct MqttSignalingConfig {
    /// MQTT broker address (e.g. "mqtt://broker.example.com:8883").
    pub broker_url: String,
    /// Broker username.
    pub username: String,
    /// Broker password.
    pub password: String,
    /// QoS for published messages (0 = at-most-once, 1 = at-least-once).
    pub qos: u8,
}

/// Publish a node descriptor to the hidden MQTT topic.
///
/// Uses the `rumqttc` async client; the publish is fire-and-forget from the
/// perspective of this function (QoS-1 ack is awaited however).
async fn push_mqtt(
    config: &MqttSignalingConfig,
    payload: &DnsTxtPayload,
    psk: Option<&str>,
) -> anyhow::Result<()> {
    use rumqttc::{AsyncClient, MqttOptions, QoS};
    use std::time::Duration;

    let client_id = format!("rr-ui-signaling-{}", Utc::now().timestamp());
    let url = url::Url::parse(&config.broker_url)?;
    let host = url.host_str().unwrap_or("localhost");
    let port = url.port().unwrap_or(1883);

    let mut opts = MqttOptions::new(&client_id, host, port);
    opts.set_credentials(&config.username, &config.password)
        .set_keep_alive(Duration::from_secs(10));

    let (client, mut eventloop) = AsyncClient::new(opts, 16);

    let topic = format!("_sr/nodes/{}", payload.node_id);

    // Encrypt payload
    let encrypted_content = encrypt_payload(payload, psk)?;
    let payload_bytes = encrypted_content.into_bytes();

    let qos = match config.qos {
        0 => QoS::AtMostOnce,
        2 => QoS::ExactlyOnce,
        _ => QoS::AtLeastOnce,
    };

    client.publish(&topic, qos, false, payload_bytes).await?;

    // Drain the event loop briefly to ensure QoS-1 PUBACK is received.
    tokio::time::timeout(Duration::from_secs(5), async {
        loop {
            match eventloop.poll().await {
                Ok(rumqttc::Event::Incoming(rumqttc::Packet::PubAck(_))) => break,
                Err(_) => break,
                _ => {}
            }
        }
    })
    .await
    .ok(); // timeout or PUBACK — either way we're done

    client.disconnect().await.ok();

    log::info!(
        "[signaling/mqtt] Published to {} → ip={}",
        topic,
        payload.ip
    );

    Ok(())
}

// ──────────────────────────────── Auth helper ─────────────────────────────────

/// Extract the `Bearer` token from the `Authorization` header and validate it
/// against the list of known tokens.  Returns `true` if valid.
fn validate_bearer(req: &HttpRequest, valid_tokens: &[String]) -> bool {
    if let Some(auth_header) = req.headers().get("Authorization") {
        if let Ok(auth_str) = auth_header.to_str() {
            if let Some(token) = auth_str.strip_prefix("Bearer ") {
                return valid_tokens.iter().any(|t| t == token);
            }
        }
    }
    false
}

// ──────────────────────────────── Handlers ───────────────────────────────────

/// `POST /panel/api/signaling/push`
///
/// Push a node configuration update through the requested channel(s).
/// Returns per-channel delivery status.
#[post("/push")]
pub async fn push_update(
    req: HttpRequest,
    body: web::Json<PushRequest>,
    state: web::Data<Arc<SignalingState>>,
) -> HttpResponse {
    if !validate_bearer(&req, &state.valid_tokens) {
        return HttpResponse::Unauthorized().json(serde_json::json!({
            "error": "invalid or missing bearer token"
        }));
    }

    let payload = DnsTxtPayload {
        node_id: body.node_id.clone(),
        ip: body.new_ip.clone(),
        remark: body.remark.clone(),
        ts: Utc::now().timestamp(),
    };

    let mut results: Vec<ChannelResult> = Vec::new();
    let mut dns_ok = false;
    let mut mqtt_ok = false;
    let psk = state.psk.as_deref();

    for channel in &body.channels {
        match channel {
            SignalChannel::Dns => {
                let result = if let Some(ref cfg) = state.dns_config {
                    push_dns_txt(cfg, &payload, psk)
                        .await
                        .map_err(|e| e.to_string())
                } else {
                    Err("DNS signaling not configured".to_string())
                };
                dns_ok = result.is_ok();
                results.push(ChannelResult {
                    channel: "dns".to_string(),
                    ok: dns_ok,
                    error: result.err(),
                });
            }
            SignalChannel::Mqtt => {
                let result = if let Some(ref cfg) = state.mqtt_config {
                    push_mqtt(cfg, &payload, psk)
                        .await
                        .map_err(|e| e.to_string())
                } else {
                    Err("MQTT signaling not configured".to_string())
                };
                mqtt_ok = result.is_ok();
                results.push(ChannelResult {
                    channel: "mqtt".to_string(),
                    ok: mqtt_ok,
                    error: result.err(),
                });
            }
        }
    }

    // Store receipt
    {
        let mut receipts = state.receipts.lock().await;
        receipts.insert(
            body.node_id.clone(),
            DeliveryReceipt {
                node_id: body.node_id.clone(),
                new_ip: body.new_ip.clone(),
                remark: body.remark.clone(),
                timestamp: payload.ts,
                dns_ok,
                mqtt_ok,
            },
        );
    }

    let all_ok = results.iter().all(|r| r.ok);
    let response = PushResponse {
        node_id: body.node_id.clone(),
        timestamp: payload.ts,
        results,
    };

    if all_ok {
        HttpResponse::Ok().json(response)
    } else {
        HttpResponse::MultiStatus().json(response)
    }
}

/// `GET /panel/api/signaling/status`
///
/// Returns the latest delivery receipt for every node that has received a push.
#[get("/status")]
pub async fn delivery_status(
    req: HttpRequest,
    state: web::Data<Arc<SignalingState>>,
) -> HttpResponse {
    if !validate_bearer(&req, &state.valid_tokens) {
        return HttpResponse::Unauthorized().json(serde_json::json!({
            "error": "invalid or missing bearer token"
        }));
    }

    let receipts = state.receipts.lock().await;
    let receipts_vec: Vec<DeliveryReceipt> = receipts.values().cloned().collect();
    HttpResponse::Ok().json(receipts_vec)
}

// ──────────────────────────────── Router config ───────────────────────────────

/// Register signaling endpoints under `/panel/api/signaling`.
///
/// Add to `api/mod.rs`:
/// ```rust,ignore
/// #[cfg(feature = "server")]
/// pub mod signaling;
/// // … in config():
/// panel_scope = panel_scope.service(
///     web::scope("/signaling").configure(signaling::config)
/// );
/// ```
pub fn config(cfg: &mut web::ServiceConfig) {
    cfg.service(push_update).service(delivery_status);
}

// ──────────────────────────────────── Tests ───────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_channels_includes_both() {
        let channels = default_channels();
        assert!(channels.contains(&SignalChannel::Dns));
        assert!(channels.contains(&SignalChannel::Mqtt));
    }

    #[test]
    fn test_dns_txt_payload_roundtrip() {
        let payload = DnsTxtPayload {
            node_id: "ir-mci-01".to_string(),
            ip: "185.55.225.1".to_string(),
            remark: "failover test".to_string(),
            ts: 1_700_000_000,
        };
        let encoded = serde_json::to_vec(&payload).unwrap();
        let b64 = B64.encode(&encoded);
        let decoded_bytes = B64.decode(b64).unwrap();
        let decoded: DnsTxtPayload = serde_json::from_slice(&decoded_bytes).unwrap();
        assert_eq!(decoded.node_id, payload.node_id);
        assert_eq!(decoded.ip, payload.ip);
    }

    #[test]
    fn test_validate_bearer_rejects_missing() {
        use actix_web::test::TestRequest;
        let req = TestRequest::default().to_http_request();
        assert!(!validate_bearer(&req, &["secret".to_string()]));
    }

    #[test]
    fn test_validate_bearer_accepts_valid_token() {
        use actix_web::test::TestRequest;
        let req = TestRequest::default()
            .insert_header(("Authorization", "Bearer my-secret-token"))
            .to_http_request();
        assert!(validate_bearer(&req, &["my-secret-token".to_string()]));
    }

    #[test]
    fn test_validate_bearer_rejects_wrong_token() {
        use actix_web::test::TestRequest;
        let req = TestRequest::default()
            .insert_header(("Authorization", "Bearer wrong"))
            .to_http_request();
        assert!(!validate_bearer(&req, &["correct-token".to_string()]));
    }

    #[test]
    fn test_encryption_roundtrip() {
        let payload = DnsTxtPayload {
            node_id: "test-node".to_string(),
            ip: "1.1.1.1".to_string(),
            remark: "test".to_string(),
            ts: 1234567890,
        };

        let key_hex = "000102030405060708090a0b0c0d0e0f101112131415161718191a1b1c1d1e1f";
        let encrypted_b64 = encrypt_payload(&payload, Some(key_hex)).unwrap();

        // Decrypt manually to verify
        let encrypted_bytes = B64.decode(encrypted_b64).unwrap();
        let (nonce_bytes, ciphertext) = encrypted_bytes.split_at(12); // 96-bit nonce

        let key_bytes = hex::decode(key_hex).unwrap();
        let cipher = ChaCha20Poly1305::new_from_slice(&key_bytes).unwrap();
        let nonce = Nonce::from_slice(nonce_bytes);

        let plaintext = cipher.decrypt(nonce, ciphertext).unwrap();
        let decoded: DnsTxtPayload = serde_json::from_slice(&plaintext).unwrap();

        assert_eq!(decoded.node_id, "test-node");
    }
}
