# RustRay Native Deployment - Quick Start

## Prerequisites

- **Local Machine**: Rust toolchain, pnpm, RustRay binary built in `rustray_core/rustray`
- **Target Server**: Fresh Linux instance (Debian/Ubuntu/CentOS/Fedora/Arch/Alpine/OpenWrt)
- **SSH Access**: Password-less SSH or SSH key configured

## One-Command Deployment

```bash
./deploy_manual.sh user@server_ip
```

This single command will:

1. ✅ Verify RustRay binary exists and is executable
2. ✅ Build Svelte frontend (pnpm build)
3. ✅ Build Rust backend (cargo build --release)
4. ✅ Transfer binaries to server
5. ✅ Run distribution-agnostic installation
6. ✅ Configure systemd service with pre-flight validation
7. ✅ Start rr-ui service
8. ✅ Cleanup temporary files

## Manual Installation (On Server)

```bash
sudo ./install.sh
```

Follow the interactive prompts for:

- Admin username (default: admin)
- Admin password (auto-generated or custom)
- Panel port (default: 2053)
- Panel secret path (default: /panel)

## Verification

```bash
# Run integrity tests
sudo ./test_deploy_integrity.sh

# Open TUI menu
rr-ui

# Check service status
rr-ui status
systemctl status rr-ui
```

## TUI Menu Options

```
 0. Exit
 1. Start Service          ← systemctl start rr-ui
 2. Stop Service           ← systemctl stop rr-ui
 3. Restart Service        ← systemctl restart rr-ui + config validation
 4. Service Status         ← Real-time metrics (uptime, CPU, memory)
 5. Reset Admin Credentials
 6. Change Panel Port
 7. Change Panel Path
 8. Reset 2FA
 9. SSL Certificate Management
10. Enable BBR             ← kernel TCP optimization
11. Network Speedtest
12. Update Geo Files       ← Download geoip.dat/geosite.dat with progress
13. System Update
14. Enable Autostart
15. Disable Autostart
16. View Logs
```

## Key Features

### Pre-Flight Config Validation

Every service restart runs `rustray -test -c config.json` before starting.
Prevents service failures from bad configurations.

### Auto-Configured Environment

- `XRAY_LOCATION_ASSET=/usr/share/xray` for geo-assets
- RustRay binary at `/usr/local/rr-ui/bin/rustray`
- Config directory at `/etc/rr-ui`
- Dedicated `rr-ui` system user with minimal privileges

### Distribution Support

Automatically detects and uses:

- apt-get (Debian/Ubuntu)
- dnf/yum (CentOS/Fedora)
- pacman (Arch)
- apk (Alpine)
- opkg (OpenWrt)

### Security Hardening

- CAP_NET_BIND_SERVICE + CAP_NET_ADMIN only
- ProtectSystem=full
- ProtectHome=true
- NoNewPrivileges=true

## Troubleshooting

### RustRay binary not found

```bash
# Build RustRay first
cd rustray_core
cargo build --release
chmod +x target/release/rustray
cp target/release/rustray rustray
cd ..
```

### Text file busy (Binary locked)

If the installer fails with `Text file busy`, it means a process is still using the binary (e.g., a TUI session or a hung service).

```bash
# Force kill any lingering processes
sudo pkill -9 rr-ui
sudo pkill -9 rustray

# Or detect what is using the binary
sudo fuser -v /usr/bin/rr-ui
```

### Service won't start

```bash
# Check logs
journalctl -u rr-ui -n 50

# Check config validation
sudo /usr/local/rr-ui/bin/rustray -test -c /etc/rr-ui/rustray_config.json

# Check binary permissions
ls -la /usr/bin/rr-ui
ls -la /usr/local/rr-ui/bin/rustray
```

### Port already in use

```bash
# Change port via TUI
rr-ui
# Select: 6. Change Panel Port

# Or via CLI
rr-ui setting --port 8080
systemctl restart rr-ui
```

## Access Panel

```text
URL: http://YOUR_SERVER_IP:2053/panel
Username: admin (or custom)
Password: (shown during installation)
```

## Documentation

- `RUSTRAY_DEPLOYMENT_SUMMARY.md` - Detailed implementation documentation
- `GEO_ORCHESTRATION_SUMMARY.md` - Geo-asset management documentation
- Source code in `src/` with inline documentation

## Support

For issues or questions, check:

1. Service logs: `journalctl -u rr-ui -f`
2. TUI status: `rr-ui status`
3. Integrity tests: `sudo ./test_deploy_integrity.sh`
