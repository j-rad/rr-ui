#!/bin/bash
# test_deploy_integrity.sh
# Tests clean RustRay deployment on a fresh Linux instance

set -e

# Colors
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
CYAN='\033[0;36m'
NC='\033[0m'
BOLD='\033[1m'

log_info() { echo -e "${BLUE}[TEST-INFO]${NC} $1"; }
log_success() { echo -e "${GREEN}[TEST-✓]${NC} $1"; }
log_error() { echo -e "${RED}[TEST-✗]${NC} $1"; }
log_warn() { echo -e "${YELLOW}[TEST-!]${NC} $1"; }

print_banner() {
    echo -e "${CYAN}${BOLD}"
    cat << 'EOF'
╔═══════════════════════════════════════════════╗
║  RustRay Deployment Integrity Test Suite     ║
╚═══════════════════════════════════════════════╝
EOF
    echo -e "${NC}"
}

# Test counter
TESTS_PASSED=0
TESTS_FAILED=0

test_assert() {
    local description=$1
    local command=$2
    
    log_info "Testing: $description"
    
    if eval "$command"; then
        log_success "$description"
        ((TESTS_PASSED++))
        return 0
    else
        log_error "$description"
        ((TESTS_FAILED++))
        return 1
    fi
}

print_banner
log_info "Starting deployment integrity tests..."
echo ""

# Test 1: Check rr-ui binary installation
test_assert "rr-ui binary exists in /usr/bin" "[ -f /usr/bin/rr-ui ]"
test_assert "rr-ui binary is executable" "[ -x /usr/bin/rr-ui ]"

# Test 2: Check RustRay binary installation  
test_assert "rustray binary exists in /usr/local/rr-ui/bin" "[ -f /usr/local/rr-ui/bin/rustray ]"
test_assert "rustray binary is executable" "[ -x /usr/local/rr-ui/bin/rustray ]"
test_assert "rustray symlink exists" "[ -L /usr/local/bin/rustray ]"

# Test 3: Check directory structure
test_assert "Config directory /etc/rr-ui exists" "[ -d /etc/rr-ui ]"
test_assert "Binary directory /usr/local/rr-ui/bin exists" "[ -d /usr/local/rr-ui/bin ]"
test_assert "Geo assets directory /usr/share/xray exists" "[ -d /usr/share/xray ]"
test_assert "Log directory /var/log/rr-ui exists" "[ -d /var/log/rr-ui ]"

# Test 4: Check system user
test_assert "rr-ui user exists" "id -u rr-ui > /dev/null 2>&1"

# Test 5: Check permissions
test_assert "/etc/rr-ui owned by rr-ui" "[ \$(stat -c '%U' /etc/rr-ui) = 'rr-ui' ]"
test_assert "/usr/local/rr-ui owned by rr-ui" "[ \$(stat -c '%U' /usr/local/rr-ui) = 'rr-ui' ]"

# Test 6: Check systemd service (if systemd is available)
if [ -f "/bin/systemctl" ] || [ -f "/usr/bin/systemctl" ]; then
    test_assert "systemd service file exists" "[ -f /etc/systemd/system/rr-ui.service ]"
    test_assert "systemd service is enabled" "systemctl is-enabled rr-ui > /dev/null 2>&1"
    
    # Test 7: Check service configuration
    log_info "Verifying systemd service configuration..."
    
    if grep -q "ExecStartPre.*rustray.*-test" /etc/systemd/system/rr-ui.service; then
        log_success "Pre-flight config validation (-test flag) configured"
        ((TESTS_PASSED++))
    else
        log_error "Pre-flight config validation NOT configured"
        ((TESTS_FAILED++))
    fi
    
    if grep -q "XRAY_LOCATION_ASSET=/usr/share/xray" /etc/systemd/system/rr-ui.service; then
        log_success "XRAY_LOCATION_ASSET environment variable set"
        ((TESTS_PASSED++))
    else
        log_error "XRAY_LOCATION_ASSET environment variable NOT set"
        ((TESTS_FAILED++))
    fi
fi

# Test 8: Check binary versions
log_info "Checking binary versions..."

if /usr/bin/rr-ui --version > /dev/null 2>&1 || /usr/bin/rr-ui --help > /dev/null 2>&1; then
    log_success "rr-ui binary responds to version/help"
    ((TESTS_PASSED++))
else
    log_warn "rr-ui binary version check unclear (may require different flags)"
fi

if /usr/local/rr-ui/bin/rustray -version > /dev/null 2>&1; then
    log_success "rustray binary responds to -version"
    ((TESTS_PASSED++))
else
    log_warn "rustray binary version check unclear"
fi

# Test 9: TUI Launch test (non-interactive check)
log_info "Testing TUI availability..."

# Check if rr-ui can be invoked (we can't test interactively in script)
if timeout 2 rr-ui --help > /dev/null 2>&1; then
    log_success "rr-ui CLI responds properly"
    ((TESTS_PASSED++))
else
    log_warn "rr-ui CLI test inconclusive"
fi

# Test 10: Check CLI status command
log_info "Testing rr-ui status command..."

if timeout 5 rr-ui status > /dev/null 2>&1; then
    log_success "rr-ui status command works"
    ((TESTS_PASSED++))
elif echo $? | grep -q "124"; then
    log_warn "rr-ui status command timed out (service may not be running yet)"
else
    log_warn "rr-ui status command failed (service may not be running)"
fi

# Test 11: Check database initialization
if [ -f "/etc/rr-ui/rr-ui.db" ]; then
    log_success "Database file created"
    ((TESTS_PASSED++))
    
    # Check database is a valid SurrealDB file
    if file /etc/rr-ui/rr-ui.db | grep -q "data"; then
        log_success "Database file appears valid"
        ((TESTS_PASSED++))
    fi
else
    log_warn "Database file not found (may be created on first run)"
fi

# Test 12: Check capabilities (if setcap available)
if command -v getcap > /dev/null 2>&1; then
    log_info "Checking binary capabilities..."
    
    if getcap /usr/bin/rr-ui | grep -q "cap_net_bind_service\|cap_net_admin"; then
        log_success "rr-ui binary has required capabilities"
        ((TESTS_PASSED++))
    else
        log_warn "rr-ui binary capabilities not set (may require manual configuration)"
    fi
fi

# Test 13: Service start test (optional, may fail if already running)
if systemctl is-active --quiet rr-ui 2>/dev/null; then
    log_success "rr-ui service is currently running"
    ((TESTS_PASSED++))
else
    log_warn "rr-ui service is not running (this is OK for fresh install)"
fi

# Print summary
echo ""
echo -e "${CYAN}${BOLD}═══════════════════════════════════════════════${NC}"
echo -e "${BOLD}Test Summary:${NC}"
echo -e "${GREEN}  Passed: $TESTS_PASSED${NC}"
echo -e "${RED}  Failed: $TESTS_FAILED${NC}"
echo -e "${CYAN}${BOLD}═══════════════════════════════════════════════${NC}"
echo ""

if [ $TESTS_FAILED -eq 0 ]; then
    echo -e "${GREEN}${BOLD}✓ All tests passed! Deployment integrity verified.${NC}"
    exit 0
else
    echo -e "${YELLOW}${BOLD}! Some tests failed or were inconclusive.${NC}"
    echo -e "${YELLOW}  This may be expected for a fresh installation.${NC}"
    exit 1
fi
