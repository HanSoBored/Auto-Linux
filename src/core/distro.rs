use std::fs;
use std::path::PathBuf;
use crate::types::InstalledDistro;
use std::io::{BufRead, BufReader};

#[derive(Clone)]
pub struct Distro {
    pub name: String,
    pub codename: String,
    pub version: String,
    pub url: String,
}

impl Distro {
    pub fn get_ubuntu_flavors(arch: &str) -> Vec<Distro> {
        let deb_arch = match arch {
            "aarch64" => "arm64",
            "x86_64" => "amd64",
            _ => "armhf",
        };

        vec![
            Distro {
                name: "Ubuntu 20.04 LTS (Focal Fossa)".to_string(),
                codename: "focal".to_string(),
                version: "20.04.5".to_string(),
                url: format!("https://cdimage.ubuntu.com/ubuntu-base/releases/20.04/release/ubuntu-base-20.04.5-base-{}.tar.gz", deb_arch),
            },
            Distro {
                name: "Ubuntu 22.04 LTS (Jammy Jellyfish)".to_string(),
                codename: "jammy".to_string(),
                version: "22.04.5".to_string(),
                url: format!("https://cdimage.ubuntu.com/ubuntu-base/releases/22.04/release/ubuntu-base-22.04.5-base-{}.tar.gz", deb_arch),
            },
            Distro {
                name: "Ubuntu 24.04 LTS (Noble Numbat)".to_string(),
                codename: "noble".to_string(),
                version: "24.04.3".to_string(),
                url: format!("https://cdimage.ubuntu.com/ubuntu-base/releases/24.04/release/ubuntu-base-24.04.3-base-{}.tar.gz", deb_arch),
            },
            Distro {
                name: "Ubuntu 26.04 LTS (Resolute Raccoon)".to_string(),
                codename: "resolute".to_string(),
                version: "26.04".to_string(),
                url: format!("https://cdimage.ubuntu.com/ubuntu-base/releases/resolute/snapshot1/ubuntu-base-26.04-base-{}.tar.gz", deb_arch),
            },
        ]
    }

    pub fn scan_installed_distros() -> Vec<InstalledDistro> {
        let mut results = Vec::new();
        let base_path = PathBuf::from("/data/local");

        if let Ok(entries) = fs::read_dir(&base_path) {
            for entry in entries.flatten() {
                let path = entry.path();
                if path.is_dir() {
                    let _bash_path = path.join("bin/bash");

                    let folder_name = path.file_name().unwrap().to_string_lossy();
                    let script_name = format!("start-{}.sh", folder_name);
                    let script_path = base_path.join(&script_name);

                    if script_path.exists() && path.join("etc/os-release").exists() {
                        let users = Self::get_users_from_passwd(&path.join("etc/passwd"));

                        results.push(InstalledDistro {
                            name: folder_name.to_string(),
                            path: path.clone(),
                            script_path,
                            users,
                        });
                    }
                }
            }
        }
        results
    }

    fn get_users_from_passwd(passwd_path: &PathBuf) -> Vec<String> {
        let mut users = vec!["root".to_string()];

        if let Ok(file) = fs::File::open(passwd_path) {
            let reader = BufReader::new(file);
            for line in reader.lines().flatten() {
                let parts: Vec<&str> = line.split(':').collect();
                if parts.len() >= 3 {
                    if let Ok(uid) = parts[2].parse::<u32>() {
                        if uid >= 1000 && uid < 60000 {
                            users.push(parts[0].to_string());
                        }
                    }
                }
            }
        }
        users
    }
}