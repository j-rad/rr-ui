# rr-ui

A dual-mode (Server/Client) Xray panel solution built with Rust, Actix-Web, and Dioxus. Now with dual-core support for both Xray and RustRay.


*Installs the panel, creates the service, and enables it on port 2053.*

## Project Overview

This project provides a secure, performant, and maintainable UI for managing Xray-compatible cores. It supports two modes of operation:

1.  **Server Mode**: Full-featured panel for managing users, inbounds, routing, and certificates on a server. Uses SurrealDB for persistence.
2.  **Client Mode**: Lightweight configuration interface for OpenWrt or embedded devices to manage outbound connections. Uses file-based storage to minimize dependencies.

### New Features

*   **Dual-Core Support**: Seamlessly switch between **Xray-core** and **RustRay** from the settings page.
*   **Modern UI/UX**: A complete visual overhaul featuring a responsive design, dark mode, and glassmorphism effects.
*   **Enhanced Performance & Reliability**: Optimized background jobs and robust connection handling with exponential backoff.
*   **Comprehensive API**: Expanded API for managing routing, backups, Telegram bots, and more.

## Tech Stack

*   **Backend**: Rust (Actix-Web)
*   **Database**: SurrealDB (Embedded RocksDB) for Server Mode; File System for Client Mode.
*   **Frontend**: SvelteKit + TailwindCSS
*   **Communication**: gRPC (for controlling the Xray/RustRay Core)

## Setup & Run Instructions

### Prerequisites

*   Rust (latest stable)
*   Node.js & npm (for frontend)
*   Protobuf Compiler (`protoc`)
*   Xray Core or RustRay binary installed/available in path.

### Backend

To build the project, use one of the following commands:

**Server Mode (Default):**

```bash
cargo build --release --features server
```

**Client Mode (Lightweight):**

```bash
cargo build --release
```

**Running:**

```bash
./target/release/rr-ui run --port 54321
```

### Frontend

The frontend is a SvelteKit app located in `web/`.

```bash
cd web
pnpm install
pnpm run build
```

The build artifacts (`web/build`) are served by the backend.

## API Endpoint Summary

All API routes are prefixed with `/panel/api`. Protected routes require a JWT token in the `Authorization: Bearer <token>` header.

### Public Routes

*   `POST /login`: Authenticate and receive a JWT.
*   `GET /sub`: Get subscription content for clients.

### Common Routes (Server & Client)

*   **System & Core**:
    *   `GET /server/status`: Get system resource usage (CPU, RAM, Disk).
    *   `GET /server/mode`: Get current application mode ("server" or "client").
    *   `GET /server/xray/status`: Check Xray/RustRay core status.
    *   `POST /server/xray/restart`: Restart the core.
    *   `POST /server/xray/stop`: Stop the core.
*   **Settings**:
    *   `GET /setting`: Get all settings.
    *   `POST /setting`: Update a setting.
*   **Client Traffic**:
    *   `GET /client/traffic`: Get traffic stats for the current user.
    *   `POST /client/reset_traffic`: Reset traffic for the current user.

### Server-Only Routes

*   **Inbounds**:
    *   `GET /inbounds`: List all inbounds.
    *   `POST /inbounds`: Add a new inbound.
    *   `PUT /inbounds`: Update an inbound.
    *   `DELETE /inbounds`: Delete an inbound.
*   **Routing**:
    *   `GET /routing`: List all routing rules.
    *   `POST /routing`: Add a new rule.
    *   `DELETE /routing`: Delete a rule.
*   **Backups**:
    *   `GET /backup/export`: Export the database.
    *   `POST /backup/import`: Import a database backup.
*   **Certificates**:
    *   `POST /cert/issue`: Issue a new SSL certificate.
*   **Telegram Bot**:
    *   `GET /tgbot`: Get Telegram bot configuration.
    *   `POST /tgbot`: Update Telegram bot configuration.
    *   `POST /tgbot/test`: Test the bot connection.
*   **WARP**:
    *   `GET /warp/status`: Get WARP status.
    *   `POST /warp/register`: Register a new WARP account.


### test local
*   **Test**:
    *   cargo run --profile release --bin rr-ui -- run --port 2053