# Security Policy 🛡️

At **EdgeRay/RustRay**, we take the security of our users and our software extremely seriously. Given that our tools are designed for censorship circumvention, the safety of our contributors and end-users is paramount.

---

## 🔒 Reporting a Vulnerability

If you discover a security vulnerability in `rr-ui`, please disclose it responsibly. **Do not create a public issue on GitHub.**

To report a vulnerability, please reach out via one of the following channels:
- **Email**: security@edgeray.io (or contact @j-rad directly)
- **Signal**: @jrad.01 (preferred for sensitive reports)

We will respond promptly and work with you to remediate the vulnerability before public disclosure.

## 🕒 Release Lifecycle

Security updates are prioritized above all other development tasks. When a vulnerability is patched:
1.  A new release will be tagged (e.g., `v2.0.1`).
2.  A security advisory will be published on GitHub with details of the fix.
3.  Binary distributions (Docker, OpenWrt packages) will be updated within 24 hours.

## ✅ Security Best Practices for Contributors

To ensure the codebase remains secure, all contributions must adhere to the following guidelines:
1.  **Memory Safety**: Avoid `unsafe` blocks unless absolutely necessary and documented.
2.  **Cryptography**: Use audited libraries like `ring` or `dalek-cryptography`. Never implement custom crypto logic.
3.  **IPC Security**: All gRPC/UDS sockets must be correctly permissioned.
4.  **No Logic Flaws**: Review all routing/orchestration PRs with a focus on potential DPI identification vectors.

---

Thank you for helping us keep `rr-ui` secure!
