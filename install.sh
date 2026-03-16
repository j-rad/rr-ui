#!/bin/bash
set -e

# RustRay Native Installation Script for rr-ui
# Distribution-agnostic installer for Debian/Ubuntu/CentOS/Fedora/Arch/Alpine/OpenWrt

# Colors
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
CYAN='\033[0;36m'
MAGENTA='\033[0;35m'
NC='\033[0m'
BOLD='\033[1m'

# ASCII Art Banner
print_banner() {
    echo -e "${CYAN}${BOLD}"
    cat << "EOF"
    ____  ____        __  ______
   / __ \/ __ \      / / / /  _/
  / /_/ / /_/ /_____/ / / // /  
 / _, _/ _, _/_____/ /_/ // /   
/_/ |_/_/ |_|      \____/___/   
                                
    RustRay Native Installer
EOF
    echo -e "${NC}"
}

# Logging functions
log_info() { echo -e "${BLUE}[INFO]${NC} $1"; }
log_success() { echo -e "${GREEN}[✓]${NC} $1"; }
log_error() { echo -e "${RED}[✗]${NC} $1"; }
log_warn() { echo -e "${YELLOW}[!]${NC} $1"; }

# Check if running as root
check_root() {
    if [ "$EUID" -ne 0 ]; then
        log_error "This script must be run as root"
        exit 1
    fi
}

# Detect distribution
detect_distro() {
    if [ -f /etc/os-release ]; then
        . /etc/os-release
        echo "$ID"
    elif [ -f /etc/openwrt_release ]; then
        echo "openwrt"
    else
        echo "unknown"
    fi
}

# Install system dependencies based on distribution
install_dependencies() {
    local distro=$(detect_distro)
    log_info "Detected distribution: $distro"
    log_info "Installing system dependencies..."
    
    case "$distro" in
        ubuntu|debian)
            apt-get update
            apt-get install -y ca-certificates libssl-dev curl tar unzip openssl certbot net-tools psmisc
            ;;
        centos|rhel|fedora|rocky|almalinux)
            if command -v dnf &> /dev/null; then
                dnf install -y ca-certificates openssl-devel curl tar unzip openssl certbot
            else
                yum install -y ca-certificates openssl-devel curl tar unzip openssl certbot
            fi
            ;;
        arch|manjaro)
            pacman -Sy --noconfirm ca-certificates openssl curl tar unzip certbot
            ;;
        alpine)
            apk add --no-cache ca-certificates openssl-dev curl tar unzip openssl certbot
            ;;
        openwrt)
            opkg update
            opkg install ca-certificates libopenssl curl tar unzip openssl-util
            # OpenWrt might not have certbot easily, skip it for now or assume acme.sh? 
            # Instruction said "Use certbot OR acme.sh". For OpenWrt usually acme.sh.
            # But adhering to "Mandatory SSL", we'll assume manual path or certbot if available.
            ;;
        *)
            log_warn "Unknown distribution. Please install: ca-certificates, libssl, curl, tar, unzip, certbot manually"
            ;;
    esac
    
    log_success "Dependencies installed"
}

# Detect init system
detect_init_system() {
    if [ -f "/bin/systemctl" ] || [ -f "/usr/bin/systemctl" ]; then
        echo "systemd"
    elif [ -f "/sbin/procd" ]; then
        echo "procd"
    else
        echo "unknown"
    fi
}

# Stop existing service
stop_existing_service() {
    local init_system=$(detect_init_system)
    
    if [ "$init_system" = "systemd" ]; then
        if systemctl is-active --quiet rr-ui 2>/dev/null; then
            log_info "Stopping existing service..."
            systemctl stop rr-ui
            sync
            
            # Check for lingering processes
            if pgrep -x "rr-ui" > /dev/null; then
                log_warn "Service did not stop cleanly. Sending SIGKILL..."
                pkill -9 -x "rr-ui" || true
                sleep 1
            fi
            log_success "Service stopped"
        fi
    elif [ "$init_system" = "procd" ]; then
        if [ -f "/etc/init.d/rr-ui" ]; then
            log_info "Stopping existing service..."
            /etc/init.d/rr-ui stop 2>/dev/null || true
            sync
            log_success "Service stopped"
        fi
    fi
}

