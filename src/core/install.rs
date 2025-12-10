use std::fs::{self, File};
use std::io::{self, Write, Read};
use std::path::{Path, PathBuf};
use std::process::Command;
use crate::core::distro::Distro;
use crate::{log_info, log_error};
use flate2::read::GzDecoder;
use tar::Archive;
use std::net::SocketAddr;

pub enum InstallState {
    Starting,
    Downloading(f32),
    Extracting,
    Configuring,
    Finished(String),
    Error(String),
}

struct AndroidResolver;

impl ureq::Resolver for AndroidResolver {
    fn resolve(&self, netloc: &str) -> io::Result<Vec<SocketAddr>> {
        let parts: Vec<&str> = netloc.split(':').collect();
        if parts.is_empty() {
            return Err(io::Error::new(io::ErrorKind::InvalidInput, "Invalid netloc"));
        }

        let hostname = parts[0];
        let port: u16 = parts.get(1).unwrap_or(&"443").parse().unwrap_or(443);

        match resolve_hostname_via_android(hostname) {
            Ok(ip_str) => {
                let ip: std::net::IpAddr = ip_str.parse().map_err(|e| {
                    io::Error::new(io::ErrorKind::InvalidData, format!("Invalid IP from ping: {}", e))
                })?;

                Ok(vec![SocketAddr::new(ip, port)])
            },
            Err(e) => {
                Err(io::Error::new(io::ErrorKind::Other, e))
            }
        }
    }
}

pub fn install_distro(
    distro: &Distro,
    username: &str,
    password: &str,
    callback: impl Fn(InstallState)
) -> Result<(), Box<dyn std::error::Error>> {

    callback(InstallState::Starting);

    let folder_name = format!("{}-{}-{}",
        distro.name.split_whitespace().next().unwrap().to_lowercase(),
        distro.codename,
        distro.version
    );
    let install_path = PathBuf::from("/data/local").join(&folder_name);
    let start_script_path = PathBuf::from("/data/local").join(format!("start-{}.sh", folder_name));

    if install_path.exists() {
        fs::remove_dir_all(&install_path)?;
    }
    fs::create_dir_all(&install_path)?;

    callback(InstallState::Downloading(0.0));
    let tar_path = install_path.join("rootfs.tar.gz");

    log_info!("Downloading from: {}", distro.url);

    let agent = ureq::AgentBuilder::new()
        .resolver(AndroidResolver)
        .build();

    let resp = match agent.get(&distro.url).call() {
        Ok(r) => r,
        Err(ureq::Error::Status(code, response)) => {
            let status_text = response.status_text().to_string();
            let err_msg = format!("HTTP Error {}: {} - URL: {}", code, status_text, distro.url);
            log_error!("{}", err_msg);
            return Err(err_msg.into());
        },
        Err(ureq::Error::Transport(t)) => {
            let err_msg = format!("Network Error: {} - URL: {}", t, distro.url);
            log_error!("{}", err_msg);
            return Err(err_msg.into());
        }
    };

    let len = resp.header("Content-Length")
        .and_then(|s| s.parse::<usize>().ok())
        .unwrap_or(0);

    log_info!("Content-Length: {} bytes", len);

    let mut reader = resp.into_reader();
    let mut file = File::create(&tar_path)?;
    let mut buffer = [0; 8192];
    let mut downloaded = 0;
    loop {
        let n = reader.read(&mut buffer)?;
        if n == 0 { break; }
        file.write_all(&buffer[..n])?;
        downloaded += n;

        if len > 0 {
            let pct = (downloaded as f32 / len as f32) * 100.0;
            callback(InstallState::Downloading(pct));
        }
    }

    log_info!("Download complete. Saved {} bytes.", downloaded);

    log_info!("Extracting archive...");
    callback(InstallState::Extracting);
    let tar_gz = File::open(&tar_path)?;
    let tar = GzDecoder::new(tar_gz);
    let mut archive = Archive::new(tar);

    archive.set_preserve_permissions(true);
    archive.set_preserve_mtime(true);
    archive.unpack(&install_path)?;

    fs::remove_file(&tar_path)?;
    log_info!("Extraction complete. Removed temporary archive.");

    log_info!("Generating config files...");
    callback(InstallState::Configuring);

    fs::create_dir_all(install_path.join("sdcard"))?;
    fs::create_dir_all(install_path.join("dev/shm"))?;
    log_info!("Created mount points.");

    let resolv_path = install_path.join("etc/resolv.conf");
    if resolv_path.is_symlink() || resolv_path.exists() {
        let _ = fs::remove_file(&resolv_path);
    }
    fs::write(resolv_path, "nameserver 8.8.8.8\nnameserver 8.8.4.4\n")?;
    log_info!("Wrote DNS configuration.");

    let hosts_path = install_path.join("etc/hosts");
    fs::write(hosts_path, "127.0.0.1 localhost\n::1 localhost ip6-localhost ip6-loopback\n")?;
    log_info!("Wrote hosts configuration.");

    log_info!("Generating startup scripts...");
    generate_start_script(&start_script_path, &install_path, &distro.name)?;
    generate_internal_setup_script(&install_path, username, password, &distro.name)?;
    log_info!("Scripts generated successfully.");

    log_info!("Installation finished successfully at {:?}", install_path);
    callback(InstallState::Finished(start_script_path.to_string_lossy().to_string()));

    Ok(())
}

