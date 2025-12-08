use std::process::Command;
use std::env;

pub struct DeviceInfo {
    pub arch: String,
    pub is_root: bool,
    pub can_su: bool,
    pub root_type: String,
    pub android_ver: String,
}

impl DeviceInfo {
    pub fn new() -> Self {
        let current_root = check_current_user_root();
        let (su_available, r_type) = check_su_access();

        Self {
            arch: get_arch(),
            is_root: current_root,
            can_su: su_available,
            root_type: r_type,
            android_ver: get_android_version(),
        }
    }
}

fn get_arch() -> String {
    env::consts::ARCH.to_string()
}

fn check_current_user_root() -> bool {
    if let Ok(output) = Command::new("id").arg("-u").output() {
        let uid_str = String::from_utf8_lossy(&output.stdout).trim().to_string();
        return uid_str == "0";
    }
    false
}

fn check_su_access() -> (bool, String) {
    if let Ok(output) = Command::new("su").arg("-v").output() {
        if output.status.success() {
            let version_str = String::from_utf8_lossy(&output.stdout).trim().to_string();

            let type_str = if version_str.to_lowercase().contains("magisk") {
                "Magisk".to_string()
            } else if version_str.to_lowercase().contains("kernelsu") {
                "KernelSU".to_string()
            } else if version_str.to_lowercase().contains("apatch") {
                "APatch".to_string()
            } else {
                format!("Generic ({})", version_str)
            };

            return (true, type_str);
        }
    }

    let known_paths = vec![
        "/sbin/su",
        "/system/bin/su",
        "/system/xbin/su",
        "/data/adb/ksu/bin/su",
        "/data/adb/apatch/bin/su"
    ];

    for path in known_paths {
        if let Ok(output) = Command::new(path).arg("-v").output() {
            if output.status.success() {
                return (true, "Hidden/Systemless Root".to_string());
            }
        }
    }

    (false, "None".to_string())
}

fn get_android_version() -> String {
    if let Ok(output) = Command::new("getprop").arg("ro.build.version.release").output() {
        return String::from_utf8_lossy(&output.stdout).trim().to_string();
    }
    "Unknown".to_string()
}