# Create system user
create_user() {
    log_info "Creating system user..."
    
    if ! id -u rr-ui &> /dev/null; then
        useradd -r -s /bin/false rr-ui 2>/dev/null || useradd -r -s /sbin/nologin rr-ui
        log_success "User rr-ui created"
    else
        log_info "User rr-ui already exists"
    fi
}

# Create directory structure
create_directories() {
    log_info "Creating directory structure..."
    
    # Main directories
    mkdir -p /etc/rr-ui/certs
    mkdir -p /usr/local/rr-ui/bin
    mkdir -p /usr/share/xray
    mkdir -p /var/log/rr-ui
    
    log_success "Directory structure created"
}

# Install RustRay binary
install_rustray() {
    log_info "Installing RustRay binary..."
    
    # Check for local rustray binary (from deployment)
    if [ -f "./rustray/rustray" ]; then
        log_info "Using deployed RustRay binary"
        install -m 755 ./rustray/rustray /usr/local/rr-ui/bin/rustray
        
        # Create symlink for easy access
        ln -sf /usr/local/rr-ui/bin/rustray /usr/local/bin/rustray
        
        # Verify it works - rustray specific check
        if /usr/local/rr-ui/bin/rustray --version 2>&1 | grep -q "rustray\|RustRay\|version"; then
            log_success "RustRay binary installed and verified"
        else
            log_warn "RustRay binary installed but version check unclear"
        fi
    else
        log_error "RustRay binary not found at ./rustray/rustray"
        log_error "Please ensure rustray binary is included in deployment"
        exit 1
    fi
}

# Install rr-ui binary
install_binary() {
    log_info "Installing rr-ui binary..."
    
    # Check for local binary first
    if [ -f "./rr-ui" ]; then
        install -m 755 ./rr-ui /usr/bin/rr-ui
    elif [ -f "./target/release/rr-ui" ]; then
        install -m 755 ./target/release/rr-ui /usr/bin/rr-ui
    else
        log_error "rr-ui binary not found"
        exit 1
    fi
    
    # Set capabilities if setcap is available
    if command -v setcap &> /dev/null; then
        setcap cap_net_bind_service,cap_net_admin=+ep /usr/bin/rr-ui
        log_success "Capabilities set on rr-ui binary"
    fi
    
    log_success "rr-ui binary installed to /usr/bin/rr-ui"
}

# Create systemd service for RustRay
create_systemd_service() {
    log_info "Creating systemd service..."
    
    cat > /etc/systemd/system/rr-ui.service << EOF
[Unit]
Description=RR-UI Panel Service with Native RustRay
After=network.target
Wants=network-online.target

[Service]
Type=simple
User=rr-ui
Group=rr-ui
WorkingDirectory=/etc/rr-ui
RuntimeDirectory=rr-ui

# Main rr-ui service
ExecStart=/usr/bin/rr-ui run

# Allow writing to application directory
ReadWritePaths=/etc/rr-ui

Restart=always
RestartSec=5
StandardOutput=journal
StandardError=journal
LimitNOFILE=1048576

# Environment
Environment="XRAY_LOCATION_ASSET=/usr/share/xray"
Environment="RUST_LOG=info"

# Security Hardening
AmbientCapabilities=CAP_NET_BIND_SERVICE CAP_NET_ADMIN
CapabilityBoundingSet=CAP_NET_BIND_SERVICE CAP_NET_ADMIN
ProtectSystem=full
ProtectHome=true
NoNewPrivileges=true

[Install]
WantedBy=multi-user.target
EOF
    
    systemctl daemon-reload
    systemctl enable rr-ui
    log_success "Systemd service created and enabled"
}

# Create OpenWrt init script
create_openwrt_service() {
    log_info "Creating OpenWrt init script..."
    
    cat > /etc/init.d/rr-ui << 'EOF'
#!/bin/sh /etc/rc.common

START=99
STOP=10

USE_PROCD=1

start_service() {
    procd_open_instance
    procd_set_param command /usr/bin/rr-ui run
    procd_set_param env XRAY_LOCATION_ASSET=/usr/share/xray
    procd_set_param respawn
    procd_set_param stdout 1
    procd_set_param stderr 1
    procd_close_instance
}
EOF
    
    chmod +x /etc/init.d/rr-ui
    /etc/init.d/rr-ui enable
    log_success "OpenWrt init script created and enabled"
}