fn resolve_hostname_via_android(hostname: &str) -> Result<String, String> {
    let output = Command::new("ping")
        .arg("-c").arg("1")
        .arg("-w").arg("2")
        .arg(hostname)
        .output()
        .map_err(|e| format!("Failed to execute ping: {}", e))?;

    let stdout = String::from_utf8_lossy(&output.stdout);

    if let Some(start_idx) = stdout.find('(') {
        if let Some(end_idx) = stdout[start_idx..].find(')') {
            let ip = &stdout[start_idx + 1 .. start_idx + end_idx];
            if ip.contains('.') || ip.contains(':') {
                return Ok(ip.to_string());
            }
        }
    }

    Err(format!("Could not parse IP from ping output: {}", stdout))
}

fn generate_start_script(script_path: &Path, install_path: &Path, distro_name: &str) -> io::Result<()> {
    let path_str = install_path.to_string_lossy();
    let is_alpine = distro_name.to_lowercase().contains("alpine");

    let shell_cmd = if is_alpine { "/bin/sh" } else { "/bin/bash" };

    let content = format!(r#"#!/bin/sh
UBUNTUPATH="{}"
TARGET_USER="${{1:-root}}"

mnt() {{
    if [ -x "$(command -v busybox)" ]; then
        busybox mount "$@"
    else
        /system/bin/mount "$@"
    fi
}}

echo "[*] Mounting system folders..."
mnt -o remount,dev,suid /data

[ ! -d "$UBUNTUPATH/dev/shm" ] && mkdir -p "$UBUNTUPATH/dev/shm"

mnt --bind /dev "$UBUNTUPATH/dev"
mnt --bind /sys "$UBUNTUPATH/sys"
mnt --bind /proc "$UBUNTUPATH/proc"
mnt -t devpts devpts "$UBUNTUPATH/dev/pts"
mnt -t tmpfs -o size=256M tmpfs "$UBUNTUPATH/dev/shm"
mnt --bind /sdcard "$UBUNTUPATH/sdcard"

for pam_file in "$UBUNTUPATH/etc/pam.d/su" "$UBUNTUPATH/etc/pam.d/su-l"; do
  if [ -f "$pam_file" ]; then
    sed -i 's/^\(session.*pam_keyinit.so\)/#\1/' "$pam_file"
  fi
done

if [ -f "$UBUNTUPATH/root/finalize_setup.sh" ]; then
    echo "[!] First time setup detected. Configuring users & groups..."
    chmod +x "$UBUNTUPATH/root/finalize_setup.sh"

    # FIX 1: Gunakan shell yang sesuai distro untuk setup
    if [ -x "$(command -v busybox)" ]; then
        busybox chroot "$UBUNTUPATH" {} /root/finalize_setup.sh
    else
        /system/bin/chroot "$UBUNTUPATH" {} /root/finalize_setup.sh
    fi
    rm "$UBUNTUPATH/root/finalize_setup.sh"
fi

echo "[*] Entering Chroot as $TARGET_USER..."
echo "Type 'exit' to leave."

if [ -x "$(command -v busybox)" ]; then
    busybox chroot "$UBUNTUPATH" /bin/su - "$TARGET_USER"
else
    /system/bin/chroot "$UBUNTUPATH" /bin/su - "$TARGET_USER"
fi
"#, path_str, shell_cmd, shell_cmd);

    fs::write(script_path, content)?;
    use std::os::unix::fs::PermissionsExt;
    let mut perms = fs::metadata(script_path)?.permissions();
    perms.set_mode(0o755);
    fs::set_permissions(script_path, perms)?;

    Ok(())
}

fn generate_internal_setup_script(install_path: &Path, username: &str, password: &str, distro_name: &str) -> io::Result<()> {
    let is_alpine = distro_name.to_lowercase().contains("alpine");

    let package_logic = if is_alpine {
        r#"
echo ">>> (Alpine) Updating Repository..."
echo "http://dl-cdn.alpinelinux.org/alpine/edge/main" > /etc/apk/repositories
echo "http://dl-cdn.alpinelinux.org/alpine/edge/community" >> /etc/apk/repositories
apk update

echo ">>> (Alpine) Installing Base Tools..."
apk add bash shadow sudo nano net-tools git
"#
    } else {
        r#"
echo ">>> (Debian/Ubuntu) Updating Repository..."
apt update -y

echo ">>> (Debian/Ubuntu) Installing Tools..."
apt install -y nano net-tools sudo git
"#
    };

    let content = format!(r#"#!/bin/sh
export PATH=/usr/local/sbin:/usr/local/bin:/usr/sbin:/usr/bin:/sbin:/bin

{}

echo ">>> Configuring Sudo Access..."
# Izinkan group 'wheel' untuk menggunakan sudo
# Kita append ke akhir file agar tidak merusak konfigurasi existing
if [ -f /etc/sudoers ]; then
    echo '%wheel ALL=(ALL:ALL) ALL' >> /etc/sudoers
fi

echo ">>> Configuring Network Groups..."
# Android GID mapping
groupadd -g 3003 aid_inet || true
groupadd -g 3004 aid_net_raw || true
groupadd -g 1003 aid_graphics || true

if [ -f /etc/debian_version ]; then
    usermod -g 3003 -G 3003,3004 -a _apt 2>/dev/null || true
fi
usermod -G 3003 -a root

echo ">>> Creating User '{1}'..."
groupadd storage || true
groupadd wheel || true

useradd -m -g users -G wheel,audio,video,storage,aid_inet -s /bin/bash {1}

echo "{1}:{2}" | chpasswd

echo ">>> Done!"
"#, package_logic, username, password);

    let setup_path = install_path.join("root/finalize_setup.sh");
    fs::write(setup_path, content)?;
    Ok(())
}
