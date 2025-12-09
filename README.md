# Auto-Linux

![Build Status](https://img.shields.io/github/actions/workflow/status/HanSoBored/Auto-Linux/build.yml?branch=main)
![Language](https://img.shields.io/badge/language-Rust-orange)
![Platform](https://img.shields.io/badge/platform-Android%20(Root)-green)
![License](https://img.shields.io/badge/license-MIT-blue)

**Auto-Linux** is a standalone, lightweight Linux installer and manager for **rooted** Android devices. Written entirely in Rust, it provides an intuitive Terminal User Interface (TUI) to install and manage Ubuntu chroot environments without external dependencies like Termux or Busybox.

> Focusing on a minimal, dependency-free, and robust chroot experience.

---

## Key Features

*   **Truly Standalone:** Compiled as a single, static `musl` binary. It has zero runtime dependencies and does not require Termux to function.
*   **Intuitive TUI:** A clean, keyboard-driven dashboard powered by `ratatui` for easy navigation and management.
*   **Direct Chroot Management:** Launch directly into a chroot environment, select users (`root` or a standard user), and manage installed distributions from the dashboard.
*   **Smart Root Elevation:** The application automatically detects if it's running without root privileges and attempts to elevate itself via `su`.
*   **Automated Configuration:**
    -   **Network:** Seamlessly sets up DNS (`resolv.conf`) to ensure internet connectivity inside the chroot.
    -   **Users:** Creates a standard user with a password and `sudo` privileges during the initial setup.
    -   **System Mounts:** Automatically handles bind mounts for `/dev`, `/proc`, `/sys`, and `/sdcard`.
*   **Broad Distribution Support:** Installs official Ubuntu base images from 20.04 LTS up to the latest 24.10 release.

---

## Screenshots

*Dashboard:*
![preview 1](https://raw.githubusercontent.com/HanSoBored/Auto-Linux/main/preview/preview1.jpg)

*Distro List & Installation :*
![preview 2](https://raw.githubusercontent.com/HanSoBored/Auto-Linux/main/preview/preview2.jpg)

*User Selection:*
![preview 3](https://raw.githubusercontent.com/HanSoBored/Auto-Linux/main/preview/preview3.jpg)

---

## Installation

**Requirement:** A rooted Android device with an `su` binary in `PATH`.

### Recommended: Quick Install Script
Execute this command in a terminal environment like **Termux** or an **ADB Shell**:

```bash
curl -sL https://raw.githubusercontent.com/HanSoBored/Auto-Linux/main/install.sh | sh
```
> The script automatically detects your device architecture, downloads the correct binary to `/data/local/tmp`, sets permissions, and creates a convenient `autolinux` alias if you are using Termux.

### Manual Installation
1.  Download the latest `autolinux-aarch64` binary from the [Releases](https://github.com/HanSoBored/Auto-Linux/releases) page.
2.  Push the binary to your device:
    ```sh
    adb push autolinux-aarch64 /data/local/tmp/autolinux
    ```
3.  Make it executable:
    ```sh
    adb shell "chmod +x /data/local/tmp/autolinux"
    ```
4.  Run the application:
    ```sh
    adb shell "/data/local/tmp/autolinux"
    ```

---

## Build from Source

### Prerequisites
-   [Rust](https://www.rust-lang.org/tools/install) toolchain.
-   [Cross](https://github.com/cross-rs/cross) for easy cross-compilation.
    ```bash
    cargo install cross
    ```

### Build Command
Compile for `aarch64` using the `musl` target to create a fully static binary. This ensures maximum compatibility across different Android versions by not depending on the system's C library (Bionic).

```bash
cross build --target aarch64-unknown-linux-musl --release
```

The final binary will be located at `target/aarch64-unknown-linux-musl/release/autolinux`.

---

## Contributing

Contributions are highly welcome! Please follow these steps:
1.  Fork the project.
2.  Create your feature branch (`git checkout -b feature/AmazingFeature`).
3.  Commit your changes (`git commit -m 'Add some AmazingFeature'`).
4.  Push to the branch (`git push origin feature/AmazingFeature`).
5.  Open a Pull Request.

---

## Disclaimer

This tool requires root and performs system-level operations like mounting filesystems and modifying the `/data` partition. While it is designed to be safe, **the author is not responsible for any data loss or damage to your device.** Always ensure you have a backup of important data.

---

## License

This project is distributed under the MIT License. See the [LICENSE](LICENSE) for more information.