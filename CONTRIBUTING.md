# Contributing to rr-ui 🚀

Welcome to the **EdgeRay/RustRay** open-source community! We're thrilled that you're interested in helping us build a more open and resilient web.

---

## 🛠️ Setting Up Your Development Environment

To contribute effectively to `rr-ui`, follow these steps:

1.  **Fork the Repository** on GitHub.
2.  **Clone Your Fork** to your local machine:
    ```bash
    git clone https://github.com/YOUR_USERNAME/rr-ui.git
    cd rr-ui
    ```
3.  **Install Dependencies**:
    - Ensure you have the latest stable Rust installed.
    - Install the Dioxus CLI: `cargo install dioxus-cli`.
    - (Optional) Install `pnpm` for any legacy frontend builds.
4.  **Run in Development Mode**:
    ```bash
    # For fullstack development with hot reload
    dx serve
    ```

## 📜 Coding Standards & PR Process

To maintain a production-ready codebase, we follow these strict rules:

### 1. Style & Linting
Prior to every commit, ensure your code passes:
- **Formatting**: `cargo fmt --all`
- **Linting**: `cargo clippy --all-targets --all-features -- -D warnings`
- **Testing**: `./run_tests.sh ci`

### 2. Creating a Pull Request
- Create a feature branch: `git checkout -b feature/awesome-new-visualization`.
- Commit with descriptive, conventional commit messages.
- Ensure all tests pass.
- Submit your PR with a clear description of the problem solved or feature added.

## 🧪 Testing

We value high test coverage, especially for security-critical components like gRPC orchestration and telemetry management.
- Write unit tests for new logic in `src/`.
- Add integration tests in `tests/` for full-system behavior.
- Use `cargo test --features server` to run the full server-side suite.

## 💎 Design Guidelines

- Follow the **Obsidian** design system tokens.
- Maintain **Glassmorphism** visual consistency.
- Ensure all UI elements are fully responsive for mobile/tablet EdgeRay users.

---

## 💡 Where to Start?

Check out our [Issue Tracker](https://github.com/j-rad/rr-ui/issues) for tags like `good-first-issue` or `help-wanted`. If you have a major architectural proposal, please [start a discussion](https://github.com/j-rad/rr-ui/discussions) before implementing.

Happy Coding! 🦀
