# rr-ui 🛡️

[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)
[![Rust](https://img.shields.io/badge/rust-2024-orange.svg)](https://www.rust-lang.org)
[![Dioxus](https://img.shields.io/badge/UI-Dioxus-blue.svg)](https://dioxuslabs.com)

A **High-Performance, Dual-Mode Configuration & Telemetry Panel** built entirely in Rust. `rr-ui` is the visual orchestrator for the **RustRay** censorship circumvention core, designed for both enterprise-grade servers and resource-constrained OpenWrt routers.

---

## 🌟 Project Overview

`rr-ui` serves as the visual command center for the EdgeRay/RustRay ecosystem. It operates in two specialized modes:

1.  **Server Mode**: A full-featured administrative panel for user management, P2P link generation, and geo-routing orchestration. Backed by **SurrealDB**.
2.  **Client Mode**: A lightweight, minimal-footprint interface tailored for OpenWrt routers, IoT hubs, and embedded devices.

### 🚀 Core Features

*   **P2P Orchestration**: Visually manage BLAKE3 authenticated peer-to-peer links and decentralized rings.
*   **Dynamic Routing**: Map primary transports (e.g., Brutal-QUIC) with autonomous fallback logic via gRPC.
*   **Live Telemetry**: Sub-millisecond connection health and throughput metrics via a self-healing UDS manager.
*   **Obsidian Design System**: A premium, GPU-accelerated interface featuring glassmorphism and real-world telemetry overlays.
*   **SIP003 Supervison**: Integrated support for legacy SIP003 UDP-over-TCP wrappers.

## 🛠️ Tech Stack

*   **UI Framework**: [Dioxus](https://dioxuslabs.com) (Fullstack Rust)
*   **Web Server**: [Actix-Web](https://actix.rs)
*   **Database**: SurrealDB (Server mode) / Local Flat-file (Client mode)
*   **IPC**: gRPC over Unix Domain Sockets (UDS)
*   **Style**: Vanilla CSS with the Obsidian design system

## ⚙️ Setup & Development

### Prerequisites
*   [Rust](https://rustup.rs/) (Stable/Nightly)
*   [Dioxus CLI](https://dioxuslabs.com/learn/0.6/getting_started) (`cargo install dioxus-cli`)

### Running the Project

**1. Server Mode (Full Features):**
```bash
cargo run --features server -- run
```

**2. Development with Hot Reload:**
```bash
dx serve
```

### 🧪 Testing
We maintain high standards for production readiness. Always run the test suite before contributing:
```bash
./run_tests.sh ci
```

## 🗺️ Roadmap & Production Readiness

We are currently in **Phase 7** of implementation. Our goal for **v1.0 (Production Ready)** includes:
- [ ] 100% Code Coverage for core routing logic.
- [ ] Automated security audits of the gRPC/UDS bridge.
- [ ] Full support for mobile-responsive EdgeRay clients.
- [ ] Decentralized signaling for PQC-resistant handshakes.

See [ROADMAP.md](./ROADMAP.md) for the full vision.

## 🤝 Contributing

We ❤️ open-source contributors! Whether you're fixing a bug, improving documentation, or adding a new transport visualization, we welcome your PRs.

1.  Check out our [CONTRIBUTING.md](./CONTRIBUTING.md) for setup guides.
2.  Read our [CODE_OF_CONDUCT.md](./CODE_OF_CONDUCT.md).
3.  Ensure your code passes `./run_tests.sh clippy`.

## 🔒 Security

For security vulnerabilities, please refer to [SECURITY.md](./SECURITY.md). **Do not open public issues for security exploits.**

## 📄 License

This project is licensed under the **MIT License**. See the [LICENSE](./LICENSE) file for details.