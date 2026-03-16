#!/bin/bash

#
# rr-ui Management Script
#
# Description: A comprehensive shell script to manage the rr-ui service,
# settings, security, and updates. Ported and adapted from the original
# tx-ui script.
#

# --- Configuration ---
# Colors for better user feedback
RED="\033[0;31m"
GREEN="\033[0;32m"
YELLOW="\033[0;33m"
NC="\033[0m" # No Color

# Paths
BINARY_PATH="/usr/local/rr-ui/rr-ui"
DATA_DIR="/usr/local/rr-ui/bin"
GEOIP_PATH="${DATA_DIR}/geoip.dat"
GEOSITE_PATH="${DATA_DIR}/geosite.dat"
SERVICE_NAME="rr-ui"

# URLs for geo data
GEOIP_URL="https://github.com/v2fly/geoip/releases/latest/download/geoip.dat"
GEOSITE_URL="https://github.com/v2fly/domain-list-community/releases/latest/download/dlc.dat" # Note: dlc.dat is the correct file for geosite

# --- Utility Functions ---
check_root() {
    if [[ $EUID -ne 0 ]]; then
        echo -e "${RED}Error: This script must be run as root.${NC}"
        exit 1
    fi
}

pause() {
    read -p "Press Enter to continue..."
}

# --- Service Management ---
service_menu() {
    clear
    echo "----------------------------------------"
    echo " rr-ui Service Management"
    echo "----------------------------------------"
    systemctl is-active --quiet $SERVICE_NAME && echo -e "Service Status: ${GREEN}Running${NC}" || echo -e "Service Status: ${RED}Stopped${NC}"
    echo "----------------------------------------"
    echo -e "${YELLOW}1.${NC} Start Service"
    echo -e "${YELLOW}2.${NC} Stop Service"
    echo -e "${YELLOW}3.${NC} Restart Service"
    echo -e "${YELLOW}4.${NC} Show Status"
    echo -e "${YELLOW}5.${NC} View Logs"
    echo "----------------------------------------"
    read -p "Enter selection [1-5]: " choice

    case $choice in
        1) systemctl start $SERVICE_NAME && echo -e "${GREEN}Service started.${NC}" || echo -e "${RED}Failed to start service.${NC}";;
        2) systemctl stop $SERVICE_NAME && echo -e "${GREEN}Service stopped.${NC}" || echo -e "${RED}Failed to stop service.${NC}";;
        3) systemctl restart $SERVICE_NAME && echo -e "${GREEN}Service restarted.${NC}" || echo -e "${RED}Failed to restart service.${NC}";;
        4) systemctl status $SERVICE_NAME;;
        5) journalctl -u $SERVICE_NAME -f --no-pager;;
        *) echo -e "${RED}Invalid choice.${NC}";;
    esac
    pause
}

# --- Admin Management ---
admin_menu() {
    clear
    echo "----------------------------------------"
    echo " Admin Management"
    echo "----------------------------------------"
    echo -e "${YELLOW}1.${NC} Change Username & Password"
    echo -e "${YELLOW}2.${NC} Change Port"
    echo -e "${YELLOW}3.${NC} Reset Web Base Path"
    echo -e "${YELLOW}4.${NC} Reset All Settings"
    echo "----------------------------------------"
    read -p "Enter selection [1-4]: " choice

    case $choice in
        1)
            read -p "Enter new username: " username
            read -s -p "Enter new password: " password
            echo
            $BINARY_PATH setting --username "$username" --password "$password"
            ;;
        2)
            read -p "Enter new port (e.g., 8080): " port
            $BINARY_PATH setting --port "$port"
            ;;
        3)
            $BINARY_PATH setting --web-base-path "/"
            echo -e "${GREEN}Web base path reset to '/'${NC}"
            ;;
        4)
            read -p "${YELLOW}Are you sure you want to reset ALL settings to default? (y/n): ${NC}" confirm
            if [[ "$confirm" == "y" ]]; then
                $BINARY_PATH setting --reset
            fi
            ;;
        *) echo -e "${RED}Invalid choice.${NC}";;
    esac
    echo -e "\n${GREEN}Restart the service for changes to take effect.${NC}"
    pause
}

# --- Network & Security ---
network_menu() {
    clear
    echo "----------------------------------------"
    echo " Network & Security"
    echo "----------------------------------------"
    echo -e "${YELLOW}1.${NC} Manage Firewall (ufw)"
    echo -e "${YELLOW}2.${NC} Manage IP Limit (fail2ban)"
    echo -e "${YELLOW}3.${NC} Enable TCP BBR"
    echo -e "${YELLOW}4.${NC} Issue SSL Certificate (acme.sh)"
    echo "----------------------------------------"
    read -p "Enter selection [1-4]: " choice

    case $choice in
        1) firewall_menu ;;
        2) iplimit_main ;;
        3) bbr_menu ;;
        4) ssl_cert_issue ;;
        *) echo -e "${RED}Invalid choice.${NC}";;
    esac
}

