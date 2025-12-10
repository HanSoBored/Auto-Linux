use super::{Distro, DistroFamily};

pub fn get_family(arch: &str) -> DistroFamily {
    let alp_arch = match arch { "aarch64" => "aarch64", "x86_64" => "x86_64", _ => "armv7" };

    DistroFamily {
        name: "Alpine Linux".to_string(),
        description: "Security-oriented, lightweight (musl libc & busybox).".to_string(),
        variants: vec![
            Distro {
                name: "Alpine 3.20".to_string(), codename: "release".to_string(), version: "3.20.8".to_string(),
                url: format!("https://dl-cdn.alpinelinux.org/alpine/v3.20/releases/{}/alpine-minirootfs-3.20.8-{}.tar.gz", alp_arch, alp_arch),
            },
            Distro {
                name: "Alpine 3.21".to_string(), codename: "release".to_string(), version: "3.21.5".to_string(),
                url: format!("https://dl-cdn.alpinelinux.org/alpine/v3.21/releases/{}/alpine-minirootfs-3.21.5-{}.tar.gz", alp_arch, alp_arch),
            },
            Distro {
                name: "Alpine 3.22".to_string(), codename: "release".to_string(), version: "3.22.2".to_string(),
                url: format!("https://dl-cdn.alpinelinux.org/alpine/v3.22/releases/{}/alpine-minirootfs-3.22.2-{}.tar.gz", alp_arch, alp_arch),
            },
            Distro {
                name: "Alpine 3.23".to_string(), codename: "release".to_string(), version: "3.23.0".to_string(),
                url: format!("https://dl-cdn.alpinelinux.org/alpine/v3.23/releases/{}/alpine-minirootfs-3.23.0-{}.tar.gz", alp_arch, alp_arch),
            },
            Distro {
                name: "Alpine Edge".to_string(), codename: "edge".to_string(), version: "rolling".to_string(),
                url: format!("https://dl-cdn.alpinelinux.org/alpine/edge/releases/{}/alpine-minirootfs-20251016-{}.tar.gz", alp_arch, alp_arch),
            },
        ],
    }
}
