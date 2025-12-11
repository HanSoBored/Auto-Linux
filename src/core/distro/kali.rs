use super::{Distro, DistroFamily};

pub fn get_family(arch: &str) -> DistroFamily {
    let deb_arch = match arch { "aarch64" => "arm64", "x86_64" => "amd64", _ => "armhf" };

    DistroFamily {
        name: "Kali Linux".to_string(),
        description: "Security-focused distro for penetration testing.".to_string(),
        variants: vec![
            Distro {
                name: "Kali Linux 2025.1".to_string(), codename: "kali-rolling".to_string(), version: "2025.1c".to_string(),
                url: format!("https://kali.download/nethunter-images/kali-2025.1c/rootfs/kali-nethunter-rootfs-nano-{}.tar.xz", deb_arch),
            },
            Distro {
                name: "Kali Linux 2025.2".to_string(), codename: "kali-rolling".to_string(), version: "2025.2".to_string(),
                    url: format!("https://kali.download/nethunter-images/kali-2025.2/rootfs/kali-nethunter-rootfs-nano-{}.tar.xz", deb_arch),
            },
            Distro {
                name: "Kali Linux 2025.3".to_string(), codename: "kali-rolling".to_string(), version: "2025.2".to_string(),
                    url: format!("https://kali.download/nethunter-images/kali-2025.3/rootfs/kali-nethunter-rootfs-nano-{}.tar.xz", deb_arch),
            },
            Distro {
                name: "Kali Linux (Current)".to_string(), codename: "kali-rolling".to_string(), version: "latest".to_string(),
                    url: format!("https://kali.download/nethunter-images/current/rootfs/kali-nethunter-rootfs-nano-{}.tar.xz", deb_arch),
            },
        ],
    }
}