firewall_menu() {
    if ! command -v ufw &> /dev/null; then
        echo -e "${YELLOW}ufw is not installed. Installing...${NC}"
        apt-get update && apt-get install -y ufw
    fi
    clear
    echo "Firewall Management (ufw)"
    ufw status | head -n 1
    echo "----------------------------------------"
    echo -e "${YELLOW}1.${NC} Allow Port"
    echo -e "${YELLOW}2.${NC} Deny Port"
    echo -e "${YELLOW}3.${NC} Delete Rule"
    echo -e "${YELLOW}4.${NC} Enable Firewall"
    echo -e "${YELLOW}5.${NC} Disable Firewall"
    echo "----------------------------------------"
    read -p "Enter selection: " choice
    case $choice in
        1) read -p "Enter port to allow: " port; ufw allow "$port";;
        2) read -p "Enter port to deny: " port; ufw deny "$port";;
        3) ufw status numbered; read -p "Enter rule number to delete: " num; ufw delete "$num";;
        4) ufw enable;;
        5) ufw disable;;
        *) echo -e "${RED}Invalid choice.${NC}";;
    esac
    pause
}

iplimit_main() {
    if ! command -v fail2ban-client &> /dev/null; then
        echo -e "${YELLOW}fail2ban is not installed. Installing...${NC}"
        apt-get update && apt-get install -y fail2ban
    fi
    echo -e "${GREEN}fail2ban is installed and running.${NC}"
    echo "It provides default protection for SSH."
    echo "Custom rr-ui integration requires specific log parsing rules."
    pause
}

bbr_menu() {
    clear
    echo "TCP BBR Management"
    echo "----------------------------------------"
    if grep -q "bbr" /proc/sys/net/ipv4/tcp_congestion_control; then
        echo -e "BBR Status: ${GREEN}Enabled${NC}"
    else
        echo -e "BBR Status: ${RED}Disabled${NC}"
    fi
    echo "----------------------------------------"
    echo -e "${YELLOW}1.${NC} Enable BBR"
    echo -e "${YELLOW}2.${NC} Disable BBR (revert to cubic)"
    echo "----------------------------------------"
    read -p "Enter selection: " choice
    case $choice in
        1)
            echo "net.core.default_qdisc=fq" >> /etc/sysctl.conf
            echo "net.ipv4.tcp_congestion_control=bbr" >> /etc/sysctl.conf
            sysctl -p
            echo -e "${GREEN}BBR enabled. A reboot is recommended.${NC}"
            ;;
        2)
            sed -i '/net.core.default_qdisc=fq/d' /etc/sysctl.conf
            sed -i '/net.ipv4.tcp_congestion_control=bbr/d' /etc/sysctl.conf
            sysctl -p
            echo -e "${GREEN}BBR disabled.${NC}"
            ;;
        *) echo -e "${RED}Invalid choice.${NC}";;
    esac
    pause
}

ssl_cert_issue() {
    if ! command -v acme.sh &> /dev/null; then
        echo -e "${YELLOW}acme.sh is not installed. Installing...${NC}"
        curl https://get.acme.sh | sh
        source ~/.bashrc
    fi
    read -p "Enter your domain name: " domain
    read -p "Enter your email for notifications: " email
    
    echo "Issuing certificate using standalone mode (requires port 80 to be free)..."
    ~/.acme.sh/acme.sh --issue --standalone -d "$domain" --server letsencrypt --register-account -m "$email"

    CERT_PATH="/root/.acme.sh/${domain}_ecc/fullchain.cer"
    KEY_PATH="/root/.acme.sh/${domain}_ecc/${domain}.key"

    if [[ -f "$CERT_PATH" && -f "$KEY_PATH" ]]; then
        echo -e "${GREEN}Certificate issued successfully.${NC}"
        $BINARY_PATH cert --cert "$CERT_PATH" --key "$KEY_PATH"
        echo -e "${GREEN}Updated rr-ui with new certificate paths. Please restart the service.${NC}"
    else
        echo -e "${RED}Failed to issue certificate.${NC}"
    fi
    pause
}

# --- Updates ---
update_menu() {
    clear
    echo "----------------------------------------"
    echo " Update Geo Data"
    echo "----------------------------------------"
    echo -e "${YELLOW}1.${NC} Update GeoIP Database (geoip.dat)"
    echo -e "${YELLOW}2.${NC} Update GeoSite Database (geosite.dat)"
    echo "----------------------------------------"
    read -p "Enter selection [1-2]: " choice

    mkdir -p $DATA_DIR
    case $choice in
        1)
            echo "Downloading latest GeoIP database..."
            if wget -q -O "$GEOIP_PATH" "$GEOIP_URL"; then
                echo -e "${GREEN}GeoIP database updated successfully.${NC}"
            else
                echo -e "${RED}Failed to download GeoIP database.${NC}"
            fi
            ;;
        2)
            echo "Downloading latest GeoSite database..."
            if wget -q -O "$GEOSITE_PATH" "$GEOSITE_URL"; then
                echo -e "${GREEN}GeoSite database updated successfully.${NC}"
            else
                echo -e "${RED}Failed to download GeoSite database.${NC}"
            fi
            ;;
        *) echo -e "${RED}Invalid choice.${NC}";;
    esac
    pause
}

