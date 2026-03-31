#!/bin/bash
set -e

# RustRay Native Deployment Script
# Optimized for rr-ui with native rustray binary

# Configuration
BINARY_NAME="rr-ui"
RUSTRAY_BINARY="rustray_core/rustray/rustray"
LOCAL_BINARY_PATH="../target/release/$BINARY_NAME"
WEB_DIR="./web"

# UI Configuration
# Set to "true" to use Dioxus UI instead of static web build
# Note: Dioxus UI is currently in development (see DIOXUS_UI_STATUS.md)
USE_DIOXUS_UI="${USE_DIOXUS_UI:-false}"


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

# 0. Verify RustRay binary exists
log_info "Verifying RustRay binary..."
if [ ! -f "$RUSTRAY_BINARY" ]; then
    log_error "RustRay binary not found at $RUSTRAY_BINARY"
    log_info "Please build RustRay first: cd rustray_core && cargo build --release --offline"
    exit 1
fi

# Verify execution permissions
if [ ! -x "$RUSTRAY_BINARY" ]; then
    log_warn "Setting execution permissions on RustRay binary..."
    chmod +x "$RUSTRAY_BINARY"
fi

log_success "RustRay binary verified"

# 1. Build Frontend
if [ "$USE_DIOXUS_UI" = "true" ]; then
    log_info "Building Dioxus UI (Manual Mode)..."
    
    # Check for wasm-bindgen
    if ! command -v wasm-bindgen &> /dev/null; then
        log_error "wasm-bindgen CLI not found. Install with: cargo install wasm-bindgen-cli --version 0.2.106"
        exit 1
    fi
    
    # A. Build Client (Wasm)
    log_info "Compiling WASM..."
    cargo build --target wasm32-unknown-unknown --features web --release --no-default-features --offline
    
    log_info "Running wasm-bindgen..."
    mkdir -p web/static
    wasm-bindgen target/wasm32-unknown-unknown/release/rr-ui.wasm --out-dir web/static --target web --no-typescript
    
    log_success "Dioxus Client built and assets generated in web/static"

    # B. Build Server Binary (x86_64)
    log_info "Building Dioxus Server Binary..."
    # Ensure server feature is enabled which bundles the web/static assets
    cargo build --release --bin rr-ui --features "server" --offline
    
    CLI_BINARY_PATH="../target/release/rr-ui"
else
    # Standard Static Web Build (Vue/Svelte/etc)
    log_info "Building Static Web Frontend..."
    if [ -d "$WEB_DIR" ]; then
        cd "$WEB_DIR"
        if ! command -v pnpm &> /dev/null; then
            log_error "pnpm not found. Please install it."
            exit 1
        fi
        pnpm install
        pnpm build
        cd ..
        log_success "Frontend built"
    else
        log_warn "Web directory not found at $WEB_DIR. Skipping frontend build."
    fi
    
    # Standard Server Build
    log_info "Building Backend (Release with Server features)..."
    cargo build --release --bin rr-ui --features "server" --offline
    CLI_BINARY_PATH="./target/release/rr-ui"
fi

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
