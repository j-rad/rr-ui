use crate::models::{Inbound, ProtocolSettings};
use base64::{Engine as _, engine::general_purpose::STANDARD};
use serde_json::json;
use std::fmt::Write;

pub struct SubscriptionGenerator;

impl SubscriptionGenerator {
    pub fn generate_links(inbounds: &[Inbound], host: &str) -> String {
        let mut buffer = String::with_capacity(4096);

        for inbound in inbounds {
            if !inbound.enable {
                continue;
            }

            if let Some(clients) = inbound.settings.clients() {
                for client in clients {
                    if !client.enable {
                        continue;
                    }

                    // Extract common fields
                    let uuid = client.id.as_deref().unwrap_or_default();
                    let remark = format!(
                        "{}-{}",
                        inbound.tag,
                        client.email.as_deref().unwrap_or("user")
                    );

                    match &inbound.settings {
                        ProtocolSettings::Vless(_) => {
                            // vless://uuid@host:port?params#remark
                            let _ = write!(
                                buffer,
                                "vless://{}@{}:{}?type={}&security={}&fp={}&sni={}&sid={}#{}\n",
                                uuid,
                                host,
                                inbound.port,
                                inbound.stream_settings.network,
                                inbound.stream_settings.security,
                                inbound
                                    .stream_settings
                                    .total_fingerprint()
                                    .unwrap_or_default(),
                                inbound
                                    .stream_settings
                                    .total_sni()
                                    .unwrap_or(host.to_string()), // Default SNI to host if missing
                                inbound.stream_settings.total_short_id().unwrap_or_default(),
                                urlencoding::encode(&remark)
                            );
                        }
                        ProtocolSettings::Vmess(_) => {
                            // VMess uses a base64 encoded JSON object
                            let vmess_json = json!({
                                "v": "2",
                                "ps": remark,
                                "add": host,
                                "port": inbound.port,
                                "id": uuid,
                                "aid": 0,
                                "scy": "auto",
                                "net": inbound.stream_settings.network,
                                "type": "none",
                                "host": inbound.stream_settings.total_host().unwrap_or_default(),
                                "path": inbound.stream_settings.total_path().unwrap_or_default(),
                                "tls": if inbound.stream_settings.security == "tls" { "tls" } else { "" },
                                "sni": inbound.stream_settings.total_sni().unwrap_or_default(),
                                "alpn": inbound.stream_settings.total_alpn().unwrap_or_default(),
                                "fp": inbound.stream_settings.total_fingerprint().unwrap_or_default(),
                            });
                            let vmess_b64 = STANDARD.encode(vmess_json.to_string());
                            let _ = write!(buffer, "vmess://{}\n", vmess_b64);
                        }
                        ProtocolSettings::Trojan(_) => {
                            // trojan://password@host:port?security=...&sni=...#remark
                            let password = client.password.as_deref().unwrap_or_default();
                            let _ = write!(
                                buffer,
                                "trojan://{}@{}:{}?security={}&sni={}#{}\n",
                                password,
                                host,
                                inbound.port,
                                inbound.stream_settings.security,
                                inbound
                                    .stream_settings
                                    .total_sni()
                                    .unwrap_or(host.to_string()),
                                urlencoding::encode(&remark)
                            );
                        }
                        // Add other protocols as needed
                        _ => {}
                    }
                }
            }
        }

        // Return Base64 of the entire list for standard subscription
        STANDARD.encode(buffer)
    }

