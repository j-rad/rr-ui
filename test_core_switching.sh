#!/bin/bash
# RustRay Integration Test Script
# This script helps verify the Xray/RustRay core switching functionality

set -e

echo "======================================"
echo "RR-UI Core Switching Test Suite"
echo "======================================"
echo ""

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

# Test counter
TESTS_PASSED=0
TESTS_FAILED=0

# Helper functions
pass() {
    echo -e "${GREEN}✓ PASS${NC}: $1"
    ((TESTS_PASSED++))
}

fail() {
    echo -e "${RED}✗ FAIL${NC}: $1"
    ((TESTS_FAILED++))
}

info() {
    echo -e "${YELLOW}ℹ INFO${NC}: $1"
}

# Check if rr-ui is running
check_server() {
    if curl -s http://localhost:54321/panel/setting/all > /dev/null 2>&1; then
        return 0
    else
        return 1
    fi
}

# Test 1: Server Availability
echo "Test 1: Checking if rr-ui server is running..."
if check_server; then
    pass "Server is running on port 54321"
else
    fail "Server is not running. Please start rr-ui first."
    exit 1
fi
echo ""

# Test 2: Fetch Current Settings
echo "Test 2: Fetching current settings..."
SETTINGS=$(curl -s http://localhost:54321/panel/setting/all)
if echo "$SETTINGS" | jq -e '.success == true' > /dev/null 2>&1; then
    CORE_TYPE=$(echo "$SETTINGS" | jq -r '.obj.core_type // "xray"')
    CORE_PATH=$(echo "$SETTINGS" | jq -r '.obj.core_path // "default"')
    pass "Settings retrieved successfully"
    info "Current core_type: $CORE_TYPE"
    info "Current core_path: $CORE_PATH"
else
    fail "Failed to fetch settings"
fi
echo ""

# Test 3: Check if Xray binary exists
echo "Test 3: Checking for Xray binary..."
if command -v xray &> /dev/null; then
    XRAY_PATH=$(which xray)
    pass "Xray found at: $XRAY_PATH"
else
    fail "Xray binary not found in PATH"
fi
echo ""

# Test 4: Check if RustRay binary exists
echo "Test 4: Checking for RustRay binary..."
if command -v rustray &> /dev/null; then
    RUSTRAY_PATH=$(which rustray)
    pass "RustRay found at: $RUSTRAY_PATH"
else
    info "RustRay binary not found in PATH (this is OK if you haven't installed it yet)"
fi
echo ""

# Test 5: Update settings to use Xray (baseline)
echo "Test 5: Setting core to Xray..."
RESPONSE=$(curl -s -X POST http://localhost:54321/panel/setting/update \
    -H "Content-Type: application/json" \
    -d '{
        "web_port": 54321,
        "web_cert_file": null,
        "web_key_file": null,
        "username": "admin",
        "password_hash": "",
        "core_type": "xray",
        "core_path": null
    }')

if echo "$RESPONSE" | jq -e '.success == true' > /dev/null 2>&1; then
    pass "Successfully switched to Xray"
    info "Waiting 3 seconds for core to restart..."
    sleep 3
else
    fail "Failed to switch to Xray"
    echo "Response: $RESPONSE"
fi
echo ""

# Test 6: Verify Xray is running
echo "Test 6: Verifying Xray process..."
if pgrep -x "xray" > /dev/null; then
    pass "Xray process is running"
else
    fail "Xray process not found"
fi
echo ""

# Test 7: Test invalid binary path
echo "Test 7: Testing error handling with invalid path..."
RESPONSE=$(curl -s -X POST http://localhost:54321/panel/setting/update \
    -H "Content-Type: application/json" \
    -d '{
        "web_port": 54321,
        "web_cert_file": null,
        "web_key_file": null,
        "username": "admin",
        "password_hash": "",
        "core_type": "xray",
        "core_path": "/invalid/path/to/xray"
    }')

if echo "$RESPONSE" | jq -e '.success == false' > /dev/null 2>&1; then
    pass "Correctly rejected invalid binary path"
    ERROR_MSG=$(echo "$RESPONSE" | jq -r '.msg')
    info "Error message: $ERROR_MSG"
else
    fail "Should have rejected invalid path"
fi
echo ""

# Test 8: Restore to working Xray
echo "Test 8: Restoring to working Xray configuration..."
RESPONSE=$(curl -s -X POST http://localhost:54321/panel/setting/update \
    -H "Content-Type: application/json" \
    -d '{
        "web_port": 54321,
        "web_cert_file": null,
        "web_key_file": null,
        "username": "admin",
        "password_hash": "",
        "core_type": "xray",
        "core_path": null
    }')

if echo "$RESPONSE" | jq -e '.success == true' > /dev/null 2>&1; then
    pass "Restored to Xray"
    sleep 3
else
    fail "Failed to restore Xray"
fi
echo ""

# Test 9: Switch to RustRay (if available)
if command -v rustray &> /dev/null; then
    echo "Test 9: Switching to RustRay..."
    RESPONSE=$(curl -s -X POST http://localhost:54321/panel/setting/update \
        -H "Content-Type: application/json" \
        -d '{
            "web_port": 54321,
            "web_cert_file": null,
            "web_key_file": null,
            "username": "admin",
            "password_hash": "",
            "core_type": "rustray",
            "core_path": null
        }')

    if echo "$RESPONSE" | jq -e '.success == true' > /dev/null 2>&1; then
        pass "Successfully switched to RustRay"
        info "Waiting 3 seconds for core to restart..."
        sleep 3
        
        # Verify RustRay is running
        if pgrep -x "rustray" > /dev/null; then
            pass "RustRay process is running"
        else
            fail "RustRay process not found"
        fi
    else
        fail "Failed to switch to RustRay"
        echo "Response: $RESPONSE"
    fi
    echo ""
    
    # Switch back to Xray
    echo "Switching back to Xray..."
    curl -s -X POST http://localhost:54321/panel/setting/update \
        -H "Content-Type: application/json" \
        -d '{
            "web_port": 54321,
            "web_cert_file": null,
            "web_key_file": null,
            "username": "admin",
            "password_hash": "",
            "core_type": "xray",
            "core_path": null
        }' > /dev/null
    sleep 2
else
    info "Skipping RustRay test (binary not found)"
fi

# Summary
echo ""
echo "======================================"
echo "Test Summary"
echo "======================================"
echo -e "${GREEN}Passed: $TESTS_PASSED${NC}"
echo -e "${RED}Failed: $TESTS_FAILED${NC}"
echo ""

if [ $TESTS_FAILED -eq 0 ]; then
    echo -e "${GREEN}All tests passed!${NC}"
    exit 0
else
    echo -e "${RED}Some tests failed. Please review the output above.${NC}"
    exit 1
fi