# --- Nginx Management ---
nginx_menu() {
    clear
    echo "----------------------------------------"
    echo " Nginx & Reverse Proxy Management"
    echo "----------------------------------------"
    echo -e "${YELLOW}1.${NC} Install Nginx"
    echo -e "${YELLOW}2.${NC} Configure Reverse Proxy"
    echo -e "${YELLOW}3.${NC} Setup SSL (Certbot)"
    echo -e "${YELLOW}4.${NC} Start Nginx"
    echo -e "${YELLOW}5.${NC} Stop Nginx"
    echo -e "${YELLOW}6.${NC} Restart Nginx"
    echo -e "${YELLOW}7.${NC} Return"
    echo "----------------------------------------"
    read -p "Enter selection [1-7]: " choice

    case $choice in
        1) install_nginx ;;
        2) configure_nginx ;;
        3) setup_certbot ;;
        4) systemctl start nginx && echo -e "${GREEN}Nginx started.${NC}" ;;
        5) systemctl stop nginx && echo -e "${GREEN}Nginx stopped.${NC}" ;;
        6) systemctl restart nginx && echo -e "${GREEN}Nginx restarted.${NC}" ;;
        7) return ;;
        *) echo -e "${RED}Invalid choice.${NC}" ;;
    esac
    pause
}

install_nginx() {
    if [ -f /etc/os-release ]; then
        . /etc/os-release
        OS=$NAME
    fi
    
    if [[ "$OS" == *"Debian"* ]] || [[ "$OS" == *"Ubuntu"* ]]; then
        echo "Detected Debian/Ubuntu..."
        apt-get update
        apt-get install -y nginx socat
    elif [[ "$OS" == *"CentOS"* ]] || [[ "$OS" == *"AlmaLinux"* ]]; then
        echo "Detected CentOS/AlmaLinux..."
        yum install -y epel-release
        yum install -y nginx socat
    else
        echo -e "${RED}Unsupported OS for auto-install. Please install Nginx manually.${NC}"
        return
    fi
    systemctl enable nginx
    echo -e "${GREEN}Nginx installed successfully.${NC}"
}

configure_nginx() {
    read -p "Enter your Domain Name (e.g., panel.example.com): " domain
    read -p "Enter Panel Port (default 2053): " port
    port=${port:-2053}

    cat > /etc/nginx/conf.d/rr-ui.conf <<EOF
server {
    listen 80;
    server_name $domain;

    location / {
        proxy_pass http://127.0.0.1:$port;
        proxy_http_version 1.1;
        proxy_set_header Upgrade \$http_upgrade;
        proxy_set_header Connection "upgrade";
        proxy_set_header Host \$http_host;
        proxy_set_header X-Real-IP \$remote_addr;
        proxy_set_header X-Forwarded-For \$proxy_add_x_forwarded_for;
    }
}
EOF
    echo -e "${GREEN}Configuration created at /etc/nginx/conf.d/rr-ui.conf${NC}"
    echo "Testing Nginx config..."
    nginx -t && systemctl reload nginx
    echo -e "${GREEN}Nginx reloaded.${NC}"
}

setup_certbot() {
    read -p "Enter your Domain Name: " domain
    read -p "Enter Email for renewal notifications: " email

    if [ -f /etc/os-release ]; then
        . /etc/os-release
        OS=$NAME
    fi

    if [[ "$OS" == *"Debian"* ]] || [[ "$OS" == *"Ubuntu"* ]]; then
        apt-get install -y certbot python3-certbot-nginx
    elif [[ "$OS" == *"CentOS"* ]] || [[ "$OS" == *"AlmaLinux"* ]]; then
        yum install -y certbot python3-certbot-nginx
    fi

    certbot --nginx --non-interactive --agree-tos -m "$email" -d "$domain"
}

# --- Main Menu ---

main_menu() {
    check_root
    while true; do
        clear
        echo "========================================"
        echo " rr-ui Management Script"
        echo "========================================"
        echo -e "${GREEN}1.${NC} Service Management"
        echo -e "${GREEN}2.${NC} Admin Management"
        echo -e "${GREEN}3.${NC} Network & Security"
        echo -e "${GREEN}4.${NC} Update Geo Data"
        echo -e "${GREEN}5.${NC} Nginx Management"
        echo "----------------------------------------"
        echo -e "${YELLOW}0.${NC} Exit"
        echo "----------------------------------------"
        read -p "Enter selection [0-5]: " choice

        case $choice in
            1) service_menu ;;
            2) admin_menu ;;
            3) network_menu ;;
            4) update_menu ;;
            5) nginx_menu ;;
            0) exit 0 ;;
            *) echo -e "${RED}Invalid choice.${NC}" && sleep 1 ;;
        esac
    done
}

# Entry point
main_menu
