#!/bin/bash
set -e

# Colors
GREEN='\033[0;32m'
BLUE='\033[0;34m'
RED='\033[0;31m'
NC='\033[0m'

log_info() { echo -e "${BLUE}[INFO]${NC} $1"; }
log_success() { echo -e "${GREEN}[✓]${NC} $1"; }
log_error() { echo -e "${RED}[✗]${NC} $1"; }

log_info "Starting Developer Environment Setup..."

# 1. Update and Install System Dependencies
if command -v apt-get &> /dev/null; then
    log_info "Detected apt package manager. Installing system dependencies..."
    sudo apt-get update
    sudo apt-get install -y build-essential pkg-config libssl-dev clang cmake curl git
elif command -v dnf &> /dev/null; then
    log_info "Detected dnf package manager. Installing system dependencies..."
    sudo dnf install -y @development-tools perl-core openssl-devel clang cmake curl git
elif command -v pacman &> /dev/null; then
    log_info "Detected pacman package manager. Installing system dependencies..."
    sudo pacman -S --needed base-devel openssl clang cmake curl git
else
    log_error "Unsupported package manager. Please install 'build-essential', 'libssl-dev', 'pkg-config', 'clang', 'cmake' manually."
fi

# 2. Install Rust
if ! command -v cargo &> /dev/null; then
    log_info "Installing Rust..."
    curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y
    source $HOME/.cargo/env
    log_success "Rust installed."
else
    log_info "Rust is already installed."
fi

# 3. Install Node.js & pnpm
if ! command -v node &> /dev/null; then
    log_info "Installing Node.js (using nvm)..."
    curl -o- https://raw.githubusercontent.com/nvm-sh/nvm/v0.39.7/install.sh | bash
    export NVM_DIR="$HOME/.nvm"
    [ -s "$NVM_DIR/nvm.sh" ] && \. "$NVM_DIR/nvm.sh"
    nvm install --lts
    log_success "Node.js $(node -v) installed."
else
    log_info "Node.js $(node -v) is already installed."
fi

if ! command -v pnpm &> /dev/null; then
    log_info "Installing pnpm..."
    npm install -g pnpm
    log_success "pnpm installed."
else
    log_info "pnpm is already installed."
fi

# 4. Proto buffer compiler (protobuf-compiler)
if command -v apt-get &> /dev/null; then
    sudo apt-get install -y protobuf-compiler
elif command -v dnf &> /dev/null; then
    sudo dnf install -y protobuf-compiler
elif command -v pacman &> /dev/null; then
    sudo pacman -S --needed protobuf
fi

log_success "Development environment setup complete!"
echo ""
echo "You can now build the project with:"
echo "  pnpm install  (in ./web)"
echo "  pnpm build    (in ./web)"
echo "  cargo build"
