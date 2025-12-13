use super::{Distro, DistroFamily};

pub fn get_family(arch: &str) -> DistroFamily {
    let void_arch = match arch { "aarch64" => "aarch64", "x86_64" => "x86_64-musl", _ => "armv7l-musl" };

    DistroFamily {
        name: "Void Linux".to_string(),
        description: "Modern Linux distro with rolling releases and XBPS.".to_string(),
        variants: vec![
            Distro {
                name: "Void Linux".to_string(), codename: "rolling".to_string(), version: "20240314".to_string(),
                url: format!("https://repo-default.voidlinux.org/live/20240314/void-{}-ROOTFS-20240314.tar.xz", void_arch),
            },
            Distro {
                name: "Void Linux".to_string(), codename: "rolling".to_string(), version: "20250202".to_string(),
                url: format!("https://repo-default.voidlinux.org/live/20250202/void-{}-ROOTFS-20250202.tar.xz", void_arch),
            },
        ],
    }
}
