use std::fs::{self, OpenOptions};
use std::io::Write;
use std::path::PathBuf;
use std::env;
use std::time::{SystemTime, UNIX_EPOCH};

pub fn get_log_path() -> PathBuf {
    let home = env::var("HOME").unwrap_or_else(|_| "/data/local/tmp".to_string());

    PathBuf::from(home).join(".local/share/auto-linux/debug.logs")
}

pub fn init() {
    let path = get_log_path();

    if let Some(parent) = path.parent() {
        let _ = fs::create_dir_all(parent);
    }

    write_log("INFO", "=== APPLICATION STARTED ===");
    write_log("INFO", &format!("Log Path: {:?}", path));

    std::panic::set_hook(Box::new(|info| {
        let msg = match info.payload().downcast_ref::<&str>() {
            Some(s) => *s,
            None => match info.payload().downcast_ref::<String>() {
                Some(s) => &s[..],
                None => "Box<Any>",
            },
        };

        let location = if let Some(location) = info.location() {
            format!("{}:{}", location.file(), location.line())
        } else {
            "unknown".to_string()
        };

        let err_msg = format!("CRITICAL PANIC at {}: {}", location, msg);
        eprintln!("{}", err_msg);
        write_log("FATAL", &err_msg);
    }));
}

pub fn write_log(level: &str, msg: &str) {
    let path = get_log_path();

    let timestamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs();

    let log_line = format!("[{}] [{}] {}\n", timestamp, level, msg);

    if let Ok(mut file) = OpenOptions::new().create(true).append(true).open(path) {
        let _ = file.write_all(log_line.as_bytes());
    }
}

#[macro_export]
macro_rules! log_info {
    ($($arg:tt)*) => {
        $crate::core::logger::write_log("INFO", &format!($($arg)*));
    }
}

#[macro_export]
macro_rules! log_error {
    ($($arg:tt)*) => {
        $crate::core::logger::write_log("ERROR", &format!($($arg)*));
    }
}