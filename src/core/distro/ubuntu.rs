use super::{Distro, DistroFamily};

pub fn get_family(arch: &str) -> DistroFamily {
    let deb_arch = match arch { "aarch64" => "arm64", "x86_64" => "amd64", _ => "armhf" };

    DistroFamily {
        name: "Ubuntu".to_string(),
        description: "Popular, user-friendly, Debian-based.".to_string(),
        variants: vec![
            Distro {
                name: "Ubuntu 20.04 LTS (Focal Fossa)".to_string(), codename: "focal".to_string(), version: "20.04.5".to_string(),
                url: format!("https://cdimage.ubuntu.com/ubuntu-base/releases/20.04/release/ubuntu-base-20.04.5-base-{}.tar.gz", deb_arch),
            },
            Distro {
                name: "Ubuntu 22.04 LTS (Jammy Jellyfish)".to_string(), codename: "jammy".to_string(), version: "22.04.5".to_string(),
                url: format!("https://cdimage.ubuntu.com/ubuntu-base/releases/22.04/release/ubuntu-base-22.04.5-base-{}.tar.gz", deb_arch),
            },
            Distro {
                name: "Ubuntu 24.04 LTS (Noble Numbat)".to_string(), codename: "noble".to_string(), version: "24.04.3".to_string(),
                url: format!("https://cdimage.ubuntu.com/ubuntu-base/releases/24.04/release/ubuntu-base-24.04.3-base-{}.tar.gz", deb_arch),
            },
            Distro {
                name: "Ubuntu 26.04 LTS (Resolute Raccoon)".to_string(), codename: "resolute".to_string(), version: "26.04".to_string(),
                url: format!("https://cdimage.ubuntu.com/ubuntu-base/releases/26.04/snapshot1/ubuntu-base-26.04-base-{}.tar.gz", deb_arch),
            },

        ],
    }
}
