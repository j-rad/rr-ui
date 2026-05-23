#!/bin/bash
set -e

# RustRay Native Deployment Script
# Optimized for rr-ui with native rustray binary

# Configuration
BINARY_NAME="rr-ui"
RUSTRAY_BINARY="rustray_core/rustray/rustray"
LOCAL_BINARY_PATH="../target/release/$BINARY_NAME"


# Colors
GREEN='\033[0;32m'
BLUE='\033[0;34m'
RED='\033[0;31m'
YELLOW='\033[1;33m'
CYAN='\033[0;36m'
NC='\033[0m'
BOLD='\033[1m'

log_info() { echo -e "${BLUE}[INFO]${NC} $1"; }
log_success() { echo -e "${GREEN}[✓]${NC} $1"; }
log_warn() { echo -e "${YELLOW}[!]${NC} $1"; }
log_error() { echo -e "${RED}[✗]${NC} $1"; }

print_banner() {
    echo -e "${CYAN}${BOLD}"
    cat << 'EOF'
    ____             __  ____             
   / __ \__  _______/ /_/ __ \____ ___  __
  / /_/ / / / / ___/ __/ /_/ / __ `/ / / /
 / _, _/ /_/ (__  ) /_/ _, _/ /_/ / /_/ / 
/_/ |_|\__,_/____/\__/_/ |_|\__,_/\__, /  
                                 /____/   
     Native RustRay Deployment
EOF
    echo -e "${NC}"
}

# Check for arguments
if [ "$#" -ne 1 ]; then
    echo "Usage: $0 <user@server_ip>"
    echo "Example: $0 root@192.168.1.10"
    exit 1
fi

SERVER_DEST=$1

print_banner
log_info "Starting Native RustRay Deployment to $SERVER_DEST"

# 0. Verify and build RustRay binary
log_info "Verifying RustRay binary..."
if [ ! -f "$RUSTRAY_BINARY" ]; then
    log_warn "RustRay binary not found at $RUSTRAY_BINARY"
    log_info "Attempting to build RustRay core automatically..."
    (cd rustray_core && cargo build --release --offline)
    if [ ! -f "$RUSTRAY_BINARY" ]; then
        log_error "Failed to build rustray manually, executable not found."
        exit 1
    fi
fi

# Verify execution permissions
if [ ! -x "$RUSTRAY_BINARY" ]; then
    log_warn "Setting execution permissions on RustRay binary..."
    chmod +x "$RUSTRAY_BINARY"
fi

log_success "RustRay binary verified"

# 1. Build Server Binary (server-only, no standalone WASM client)
log_info "Building RR-UI Server Binary (release)..."
cargo build --release --bin rr-ui --features "server" --offline

CLI_BINARY_PATH="../target/release/rr-ui"

if [ ! -f "$CLI_BINARY_PATH" ]; then
    log_error "Binary not found at $CLI_BINARY_PATH after build."
    exit 1
fi

# Copy to expected name for deployment if different
if [ "$CLI_BINARY_PATH" != "$LOCAL_BINARY_PATH" ]; then
    cp "$CLI_BINARY_PATH" "$LOCAL_BINARY_PATH"
fi
log_success "Backend built successfully"

# 3. Optimize binary (optional)
log_info "Binary optimization..."
# read -p "Strip binary to reduce size? (y/n) " -n 1 -r
# echo
# if [[ $REPLY =~ ^[Yy]$ ]]; then
    strip "$LOCAL_BINARY_PATH"
    strip "$RUSTRAY_BINARY"
    log_success "Binaries stripped"
# fi

# SSH Configuration for Multiplexing
SOCKET_DIR="./.ssh_sockets"
mkdir -p "$SOCKET_DIR"
CONTROL_SOCKET="$SOCKET_DIR/socket-%r@%h:%p"
SSH_OPTS="-o ControlPath=$CONTROL_SOCKET -o ControlMaster=auto -o ControlPersist=600"

unset DISPLAY
unset SSH_ASKPASS

# Function to establish master connection
establish_connection() {
    log_info "Establishing SSH Master Connection to $SERVER_DEST..."
    log_info "You will be asked for the password ONCE."
    
    # -M: master mode
    # -f: go to background
    # -N: do not execute remote command
    ssh -M -S "$CONTROL_SOCKET" -fN "$SERVER_DEST"
    
    if [ $? -eq 0 ]; then
        log_success "Connection established."
    else
        log_error "Failed to connect."
        exit 1
    fi
}

# Ensure connection is closed on script exit
cleanup() {
    if [ -S "$CONTROL_SOCKET" ]; then
        log_info "Closing SSH connection..."
        ssh -S "$CONTROL_SOCKET" -O exit "$SERVER_DEST" 2>/dev/null || true
    fi
}
trap cleanup EXIT

# Start connection
establish_connection

# 4. Transfer to Server
log_info "Transferring files to server..."
DEPLOY_DIR="/tmp/rr-ui"

# We use the existing socket for all subsequent commands
# Note: scp needs -o ControlPath=...
ssh -S "$CONTROL_SOCKET" "$SERVER_DEST" "mkdir -p $DEPLOY_DIR/rustray"

# Transfer files
scp -o ControlPath="$CONTROL_SOCKET" "$LOCAL_BINARY_PATH" "$SERVER_DEST:$DEPLOY_DIR/"
scp -o ControlPath="$CONTROL_SOCKET" "$RUSTRAY_BINARY" "$SERVER_DEST:$DEPLOY_DIR/rustray/"
scp -o ControlPath="$CONTROL_SOCKET" "install.sh" "$SERVER_DEST:$DEPLOY_DIR/"

log_success "Files transferred"

# 5. Execute Installation on Server
log_info "Executing installation on server..."
ssh -t -S "$CONTROL_SOCKET" "$SERVER_DEST" "cd $DEPLOY_DIR && chmod +x install.sh && chmod +x rustray/rustray && sudo RUSTRAY_NATIVE=1 ./install.sh"

# 6. Cleanup remote
log_info "Cleaning up temporary files..."
ssh -S "$CONTROL_SOCKET" "$SERVER_DEST" "rm -rf $DEPLOY_DIR"

echo ""
log_success "═══════════════════════════════════════════════════"
log_success "  RustRay Deployment Complete!"
log_success "═══════════════════════════════════════════════════"
echo ""
log_info "Next steps:"
log_info "  1. Check status: ssh $SERVER_DEST 'systemctl status rr-ui'"
log_info "  2. Access panel at https://$SERVER_DEST:2053/psb"
echo ""
