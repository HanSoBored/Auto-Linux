# Auto-Linux

![Build Status](https://img.shields.io/github/actions/workflow/status/HanSoBored/Auto-Linux/build.yml?branch=main)
![Language](https://img.shields.io/badge/language-Rust-orange)
![Platform](https://img.shields.io/badge/platform-Android%20(Root)-green)
![License](https://img.shields.io/badge/license-MIT-blue)

**Auto-Linux** is a standalone, advanced Linux installer and manager for **rooted** Android devices. Built with Rust, it provides a feature-rich Terminal User Interface (TUI) to install, configure, and manage various Linux distributions in a Chroot environment without relying on Termux, Busybox, or other external dependencies.

> **Beyond simple scripts:** Auto-Linux handles complex tasks like OCI image extraction, host-side security attribute stripping, and DNS injection to ensure modern distros (like Fedora & Void) run smoothly on Android.

---

## Key Features

*   **Truly Standalone:** Compiled as a single static `musl` binary (~2MB). Zero runtime dependencies.
*   **Intuitive TUI:** Keyboard-driven dashboard (powered by `ratatui`) for distro selection, credential setup, and one-click launching.
*   **Multi-Distro Support:**
    *   **Debian/Ubuntu:** Ubuntu (20.04 - 26.04), Debian, Kali Linux.
    *   **Rolling Release:** Arch Linux ARM (with automatic Keyring init), Void Linux.
    *   **RPM-Based:** Fedora (Automatic OCI blob extraction & attribute cleanup).
    *   **Lightweight:** Alpine Linux.
*   **Advanced Extraction Engine:**
    *   Auto-detects and handles `.tar.gz`, `.tar.xz`, and **OCI (Docker) Image** formats.
    *   Automatically flattens nested rootfs structures (e.g., Kali).
*   **Smart Configuration:**
    *   **Robust Networking:** Uses a wrapper strategy and DNS injection (via host `ping` resolution) to bypass Android's GID permission delays and broken DNS resolvers in Chroot.
    *   **Security Cleanup:** Features a unique **Host-Side Hook** to strip `security.ima` and `security.selinux` attributes, allowing distros like Fedora to run on Android kernels that enforce strict keyring checks.
    *   **User Management:** Automatically handles `groupadd`/`useradd` compatibility across `shadow` (standard) and `busybox` (Alpine) backends.

---

## Supported Distributions

Auto-Linux currently supports fetching and installing the following families:

| Family | Distributions | Key Features |
| :--- | :--- | :--- |
| **Ubuntu** | 20.04, 22.04, 24.04, 26.04 | Standard environment, robust support. |
| **Debian** | Debian Stable | Pure Debian experience. |
| **Security** | **Kali Linux** | Includes flat-rootfs handling & network fix. |
| **Alpine** | Edge, Latest Stable | Extremely lightweight, uses `apk`. |
| **Arch** | **Arch Linux ARM** | Auto-initializes `pacman-keyring` & fixes mirrorlists. |
| **Void** | **Void Linux** | Fixes `xbps` networking & enforces SHA512 passwords. |
| **Fedora** | Fedora 40, 41, 42, 43 Latest | Handles OCI blobs & strips kernel security xattrs. |

---

## Installation

**Requirement:** A rooted Android device with an `su` binary.

### Option 1: Quick Install (Termux/ADB)
```bash
curl -sL https://raw.githubusercontent.com/HanSoBored/Auto-Linux/main/install.sh | sh
```

### Option 2: Manual Push
1.  Download the latest binary from [Releases](https://github.com/HanSoBored/Auto-Linux/releases).
2.  Push to device:
    ```sh
    adb push autolinux-aarch64 /data/local/rootfs/autolinux
    adb shell "chmod +x /data/local/rootfs/autolinux"
    ```
3.  Run:
    ```sh
    adb shell
    su -c /data/local/rootfs/autolinux
    ```

---

## Screenshots

| Dashboard | Family Selection |
| :---: | :---: |
| ![Dashboard](https://raw.githubusercontent.com/HanSoBored/Auto-Linux/main/preview/Dashboard.jpg) | ![Distro-List](https://raw.githubusercontent.com/HanSoBored/Auto-Linux/main/preview/Distro-List.jpg) |

| Version List | Installed |
| :---: | :---: |
| ![Version-List](https://raw.githubusercontent.com/HanSoBored/Auto-Linux/main/preview/Distro-Version-List.jpg) | ![Installed](https://raw.githubusercontent.com/HanSoBored/Auto-Linux/main/preview/Distro-Installed-List.jpg) |

---

## Screenshots Distro

| Debian | Ubuntu |
| :---: | :---: |
| ![Debian](https://raw.githubusercontent.com/HanSoBored/Auto-Linux/main/preview/distro/Debian.jpg) | ![Ubuntu](https://raw.githubusercontent.com/HanSoBored/Auto-Linux/main/preview/distro/Ubuntu.jpg) |

| Alpine | Arch |
| :---: | :---: |
| ![Alpine](https://raw.githubusercontent.com/HanSoBored/Auto-Linux/main/preview/distro/Alpine.jpg) | ![Arch](https://raw.githubusercontent.com/HanSoBored/Auto-Linux/main/preview/distro/Arch.jpg) |

| Fedora | Kali | Void |
| :---: | :---: | :---: |
| ![Fedora](https://raw.githubusercontent.com/HanSoBored/Auto-Linux/main/preview/distro/Fedora.jpg) | ![Kali](https://raw.githubusercontent.com/HanSoBored/Auto-Linux/main/preview/distro/Kali.jpg) | ![Void](https://raw.githubusercontent.com/HanSoBored/Auto-Linux/main/preview/distro/Void.jpg) |

---

## Build from Source

To ensure compatibility with Android's libc, we build statically against `musl`.

### Prerequisites
*   Rust Toolchain
*   `cross` (Recommended for easy cross-compilation)

```bash
cargo install cross
```

### Build Command
```bash
# Build a static binary for Android (AArch64)
cross build --target aarch64-unknown-linux-musl --release
```
The binary will be at `target/aarch64-unknown-linux-musl/release/autolinux`.

---

## Troubleshooting

Logs are automatically generated for debugging purposes:
*   **Root:** `/data/local/auto-linux/debug.logs`
*   **User:** `~/.local/share/auto-linux/debug.logs`

**Common Issues:**
*   *Network Error:* If installation fails at downloading, check your internet connection.
*   *Required key not available:* This is usually a Fedora issue. Auto-Linux attempts to fix this automatically via the host-side cleanup hook. If it persists, ensure your kernel supports `setfattr`.

---

## License

This project is distributed under the MIT License. See the [LICENSE](LICENSE) for more information.