# Configure SSL
setup_ssl() {
    echo -e "${YELLOW}--------------------------------------------------${NC}"
    echo -e "${YELLOW} MANDATORY SSL CONFIGURATION ${NC}"
    echo -e "${YELLOW}--------------------------------------------------${NC}"
    echo "1. Automatic (Certbot/Let's Encrypt)"
    echo "2. Manual (Existing Certificate Files)"
    
    while true; do
        read -p "Select SSL Method [1/2]: " ssl_method
        case $ssl_method in
            1)
                log_info "Automatic SSL Selection"
                read -p "Enter your domain name (e.g. panel.example.com): " PANEL_DOMAIN
                
                if [ -z "$PANEL_DOMAIN" ]; then
                    log_error "Domain is required."
                    continue
                fi
                
                log_info "Running Certbot..."
                # Stop any service on port 80 just in case
                if command -v systemctl &> /dev/null; then
                    systemctl stop nginx 2>/dev/null || true
                    systemctl stop apache2 2>/dev/null || true
                fi
                
                certbot certonly --standalone -d "$PANEL_DOMAIN" --non-interactive --agree-tos --register-unsafely-without-email
                
                if [ $? -eq 0 ]; then
                    log_success "Certificate issued successfully!"
                    CERT_FILE="/etc/letsencrypt/live/$PANEL_DOMAIN/fullchain.pem"
                    KEY_FILE="/etc/letsencrypt/live/$PANEL_DOMAIN/privkey.pem"
                    break
                else
                    log_error "Certbot failed. Please check your domain DNS or try manual mode."
                    exit 1
                fi
                ;;
            2)
                log_info "Manual SSL Selection"
                read -p "Enter absolute path to Certificate file (.crt/.pem): " CERT_FILE
                read -p "Enter absolute path to Key file (.key): " KEY_FILE
                
                if [ ! -f "$CERT_FILE" ] || [ ! -f "$KEY_FILE" ]; then
                    log_error "One or both files not found. Please verify paths."
                    continue
                fi
                break
                ;;
            *)
                echo "Invalid selection."
                ;;
        esac
    done
    
    # Copy/Link certs to /etc/rr-ui/certs/ to handle permission issues cleanly? 
    # Or just use them in place. User rr-ui needs read access.
    # Certbot certs in /etc/letsencrypt are root:root 0700 usually.
    # We should copy them to /etc/rr-ui/certs and chown rr-ui.
    
    log_info "Configuring certificates..."
    cp "$CERT_FILE" /etc/rr-ui/certs/server.crt
    cp "$KEY_FILE" /etc/rr-ui/certs/server.key
    chown rr-ui:rr-ui /etc/rr-ui/certs/server.crt
    chown rr-ui:rr-ui /etc/rr-ui/certs/server.key
    chmod 644 /etc/rr-ui/certs/server.crt
    chmod 600 /etc/rr-ui/certs/server.key
    
    FINAL_CERT="/etc/rr-ui/certs/server.crt"
    FINAL_KEY="/etc/rr-ui/certs/server.key"
}

