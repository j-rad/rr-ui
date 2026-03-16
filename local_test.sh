#!/bin/bash
set -e

# Define binary path
BINARY="./target/release/rr-ui"

if [ ! -f "$BINARY" ]; then
    echo "Binary not found at $BINARY. Please build first."
    exit 1
fi

echo "Initializing local database..."
# Set default credentials for local test
$BINARY setting --username admin --password admin --port 2053 --set-secret-path /panel

echo "Starting rr-ui server locally..."
echo "Access at: http://localhost:2053/panel"
echo "Press Ctrl+C to stop"

# Run the server
$BINARY run
