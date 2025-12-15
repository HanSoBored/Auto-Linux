use super::{Distro, DistroFamily};

pub fn get_family(arch: &str) -> DistroFamily {
    let fed_arch = match arch { "aarch64" => "aarch64", "x86_64" => "x86_64", _ => "armhfp" };

    DistroFamily {
        name: "Fedora".to_string(),
        description: "Cutting-edge, community-driven Red Hat distro.".to_string(),
        variants: vec![
            Distro {
                name: "Fedora 40".to_string(), codename: "40".to_string(), version: "40-1.14".to_string(),
                url: format!("https://archives.fedoraproject.org/pub/archive/fedora/linux/releases/40/Container/{}/images/Fedora-Container-Base-Generic.{}-40-1.14.oci.tar.xz", fed_arch, fed_arch),
            },
            Distro {
                name: "Fedora 41".to_string(), codename: "41".to_string(), version: "41-1.4".to_string(),
                url: format!("https://mirror.twds.com.tw/fedora/fedora/linux/releases/41/Container/{}/images/Fedora-Container-Base-Generic-41-1.4.{}.oci.tar.xz", fed_arch, fed_arch),
            },
            Distro {
                name: "Fedora 42 (Adams)".to_string(), codename: "adams".to_string(), version: "42-1.1".to_string(),
                url: format!("https://mirror.twds.com.tw/fedora/fedora/linux/releases/42/Container/{}/images/Fedora-Container-Base-Generic-42-1.1.{}.oci.tar.xz", fed_arch, fed_arch),
            },
            Distro {
                name: "Fedora 43".to_string(), codename: "43".to_string(), version: "43-1.6".to_string(),
                url: format!("https://mirror.twds.com.tw/fedora/fedora/linux/releases/43/Container/{}/images/Fedora-Container-Base-Generic-43-1.6.{}.oci.tar.xz", fed_arch, fed_arch),
            },
        ],
    }
}
