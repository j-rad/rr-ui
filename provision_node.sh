#!/bin/bash
# EdgeRay Hardened Node Provisioner v2.0
# Mission: "One-Click Hardened Provisioner"
# Purpose: Rapid deployment of stealthy, high-performance edge nodes.

set -e

# --- Configuration ---
INSTALL_DIR="/usr/local/rr-ui"
BIN_DIR="$INSTALL_DIR/bin"
LOG_DIR="/var/log/rr-ui"
CONFIG_DIR="/etc/rr-ui"
GEOIP_DIR="/usr/share/xray"

# --- Colors ---
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m'

echo -e "${GREEN}====================================================${NC}"
echo -e "${GREEN}    EdgeRay Hardened Node Provisioner v2.0          ${NC}"
echo -e "${GREEN}====================================================${NC}"

# 1. Root Check
if [[ $EUID -ne 0 ]]; then
   echo -e "${RED}Error: This script must be run as root.${NC}"
   exit 1
fi

# 2. System Hardening (Sysctl)
echo -e "${YELLOW}[1/6] Applying Kernel Hardening...${NC}"
cat <<EOF > /etc/sysctl.d/99-edgeray-hardened.conf
# --- IP Spoofing & Network Protection ---
net.ipv4.conf.all.rp_filter = 1
net.ipv4.conf.default.rp_filter = 1
net.ipv4.conf.all.accept_source_route = 0
net.ipv4.conf.default.accept_source_route = 0

# --- Anti-Reconnaissance (ICMP) ---
# Ignore ICMP echo requests to hide from simple ping scans
net.ipv4.icmp_echo_ignore_all = 1
net.ipv4.icmp_echo_ignore_broadcasts = 1

# --- TCP/IP Stack Hardening ---
net.ipv4.tcp_syncookies = 1
net.ipv4.tcp_rfc1337 = 1
net.ipv4.tcp_timestamps = 0
net.ipv4.tcp_sack = 0
net.ipv4.tcp_dsack = 0
net.ipv4.tcp_fack = 0

# --- High-Performance Scaling ---
net.core.somaxconn = 65535
net.ipv4.tcp_max_syn_backlog = 65535
net.ipv4.ip_local_port_range = 1024 65535
net.ipv4.tcp_tw_reuse = 1
net.ipv4.tcp_fin_timeout = 15
net.core.netdev_max_backlog = 50000

# Connection tracking for 10k+ concurrent streams
net.netfilter.nf_conntrack_max = 2097152
net.netfilter.nf_conntrack_tcp_timeout_established = 3600

# --- BPF / XDP Security ---
net.core.bpf_jit_enable = 1
net.core.bpf_jit_harden = 2
net.core.bpf_jit_kallsyms = 0
EOF

sysctl -p /etc/sysctl.d/99-edgeray-hardened.conf || echo "Warn: Some sysctl parameters could not be applied."

# 3. Dependency Installation
echo -e "${YELLOW}[2/6] Installing Hardening Tools & Dependencies...${NC}"
if command -v apt-get >/dev/null; then
    apt-get update -qq
    apt-get install -y -qq curl nftables ipset bpftool clang llvm libelf-dev kmod
elif command -v yum >/dev/null; then
    yum install -y -q curl nftables ipset bpftool clang llvm libelf-dev
fi

# 4. BPF/XDP Environment Readiness
echo -e "${YELLOW}[3/6] Initializing BPF/XDP Virtual Filesystem...${NC}"
if ! mount | grep -q "/sys/fs/bpf"; then
    mount -t bpf bpf /sys/fs/bpf || true
fi

# 5. Firewall Configuration (nftables)
echo -e "${YELLOW}[4/6] Deploying Stealth Firewall (nftables)...${NC}"
cat <<EOF > /etc/nftables.conf
flush ruleset

table inet filter {
    chain input {
        type filter hook input priority 0; policy drop;

        # Allow established traffic
        ct state established,related accept

        # Stealth: Allow loopback
        iif "lo" accept

        # Allow SSH (Default 22 - Recommend changing)
        tcp dport 22 accept

        # EdgeRay Control Plane (gRPC)
        tcp dport 10085 accept

        # Allowed Proxy Ports (Broad range for flexibility)
        tcp dport { 80, 443, 8443, 2053 } accept
        udp dport { 443, 8443 } accept

        # Drop everything else silently (No ICMP unreachable)
    }

    chain forward {
        type filter hook forward priority 0; policy drop;
    }

    chain output {
        type filter hook output priority 0; policy accept;
    }
}
EOF
systemctl enable nftables
systemctl restart nftables

# 6. Directory Structure
echo -e "${YELLOW}[5/6] Creating Node Infrastructure...${NC}"
mkdir -p "$BIN_DIR" "$LOG_DIR" "$CONFIG_DIR" "$GEOIP_DIR"
chmod 755 "$INSTALL_DIR" "$BIN_DIR"
chmod 700 "$CONFIG_DIR"

# 7. Deployment Summary
echo -e "${GREEN}====================================================${NC}"
echo -e "${GREEN}    Provisioning Complete. Node is now Hardened.    ${NC}"
echo -e "${GREEN}====================================================${NC}"
echo -e "Next Steps:"
echo -e " 1. Transfer 'rustray' binary to: $BIN_DIR"
echo -e " 2. Register this Node ID in the EdgeRay Central Panel."
echo -e " 3. Verify connectivity via 'systemctl status nftables'."
