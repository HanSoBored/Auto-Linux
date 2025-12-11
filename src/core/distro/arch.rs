use super::{Distro, DistroFamily};

pub fn get_family(arch: &str) -> DistroFamily {
    if arch == "aarch64" {
        return DistroFamily {
            name: "Arch Linux".to_string(),
            description: "Rolling release, lightweight, pacman package manager.".to_string(),
            variants: vec![
                Distro {
                    name: "Arch Linux ARM (Generic)".to_string(), codename: "rolling".to_string(), version: "latest".to_string(),
                    url: "http://os.archlinuxarm.org/os/ArchLinuxARM-aarch64-latest.tar.gz".to_string(),
                },
            ],
        };
    }
    DistroFamily { name: "Arch Linux".to_string(), description: "AArch64 Only for now".to_string(), variants: vec![] }
}