    pub fn generate_clash_yaml(inbounds: &[Inbound], host: &str) -> String {
        let mut buffer = String::with_capacity(8192);

        // Access log level
        buffer.push_str("port: 7890\n");
        buffer.push_str("socks-port: 7891\n");
        buffer.push_str("allow-lan: true\n");
        buffer.push_str("mode: Rule\n");
        buffer.push_str("log-level: info\n");
        buffer.push_str("external-controller: :9090\n");
        buffer.push_str("proxies:\n");

        for inbound in inbounds {
            if !inbound.enable {
                continue;
            }

            if let Some(clients) = inbound.settings.clients() {
                for client in clients {
                    if !client.enable {
                        continue;
                    }

                    let uuid = client.id.as_deref().unwrap_or_default();
                    let remark = format!(
                        "{}-{}",
                        inbound.tag,
                        client.email.as_deref().unwrap_or("user")
                    );

                    match &inbound.settings {
                        ProtocolSettings::Vless(_) => {
                            let _ = write!(
                                buffer,
                                "  - name: {}\n    type: vless\n    server: {}\n    port: {}\n    uuid: {}\n    udp: true\n    tls: {}\n    skip-cert-verify: true\n    servername: {}\n    network: {}\n",
                                remark,
                                host,
                                inbound.port,
                                uuid,
                                if inbound.stream_settings.security == "tls" {
                                    "true"
                                } else {
                                    "false"
                                },
                                inbound
                                    .stream_settings
                                    .total_sni()
                                    .unwrap_or(host.to_string()),
                                inbound.stream_settings.network
                            );
                            if let Some(fp) = inbound.stream_settings.total_fingerprint() {
                                let _ = write!(buffer, "    client-fingerprint: {}\n", fp);
                            }
                            // Add flow if applicable (xtls-rprx-vision)
                            if let Some(flow) = &inbound
                                .stream_settings
                                .reality_settings
                                .as_ref()
                                .map(|r| r.short_ids.clone())
                            {
                                // Reality logic typically goes here, simplifying for basic VLESS
                            }
                        }
                        ProtocolSettings::Vmess(_) => {
                            let _ = write!(
                                buffer,
                                "  - name: {}\n    type: vmess\n    server: {}\n    port: {}\n    uuid: {}\n    alterId: 0\n    cipher: auto\n    udp: true\n    tls: {}\n    skip-cert-verify: true\n    servername: {}\n    network: {}\n",
                                remark,
                                host,
                                inbound.port,
                                uuid,
                                if inbound.stream_settings.security == "tls" {
                                    "true"
                                } else {
                                    "false"
                                },
                                inbound
                                    .stream_settings
                                    .total_sni()
                                    .unwrap_or(host.to_string()),
                                inbound.stream_settings.network
                            );
                            if let Some(path) = inbound.stream_settings.total_path() {
                                let _ = write!(buffer, "    ws-opts:\n      path: {}\n", path);
                                if let Some(host_header) = inbound.stream_settings.total_host() {
                                    let _ = write!(
                                        buffer,
                                        "      headers:\n        Host: {}\n",
                                        host_header
                                    );
                                }
                            }
                        }
                        ProtocolSettings::Trojan(_) => {
                            let password = client.password.as_deref().unwrap_or_default();
                            let _ = write!(
                                buffer,
                                "  - name: {}\n    type: trojan\n    server: {}\n    port: {}\n    password: {}\n    udp: true\n    sni: {}\n    skip-cert-verify: true\n",
                                remark,
                                host,
                                inbound.port,
                                password,
                                inbound
                                    .stream_settings
                                    .total_sni()
                                    .unwrap_or(host.to_string())
                            );
                        }
                        _ => {}
                    }
                }
            }
        }

        // Add proxy groups
        buffer.push_str("proxy-groups:\n");
        buffer.push_str("  - name: PROXY\n    type: select\n    proxies:\n      - AUTO\n");

        // Re-iterate to add proxy names to group
        for inbound in inbounds {
            if !inbound.enable {
                continue;
            }
            if let Some(clients) = inbound.settings.clients() {
                for client in clients {
                    if !client.enable {
                        continue;
                    }
                    let remark = format!(
                        "{}-{}",
                        inbound.tag,
                        client.email.as_deref().unwrap_or("user")
                    );
                    let _ = write!(buffer, "      - {}\n", remark);
                }
            }
        }

        buffer.push_str("  - name: AUTO\n    type: url-test\n    url: 'http://www.gstatic.com/generate_204'\n    interval: 300\n    proxies:\n");
        for inbound in inbounds {
            if !inbound.enable {
                continue;
            }
            if let Some(clients) = inbound.settings.clients() {
                for client in clients {
                    if !client.enable {
                        continue;
                    }
                    let remark = format!(
                        "{}-{}",
                        inbound.tag,
                        client.email.as_deref().unwrap_or("user")
                    );
                    let _ = write!(buffer, "      - {}\n", remark);
                }
            }
        }

        // Rules
        buffer.push_str("rules:\n");
        buffer.push_str("  - MATCH,PROXY\n");

        buffer
    }
}
