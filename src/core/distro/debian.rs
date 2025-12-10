use super::{Distro, DistroFamily};

pub fn get_family(arch: &str) -> DistroFamily {
    if arch == "aarch64" {
        return DistroFamily {
            name: "Debian".to_string(),
            description: "Stable, reliable, widely-used server distro.".to_string(),
            variants: vec![
               Distro {
                    name: "Debian 11 (Bullseye)".to_string(), codename: "bullseye".to_string(), version: "11".to_string(),
                    url: "https://github.com/HanSoBored/Debootstrap-Linux/releases/download/debian/debian-bullseye-aarch64.tar.gz".to_string(),
                },
                Distro {
                    name: "Debian 12 (Bookworm)".to_string(), codename: "bookworm".to_string(), version: "12".to_string(),
                    url: "https://github.com/HanSoBored/Debootstrap-Linux/releases/download/debian/debian-bookworm-aarch64.tar.gz".to_string(),
                },
                Distro {
                    name: "Debian 13 (Trixie)".to_string(), codename: "trixie".to_string(), version: "13".to_string(),
                    url: "https://github.com/HanSoBored/Debootstrap-Linux/releases/download/debian/debian-trixie-aarch64.tar.gz".to_string(),
                },
            ],
        };
    }
    DistroFamily { name: "Debian".to_string(), description: "AArch64 Only for now".to_string(), variants: vec![] }
}