# Configure panel settings
configure_panel() {
    log_info "Panel Configuration"
    echo -e "${YELLOW}Please enter the following configuration for your panel:${NC}"
    
    # Username
    while true; do
        read -p "Admin Username [default: admin]: " input_user
        PANEL_USER=${input_user:-"admin"}
        if [[ "$PANEL_USER" =~ ^[a-zA-Z0-9_]+$ ]]; then
            break
        else
            log_error "Invalid username. Use alphanumeric characters only."
        fi
    done

    # Password
    while true; do
        read -p "Admin Password [leave empty to generate random]: " input_pass
        if [ -z "$input_pass" ]; then
            PANEL_PASS=$(openssl rand -base64 12 | tr -d "=+/" | cut -c1-12)
            echo "Generated password: $PANEL_PASS"
            break
        elif [ ${#input_pass} -ge 6 ]; then
            PANEL_PASS="$input_pass"
            break
        else
            log_error "Password must be at least 6 characters."
        fi
    done

    # Port
    while true; do
        read -p "Panel Port [default: 2053]: " input_port
        PANEL_PORT=${input_port:-"2053"}
        if [[ "$PANEL_PORT" =~ ^[0-9]+$ ]] && [ "$PANEL_PORT" -ge 1 ] && [ "$PANEL_PORT" -le 65535 ]; then
            break
        else
            log_error "Invalid port number."
        fi
    done

    # Panel Path
    while true; do
        read -p "Panel Secret Path (e.g. /secure) [default: /panel]: " input_path
        PANEL_PATH=${input_path:-"/panel"}
        [[ "$PANEL_PATH" != /* ]] && PANEL_PATH="/$PANEL_PATH"
        
        if [[ "$PANEL_PATH" =~ ^/[a-zA-Z0-9_/-]+$ ]]; then
            break
        else
            log_error "Invalid path format."
        fi
    done
    
    # SSL Setup is MANDATORY now
    setup_ssl

    # Apply settings
    log_info "Applying settings..."
    mkdir -p /etc/rr-ui
    
    # We must ensure db exists or is writeable? 
    # rr-ui setting command will init DB.
    
    if /usr/bin/rr-ui setting \
        --username "$PANEL_USER" \
        --password "$PANEL_PASS" \
        --port "$PANEL_PORT" \
        --set-secret-path "$PANEL_PATH"; then
        
        # Apply Certs via CLI
        if /usr/bin/rr-ui cert --cert "$FINAL_CERT" --key "$FINAL_KEY"; then
             log_success "Configuration and SSL applied"
        else
             log_error "Failed to apply SSL certificate settings"
        fi
    else
        log_error "Failed to apply settings"
        exit 1
    fi
}

# Setup firewall
setup_firewall() {
    log_info "Configuring firewall..."
    
    # UFW (Ubuntu/Debian)
    if command -v ufw > /dev/null 2>&1; then
        log_info "UFW detected. Allowing port $PANEL_PORT..."
        ufw allow "$PANEL_PORT"/tcp
        ufw allow "$PANEL_PORT"/udp
        # Allow 80/443 for Certbot renewal if needed
        ufw allow 80/tcp
        ufw allow 443/tcp
    
    # Firewalld (CentOS/Fedora)
    elif command -v firewall-cmd > /dev/null 2>&1; then
        if systemctl is-active --quiet firewalld; then
            log_info "Firewalld detected. Allowing port $PANEL_PORT..."
            firewall-cmd --permanent --add-port="$PANEL_PORT"/tcp
            firewall-cmd --permanent --add-port="$PANEL_PORT"/udp
            firewall-cmd --permanent --add-service=http
            firewall-cmd --permanent --add-service=https
            firewall-cmd --reload
        fi
        
    # Iptables (Fallback)
    elif command -v iptables > /dev/null 2>&1; then
        log_info "Iptables detected. Adding rules..."
        iptables -I INPUT -p tcp --dport "$PANEL_PORT" -j ACCEPT
        iptables -I INPUT -p udp --dport "$PANEL_PORT" -j ACCEPT
    else
        log_warn "No firewall manager found. Please manually allow port $PANEL_PORT."
    fi
}

# Resolve port conflicts
resolve_port_conflicts() {
    log_info "Checking for port conflicts on $PANEL_PORT..."
    
    # Check if Nginx is running and listening on the port
    if command -v systemctl >/dev/null && systemctl is-active --quiet nginx; then
        if command -v netstat >/dev/null; then
             if netstat -tulpn | grep -q ":$PANEL_PORT.*nginx"; then
                 log_warn "Nginx is listening on port $PANEL_PORT. Stopping Nginx..."
                 systemctl stop nginx
                 systemctl disable nginx
             fi
        elif command -v ss >/dev/null; then
             if ss -tulpn | grep -q ":$PANEL_PORT.*nginx"; then
                 log_warn "Nginx is listening on port $PANEL_PORT. Stopping Nginx..."
                 systemctl stop nginx
                 systemctl disable nginx
             fi
        else
            # Fallback: Just restart Nginx to clear potential stale binds or stop it if we suspect it
            # But "Welcome to Nginx" means it IS running.
            log_warn "Nginx detected. It might be conflicting. Stopping it to ensure rr-ui can bind."
            systemctl stop nginx
        fi
    fi

    # Kill any other process on the port
    if command -v fuser >/dev/null; then
        fuser -k -n tcp "$PANEL_PORT" 2>/dev/null || true
    fi
}

# Start service
start_service() {
    local init_system=$(detect_init_system)
    
    log_info "Starting rr-ui service..."
    
    if [ "$init_system" = "systemd" ]; then
        systemctl start rr-ui
    elif [ "$init_system" = "procd" ]; then
        /etc/init.d/rr-ui start
    fi
    
    sleep 2
    log_success "Service started"
}

# Print success dashboard
print_dashboard() {
    local host=$(hostname -I | awk '{print $1}' | head -n1 || echo "YOUR_IP")
    if [ ! -z "$PANEL_DOMAIN" ]; then
        host="$PANEL_DOMAIN"
    fi
    local protocol="https"
    
    echo ""
    echo -e "${GREEN}${BOLD}"
    echo "╔════════════════════════════════════════════════════════════╗"
    echo "║                                                            ║"
    echo "║           🎉  Installation Successful!  🎉                 ║"
    echo "║                                                            ║"
    echo "╚════════════════════════════════════════════════════════════╝"
    echo -e "${NC}"
    
    echo -e "${CYAN}${BOLD}Panel Access Information:${NC}"
    echo -e "${YELLOW}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"
    echo ""
    echo -e "  ${BOLD}Access URL:${NC}      ${GREEN}${protocol}://${host}:${PANEL_PORT}${PANEL_PATH}${NC}"
    echo -e "  ${BOLD}Username:${NC}        ${CYAN}${PANEL_USER}${NC}"
    echo -e "  ${BOLD}Password:${NC}        ${MAGENTA}${PANEL_PASS}${NC}"
    echo -e "  ${BOLD}Admin Port:${NC}      ${CYAN}${PANEL_PORT}${NC}"
    echo ""
    echo -e "  ${BOLD}RustRay:${NC}         ${GREEN}Native Mode${NC}"
    echo -e "  ${BOLD}Geo Assets:${NC}      ${CYAN}/usr/share/xray${NC}"
    echo ""
    echo -e "${YELLOW}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"
    echo ""
    echo -e "${CYAN}${BOLD}Useful Commands:${NC}"
    echo -e "  ${GREEN}rr-ui${NC}             - Open TUI menu"
    echo -e "  ${GREEN}rr-ui status${NC}      - View service status"
    echo -e "  ${GREEN}rr-ui restart${NC}     - Restart the service"
    echo -e "  ${GREEN}rr-ui log${NC}         - View service logs"
    echo ""
    echo -e "${YELLOW}⚠️  Please save your credentials securely!${NC}"
    echo ""
}

# Main installation flow
main() {
    print_banner
    
    check_root
    install_dependencies
    
    local init_system=$(detect_init_system)
    log_info "Detected init system: $init_system"
    
    stop_existing_service
    create_user
    create_directories
    install_rustray
    install_binary
    
    # Configure panel
    configure_panel
    
    # Ensure correct ownership after initialization
    chown -R rr-ui:rr-ui /etc/rr-ui 2>/dev/null || chown -R rr-ui /etc/rr-ui
    chown -R rr-ui:rr-ui /usr/local/rr-ui 2>/dev/null || chown -R rr-ui /usr/local/rr-ui
    chown -R rr-ui:rr-ui /var/log/rr-ui 2>/dev/null || chown -R rr-ui /var/log/rr-ui
    
    # Setup firewall
    setup_firewall
    
    # Resolve conflicting services
    resolve_port_conflicts

    # Create service
    if [ "$init_system" = "systemd" ]; then
        create_systemd_service
    elif [ "$init_system" = "procd" ]; then
        create_openwrt_service
    else
        log_warn "Unknown init system, skipping service creation"
    fi
    
    start_service
    print_dashboard
}

# Run main function
main "$@"
