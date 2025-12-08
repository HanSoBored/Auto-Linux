# Auto-Linux (Rust Edition)

![Build Status](https://img.shields.io/github/actions/workflow/status/HanSoBored/Auto-Linux/build.yml?branch=main)
![Language](https://img.shields.io/badge/language-Rust-orange)
![Platform](https://img.shields.io/badge/platform-Android%20(Root)-green)
![License](https://img.shields.io/badge/license-MIT-blue)

**Auto-Linux** is a standalone, lightweight, and blazing fast Linux installer/manager for Android, written entirely in **Rust**. It provides a beautiful Terminal User Interface (TUI) to install, configure, and manage Ubuntu chroots without requiring Termux, Busybox, or external dependencies.

> **Built for speed, stability, and ease of use.**

---

## Key Features

*   **Native & Standalone:** Compiled as a static binary (`musl`). Zero dependencies. No Termux needed.
*   **Beautiful TUI:** Powered by `ratatui`. Keyboard-driven dashboard.
*   **Instant Launch:** Switch users and enter Chroot directly from the dashboard.
*   **Auto-Configuration:**
    *   **Network:** Auto-detects DNS and fixes connection issues inside chroot.
    *   **Users:** Auto-creates User & Password during setup.
    *   **Sudo:** Auto-configures `sudo` (wheel group) privileges.
    *   **Mounts:** Handles `/dev`, `/proc`, `/sys`, `/sdcard` binding automatically.
*   **Distribution Support:** Ubuntu 20.04 LTS up to 26.04.
*   **Root Detection:** Supports Magisk, KernelSU, and APatch natively.

---

## Screenshots

Preview:
![preview 1](https://raw.githubusercontent.com/HanSoBored/Auto-Linux/main/preview/preview1.jpg)

![preview 2](https://raw.githubusercontent.com/HanSoBored/Auto-Linux/main/preview/preview2.jpg)

![preview 3](https://raw.githubusercontent.com/HanSoBored/Auto-Linux/main/preview/preview3.jpg)


---

## Installation

### Option 1: One-Line Install.
Run this command in **Termux**, **ADB Shell**, or any Terminal Emulator:

```bash
curl -sL https://raw.githubusercontent.com/HanSoBored/Auto-Linux/main/install.sh | sh
```

> **Note:** This script automatically detects if you have Termux installed and creates a shortcut. You can then simply type `autolinux` to start.

### Option 2: Manual Install
1.  Download the latest binary from [Releases](https://github.com/HanSoBored/Auto-Linux/releases).
2.  Push to device: `adb push autolinux-aarch64 /data/local/tmp/autolinux`
3.  Permission: `chmod +x /data/local/tmp/autolinux`
4.  Run: `/data/local/tmp/autolinux`

---

## Build from Source

You need **Rust** and **Cross** (for cross-compiling to Android/ARM64 Musl).

1.  **Install Prerequisites**:
    ```bash
    cargo install cross
    ```
2.  **Build Release**:
    ```bash
    # Static binary (Musl) ensures it runs on any Android version
    cross build --target aarch64-unknown-linux-musl --release
    ```
3.  **Locate Binary**:
    The binary will be in `target/aarch64-unknown-linux-musl/release/autolinux`.

---

## Contributing

Contributions are welcome!
1.  Fork the project
2.  Create your Feature Branch (`git checkout -b feature/AmazingFeature`)
3.  Commit your Changes (`git commit -m 'Add some AmazingFeature'`)
4.  Push to the Branch (`git push origin feature/AmazingFeature`)
5.  Open a Pull Request

---

## Disclaimer

This tool modifies system partitions (mounting) and creates files in `/data`. While safe, **I am not responsible for any bricked devices or data loss.** Always backup your data.

---

## License

Distributed under the MIT License. See [LICENSE](LICENSE)for more information.
