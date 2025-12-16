use std::fs;
use std::path::PathBuf;
use crate::types::InstalledDistro;
use std::io::{BufRead, BufReader};

pub mod ubuntu;
pub mod debian;
pub mod alpine;
pub mod arch;
pub mod kali;
// pub mod parrot;
// pub mod postmarket;
pub mod void;
pub mod fedora;

#[derive(Clone, Debug)]
pub struct Distro {
    pub name: String,
    pub codename: String,
    pub version: String,
    pub url: String,
}

#[derive(Clone, Debug)]
pub struct DistroFamily {
    pub name: String,
    pub description: String,
    pub variants: Vec<Distro>,
}

impl Distro {
    pub fn get_all_families(arch: &str) -> Vec<DistroFamily> {
        vec![
            ubuntu::get_family(arch),
            debian::get_family(arch),
            alpine::get_family(arch),
            arch::get_family(arch),
            kali::get_family(arch),
           // parrot::get_family(arch),
            void::get_family(arch),
            fedora::get_family(arch),
           // postmarket::get_family(arch),
        ]
    }

    pub fn scan_installed_distros() -> Vec<InstalledDistro> {
        let mut results = Vec::new();
        let base_path = PathBuf::from("/data/local/rootfs");
        if let Ok(entries) = fs::read_dir(&base_path) {
            for entry in entries.flatten() {
                let path = entry.path();
                if path.is_dir() {
                    let folder_name = path.file_name().unwrap().to_string_lossy();
                    let script_name = format!("start-{}.sh", folder_name);
                    let script_path = base_path.join(&script_name);
                    if script_path.exists() && path.join("etc").exists() {
                        let users = Self::get_users_from_passwd(&path.join("etc/passwd"));
                        results.push(InstalledDistro {
                            name: folder_name.to_string(),
                            path,
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
