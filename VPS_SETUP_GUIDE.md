# RR-UI: VPS Setup, Configuration & Usage Guide

> **RR-UI** is a self-hosted admin panel for managing RustRay censorship circumvention proxy.  
> It runs as a single server binary with embedded web UI — no separate frontend build required.

---

## Table of Contents

1. [Prerequisites](#prerequisites)
2. [Building the Server Binary](#building-the-server-binary)
3. [Deploying to VPS](#deploying-to-vps)
4. [Installation on VPS](#installation-on-vps)
5. [Panel Configuration](#panel-configuration)
6. [SSL/TLS Certificate Setup](#ssltls-certificate-setup)
7. [RustRay Core Configuration](#rustray-core-configuration)
8. [Testing the Server](#testing-the-server)
9. [Managing the Service](#managing-the-service)
10. [User Management](#user-management)
11. [Firewall & Security](#firewall--security)
12. [Troubleshooting](#troubleshooting)

---

## Prerequisites

### Build Machine (Development)

- **Rust toolchain**: 1.85+ (edition 2024)
- **Protobuf compiler**: `protoc`
- **System libraries**: `libssl-dev`, `pkg-config`

### VPS (Production)

| Requirement | Minimum |
|---|---|
| **OS** | Ubuntu 22.04+, Debian 12+, CentOS 9, Alpine, OpenWrt |
| **CPU** | 1 vCPU |
| **RAM** | 512 MB |
| **Disk** | 50 MB (binary) + 30 MB (GeoIP assets) |
| **Open Ports** | Panel port (default: 2053), Inbound ports |

---

## Building the Server Binary

### Quick Build

```bash
cd edgeray-workspace/rr-ui

# Debug build (development)
cargo build -p rr-ui

# Release build (production)
cargo build -p rr-ui --release

# The binary is at: ../target/release/rr-ui
```

### Verify the Build

```bash
# Check binary size
ls -lh ../target/release/rr-ui

# Run tests
cargo test -p rr-ui

# Quick check
../target/release/rr-ui --help
```

### Cross-Compilation (if VPS has different architecture)

```bash
# For x86_64 Linux (most VPS)
cargo build --release --bin rr-ui --target x86_64-unknown-linux-gnu

# For ARM64 (Oracle Cloud, Hetzner ARM)
cargo build --release --bin rr-ui --target aarch64-unknown-linux-gnu
```

---

## Deploying to VPS

### Method 1: Automated Deployment (`deploy_manual.sh`)

This is the recommended method. It builds, transfers, and installs everything in one step.

```bash
cd edgeray-workspace/rr-ui

# Deploy to your VPS (you'll be prompted for SSH password once)
./deploy_manual.sh root@YOUR_VPS_IP
```

**What `deploy_manual.sh` does:**
1. ✅ Verifies and builds the RustRay core binary
2. ✅ Builds the rr-ui server binary in release mode
3. ✅ Strips symbols for smaller binary size
4. ✅ Establishes SSH connection (single password prompt)
5. ✅ Transfers `rr-ui`, `rustray`, and `install.sh` to VPS
6. ✅ Runs `install.sh` on the VPS (sets up systemd, user, directories)

### Method 2: Manual Transfer

```bash
# Build locally
cargo build --release --bin rr-ui

# Transfer to VPS
scp ../target/release/rr-ui root@YOUR_VPS_IP:/tmp/
scp rustray_core/rustray/rustray root@YOUR_VPS_IP:/tmp/
scp install.sh root@YOUR_VPS_IP:/tmp/

# SSH into VPS and run installer
ssh root@YOUR_VPS_IP
cd /tmp && chmod +x install.sh && chmod +x rustray && sudo RUSTRAY_NATIVE=1 ./install.sh
```

---

## Installation on VPS

The `install.sh` installer handles everything automatically:

### What It Installs

| Component | Location |
|---|---|
| rr-ui binary | `/usr/bin/rr-ui` |
| RustRay core binary | `/usr/local/rr-ui/bin/rustray` |
| Configuration & database | `/etc/rr-ui/` |
| SSL certificates | `/etc/rr-ui/certs/` |
| GeoIP assets | `/usr/share/xray/` |
| Logs | `/var/log/rr-ui/` |
| Systemd service | `/etc/systemd/system/rr-ui.service` |

### Installation Steps (Interactive)

When `install.sh` runs, it will ask you to configure:

1. **Admin Username** (default: `admin`)
2. **Admin Password** (auto-generated if left empty)
3. **Panel Port** (default: `2053`)
4. **Panel Secret Path** (default: `/panel`)
5. **SSL Certificate** (Automatic via Certbot *or* Manual path)

> ⚠️ **Save the credentials displayed at the end of installation!**

---

## Panel Configuration

### Access the Panel

After installation, access the panel at:

```
https://YOUR_VPS_IP:2053/panel
```

Or if you set a custom secret path:

```
https://YOUR_VPS_IP:2053/your-secret-path
```

### Default Credentials

| Field | Default |
|---|---|
| Username | `admin` (or what you set) |
| Password | Displayed during install |
| Port | `2053` |
| Path | `/panel` |

### Changing Settings After Install

```bash
# Change admin password
rr-ui setting --password "YourNewPassword"

# Change panel port
rr-ui setting --port 8443

# Change secret path
rr-ui setting --set-secret-path "/mysecretpath"

# Change username
rr-ui setting --username "myadmin"

# Disable 2FA
rr-ui reset-2fa

# Apply all changes: restart service
rr-ui restart
```

---

## SSL/TLS Certificate Setup

### Option A: Automatic (Let's Encrypt)

This is configured during `install.sh`. If you need to reconfigure:

```bash
# Obtain certificate
certbot certonly --standalone -d panel.yourdomain.com

# Apply to rr-ui
rr-ui cert --cert /etc/letsencrypt/live/panel.yourdomain.com/fullchain.pem \
           --key /etc/letsencrypt/live/panel.yourdomain.com/privkey.pem

rr-ui restart
```

### Option B: Manual Certificate

```bash
# Copy certificate files to VPS
scp your-cert.pem root@VPS_IP:/etc/rr-ui/certs/server.crt
scp your-key.pem  root@VPS_IP:/etc/rr-ui/certs/server.key

# Apply
rr-ui cert --cert /etc/rr-ui/certs/server.crt --key /etc/rr-ui/certs/server.key
rr-ui restart
```

---

## RustRay Core Configuration

RustRay is the proxy engine managed by rr-ui. It's automatically started by the panel.

### Configuration File

The core config is at `/etc/rr-ui/config.json`. You can manage it via the web panel or CLI:

```json
{
  "log": {
    "access": "access.log",
    "error": "error.log",
    "loglevel": "warning"
  },
  "inbounds": [],
  "outbounds": [
    { "tag": "direct", "protocol": "freedom" },
    { "tag": "blocked", "protocol": "blackhole" }
  ],
  "routing": {
    "domainStrategy": "IPIfNonMatch",
    "rules": []
  }
}
```

### Adding Inbounds via Web Panel

1. Login to the panel at `https://YOUR_IP:2053/panel`
2. Go to **Inbounds** page
3. Click **Add Inbound**
4. Configure protocol (VLESS, VMess, Trojan, etc.)
5. Set security (Reality, TLS, WebSocket, etc.)
6. Add client users
7. Save — RustRay core auto-restarts with new config

### Adding Inbounds via CLI

Use the web panel for inbound management. The CLI is for service-level operations only.

---

## Testing the Server

### Quick Health Check

```bash
# Check service status
rr-ui status

# Or via systemd
systemctl status rr-ui

# Check if panel responds
curl -k -o /dev/null -w "%{http_code}" https://127.0.0.1:2053/panel
# Expected: 200

# Check API health (should return 401 - auth required)
curl -k -o /dev/null -w "%{http_code}" https://127.0.0.1:2053/panel/api/server/status
# Expected: 401

# Check the decoy page (root path)
curl -k https://127.0.0.1:2053/
# Expected: nginx welcome page (camouflage)
```

### Verify RustRay Core

```bash
# Check if RustRay process is running
pgrep -a rustray

# Check RustRay version
/usr/local/rr-ui/bin/rustray --version

# Check logs for errors
journalctl -u rr-ui -n 50 --no-pager
```

### Test Proxy Connectivity

After adding an inbound:

```bash
# From client machine, test VLESS connection
# (Use your proxy client: V2rayN, Nekoray, Hiddify, etc.)
# Import the subscription link from the panel

# Quick TCP check from VPS:
ss -tlnp | grep rustray
# Should show your inbound ports
```

### Run Unit Tests (Development)

```bash
# Run all tests
cargo test -p rr-ui

# Run specific test
cargo test -p rr-ui -- test_client_serialization

# Run integration tests
cargo test -p rr-ui -- --test transport_test
```

---

## Managing the Service

### CLI Commands

```bash
# Interactive TUI Menu
rr-ui

# Service Management
rr-ui start          # Start the service
rr-ui stop           # Stop the service
rr-ui restart        # Restart the service
rr-ui status         # Show service status & metrics

# Enable/Disable autostart
rr-ui enable         # Enable on boot
rr-ui disable        # Disable on boot

# View Logs
rr-ui log            # View last 100 log lines
rr-ui log -l 500     # View last 500 lines

# Update
rr-ui update         # Update to latest version
```

### Systemd Commands

```bash
# View service status
systemctl status rr-ui

# View live logs
journalctl -u rr-ui -f

# Restart
systemctl restart rr-ui
```

---

## User Management

### Admin Account

```bash
# Reset admin password
rr-ui setting --password "NewStrongPassword"
rr-ui restart

# Reset password via API (requires running service)
rr-ui reset-password "NewPassword"

# Change username
rr-ui setting --username "newadmin"
rr-ui restart

# Disable 2FA (if locked out)
rr-ui reset-2fa
rr-ui restart
```

### Client Users (Proxy Users)

Managed via the web panel:

1. Navigate to **Inbounds** → Select an inbound
2. Click **Add Client**
3. Configure:
   - **Email**: User identifier
   - **Traffic Limit**: Max data usage (0 = unlimited)
   - **Expiry**: Account expiration date
   - **IP Limit**: Max concurrent connections
4. Share the **subscription link** or **QR code** with the user

### View Client Traffic

- **Web Panel**: Dashboard shows real-time traffic per client
- **CLI**: `rr-ui show-settings` for panel-level info

---

## Firewall & Security

### Port Configuration

| Port | Purpose | Required |
|---|---|---|
| `2053` (configurable) | Admin panel HTTPS | Yes |
| Inbound ports (e.g., `443`, `8443`) | Proxy traffic | Per inbound |
| `80` | Certbot HTTP challenge | For auto-SSL only |

### UFW (Ubuntu/Debian)

```bash
ufw allow 2053/tcp
ufw allow 2053/udp
ufw allow 443/tcp    # If using port 443 for inbounds
ufw allow 80/tcp     # For Certbot
```

### Firewalld (CentOS/Fedora)

```bash
firewall-cmd --permanent --add-port=2053/tcp
firewall-cmd --permanent --add-port=2053/udp
firewall-cmd --reload
```

### Security Hardening

The systemd service includes security hardening by default:

- Runs as non-root user (`rr-ui`)
- `ProtectSystem=full` — read-only system directories
- `ProtectHome=true` — hidden home directories
- `NoNewPrivileges=true` — prevents privilege escalation
- `CAP_NET_BIND_SERVICE` — allows binding low ports
- Rate limiting: 100 requests/minute/IP
- IP ban on repeated auth failures
- Decoy site camouflage (nginx page on root path)

---

## Troubleshooting

### Service Won't Start

```bash
# Check logs
journalctl -u rr-ui -n 100 --no-pager

# Common issues:
# 1. Port already in use
ss -tlnp | grep 2053
fuser -k 2053/tcp  # Kill conflicting process

# 2. Certificate file permissions
ls -la /etc/rr-ui/certs/
chown rr-ui:rr-ui /etc/rr-ui/certs/*

# 3. Database corruption
cd /etc/rr-ui && rr-ui setting --reset
```

### RustRay Core Not Starting

```bash
# Check if binary exists
ls -la /usr/local/rr-ui/bin/rustray

# Check if config is valid JSON
cat /etc/rr-ui/config.json | python3 -m json.tool

# Try running manually
/usr/local/rr-ui/bin/rustray run -c /etc/rr-ui/config.json
```

### Panel Not Accessible

```bash
# Check binding
ss -tlnp | grep rr-ui

# Check firewall
ufw status              # Ubuntu
firewall-cmd --list-all  # CentOS

# Check external access
curl -k https://YOUR_IP:2053/panel
```

### Reset Everything

```bash
rr-ui stop
rr-ui setting --reset
rr-ui restart
# This resets all settings to defaults (admin/auto-generated password)
```

---

## Architecture Overview

```
VPS
├── /usr/bin/rr-ui              ← Server binary (with embedded web UI)
├── /usr/local/rr-ui/bin/rustray ← RustRay proxy core
├── /etc/rr-ui/
│   ├── config.json             ← RustRay configuration (auto-managed)
│   ├── data/rr-ui.db           ← SurrealDB database
│   └── certs/                  ← SSL certificates
├── /usr/share/xray/
│   ├── geoip.dat               ← GeoIP database
│   └── geosite.dat             ← Geo-site database
└── /etc/systemd/system/rr-ui.service ← Systemd unit
```

**Request flow:**
```
Client Browser → https://VPS:2053/panel → rr-ui (Actix-Web)
                                            ├── Serves embedded UI assets
                                            ├── REST API for panel operations
                                            └── Manages RustRay core via gRPC
                                                └── RustRay → Proxy traffic
```
