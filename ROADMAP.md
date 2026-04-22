# rr-ui v1.0 Roadmap 🧗

This document outlines the milestones and vision for reaching **v1.0 (Production Ready)** and beyond. Our primary goal is to provide a rock-solid, user-friendly interface for the **EdgeRay/RustRay** censorship circumvention core.

---

## 🎯 The Path to v1.0 (Phase 8 Focus)

To achieve a production-ready status, we are focusing on the following core areas:

### 1. 🧪 Stability & Testing
- [ ] **100% Test Coverage** for the `rustray_process` manager and `uds_manager`.
- [ ] **Fuzz Testing**: Stress test the gRPC/UDS bridge for malformed packet handling.
- [ ] **End-to-End CI**: Automated browser testing (via Playwright or Dioxus-Test) for critical flows like user creation and link generation.

### 2. 🛡️ Security Hardening
- [ ] **OAuth2/OpenID Integration**: Support external identity providers for enterprise server deployments.
- [ ] **RBAC (Role-Based Access Control)**: Granular permissions for admins vs. read-only telemetry viewers.
- [ ] **Encrypted DB-at-rest**: Secure SurrealDB storage with user-provided master keys.

### 3. 📱 Experience & Accessibility
- [ ] **Mobile Native Clients**: Export `rr-ui` components to Capacitor/Tauri for iOS and Android.
- [ ] **Advanced Visualization**: Use WebGL to render massive P2P network maps and real-time flow-j throughput.
- [ ] **Internationalization (i18n)**: Full support for Persian, Chinese, Russian, and Arabic languages.

### 4. 🛰️ Decentralized Signaling
- [ ] **P2P Signaling**: Implement decentralized handshakes using the newly finalized BLAKE3 authentication.
- [ ] **Off-grid Orchestration**: Allow the panel to manage hidden nodes without requiring a centralized API.

---

## 🚀 Post-v1.0 Vision

- **Autonomous Fallback Orchestrator (Phase 9)**: AI-driven transport switching based on real-world DPI patterns.
- **Quantum-Resistant Telemetry**: Fully post-quantum encrypted metrics for the most sensitive environments.

---

## 💡 How You Can Help

We welcome contributions towards any of these milestones! Check out [CONTRIBUTING.md](./CONTRIBUTING.md) to get started.
