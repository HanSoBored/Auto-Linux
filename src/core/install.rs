use std::fs::{self, File};
use std::io::{self, Write, Read, Seek, SeekFrom};
use std::path::{Path, PathBuf};
use std::process::Command;
use std::ffi::CString;
use std::os::unix::ffi::OsStrExt;
use crate::core::distro::Distro;
use crate::{log_info, log_error};
use flate2::read::GzDecoder;
use xz2::read::XzDecoder;
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

    let is_xz = distro.url.ends_with(".xz");
    let filename = if is_xz { "rootfs.tar.xz" } else { "rootfs.tar.gz" };
    let tar_path = install_path.join(filename);

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

    unpack_archive_generic(&tar_path, &install_path)?;
    fs::remove_file(&tar_path)?;

    if install_path.join("oci-layout").exists() || install_path.join("blobs").exists() {
        log_info!("OCI/Container Image format detected (Fedora style). Processing layers...");
        handle_oci_extraction(&install_path)?;
    }

    log_info!("Checking directory structure...");
    if let Err(e) = flatten_nested_rootfs(&install_path) {
        log_error!("Failed to flatten rootfs: {}", e);
    }

    log_info!("Cleaning up security extended attributes (IMA/SELinux)...");
    if let Err(e) = clean_security_xattrs_recursive(&install_path) {
        log_error!("Warning: Failed to clean some xattrs: {}", e);
    }

    log_info!("Generating config files...");
    callback(InstallState::Configuring);

    fs::create_dir_all(install_path.join("sdcard"))?;
    fs::create_dir_all(install_path.join("dev/shm"))?;
    log_info!("Created mount points.");

    let resolv_path = install_path.join("etc/resolv.conf");
    if let Some(parent) = resolv_path.parent() {
        fs::create_dir_all(parent)?;
    }
    if resolv_path.is_symlink() || resolv_path.exists() {
        let _ = fs::remove_file(&resolv_path);
    }
    fs::write(resolv_path, "nameserver 8.8.8.8\nnameserver 8.8.4.4\n")?;


    let hosts_path = install_path.join("etc/hosts");
    let mut hosts_content = "127.0.0.1 localhost\n::1 localhost ip6-localhost ip6-loopback\n".to_string();
    if distro.name.to_lowercase().contains("void") {
        log_info!("(Void Linux Fix) Resolving repo URL for /etc/hosts injection...");
        
        let target_repo = "repo-default.voidlinux.org";
        match resolve_hostname_via_android(target_repo) {
            Ok(ip) => {
                hosts_content.push_str(&format!("{} {}\n", ip, target_repo));
                log_info!("Successfully injected: {} -> {}", ip, target_repo);
            },
            Err(e) => {
                log_error!("Failed to resolve Void repo: {}. Installation might fail.", e);
            }
        }
    }

    fs::write(hosts_path, hosts_content)?;
    log_info!("Wrote hosts configuration.");

    log_info!("Generating startup scripts...");
    generate_start_script(&start_script_path, &install_path, &distro.name)?;
    generate_internal_setup_script(&install_path, username, password, &distro.name)?;
    log_info!("Scripts generated successfully.");

    log_info!("Installation finished successfully at {:?}", install_path);
    callback(InstallState::Finished(start_script_path.to_string_lossy().to_string()));
    Ok(())
}

fn unpack_archive_generic(archive_path: &Path, dest: &Path) -> Result<(), Box<dyn std::error::Error>> {
    let mut file = File::open(archive_path)?;

    let mut magic = [0u8; 6];
    if file.read(&mut magic).is_ok() {
        file.seek(SeekFrom::Start(0))?;
    }

    let decoder: Box<dyn Read> = if magic[0] == 0xFD && magic[1] == 0x37 && magic[2] == 0x7A
        && magic[3] == 0x58 && magic[4] == 0x5A && magic[5] == 0x00 {
        log_info!("Format detected: XZ");
        Box::new(XzDecoder::new(file))
    } else if magic[0] == 0x1F && magic[1] == 0x8B {
        log_info!("Format detected: Gzip");
        Box::new(GzDecoder::new(file))
    } else {
        log_info!("Format unknown (Magic: {:?}), trying Gzip...", magic);
        Box::new(GzDecoder::new(file))
    };

    let mut archive = Archive::new(decoder);
    archive.set_preserve_permissions(true);
    archive.set_preserve_mtime(true);
    archive.unpack(dest)?;

    Ok(())
}

fn handle_oci_extraction(base_path: &Path) -> Result<(), Box<dyn std::error::Error>> {
    let blobs_dir = base_path.join("blobs/sha256");
    if !blobs_dir.exists() {
        return Err("OCI 'blobs' directory not found.".into());
    }

    let mut max_size = 0;
    let mut best_blob_path = PathBuf::new();
    let mut found = false;

    for entry in fs::read_dir(&blobs_dir)? {
        let entry = entry?;
        let meta = entry.metadata()?;
        if meta.is_file() {
            if meta.len() > max_size {
                max_size = meta.len();
                best_blob_path = entry.path();
                found = true;
            }
        }
    }

    if !found {
        return Err("No layers found in OCI image.".into());
    }

    log_info!("Found rootfs layer: {:?} (Size: {} bytes)", best_blob_path, max_size);

    let temp_layer_path = base_path.join("rootfs_layer.tar.gz");
    fs::rename(&best_blob_path, &temp_layer_path)?;

    for entry in fs::read_dir(base_path)? {
        let entry = entry?;
        if entry.path() != temp_layer_path {
            if entry.path().is_dir() {
                fs::remove_dir_all(entry.path())?;
            } else {
                fs::remove_file(entry.path())?;
            }
        }
    }

    log_info!("Extracting inner rootfs layer...");
    unpack_archive_generic(&temp_layer_path, base_path)?;

    fs::remove_file(temp_layer_path)?;

    Ok(())
}

pub fn clean_security_xattrs_recursive(path: &Path) -> io::Result<()> {
    if let Ok(entries) = fs::read_dir(path) {
        for entry in entries {
            if let Ok(entry) = entry {
                let path = entry.path();
                let file_type = entry.file_type()?;

                if file_type.is_dir() {
                    if let Some(name) = path.file_name() {
                        let name_str = name.to_string_lossy();
                        if name_str == "proc" || name_str == "sys" || name_str == "dev" || name_str == "sdcard" {
                            continue;
                        }
                    }
                    let _ = clean_security_xattrs_recursive(&path);
                }

                unsafe {
                    let p_cstr = CString::new(path.as_os_str().as_bytes()).unwrap();
                    let ima = CString::new("security.ima").unwrap();
                    libc::lremovexattr(p_cstr.as_ptr(), ima.as_ptr());
                    let selinux = CString::new("security.selinux").unwrap();
                    libc::lremovexattr(p_cstr.as_ptr(), selinux.as_ptr());
                    let cap = CString::new("security.capability").unwrap();
                    libc::lremovexattr(p_cstr.as_ptr(), cap.as_ptr());
                }
            }
        }
    }
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

fn flatten_nested_rootfs(path: &Path) -> std::io::Result<()> {
    let entries = fs::read_dir(path)?
        .map(|res| res.map(|e| e.path()))
        .collect::<Result<Vec<_>, std::io::Error>>()?;

    if entries.len() == 1 && entries[0].is_dir() {
        let nested_dir = entries[0].clone();
        let nested_name = nested_dir.file_name().unwrap().to_string_lossy();
        log_info!("Nested rootfs detected in subfolder: '{}'. Moving files up...", nested_name);

        let sub_entries = fs::read_dir(&nested_dir)?;
        for entry in sub_entries {
            let entry = entry?;
            let file_name = entry.file_name();
            let dest_path = path.join(&file_name);

            fs::rename(entry.path(), dest_path)?;
        }
        fs::remove_dir(nested_dir)?;
        log_info!("Rootfs flattened successfully.");
    }
    Ok(())
}

fn generate_start_script(script_path: &Path, install_path: &Path, distro_name: &str) -> io::Result<()> {
    let path_str = install_path.to_string_lossy();
    let is_alpine = distro_name.to_lowercase().contains("alpine");
    let shell_cmd = if is_alpine { "/bin/sh" } else { "/bin/bash" };

    let current_exe = std::env::current_exe().unwrap_or_else(|_| PathBuf::from("/data/local/tmp/autolinux"));
    let exe_str = current_exe.to_string_lossy();

    let content = format!(r##"#!/bin/sh
DISTROPATH="{}"
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

[ ! -d "$DISTROPATH/dev" ] && mkdir -p "$DISTROPATH/dev"
[ ! -d "$DISTROPATH/dev/pts" ] && mkdir -p "$DISTROPATH/dev/pts"
[ ! -d "$DISTROPATH/dev/shm" ] && mkdir -p "$DISTROPATH/dev/shm"
[ ! -d "$DISTROPATH/proc" ] && mkdir -p "$DISTROPATH/proc"
[ ! -d "$DISTROPATH/sys" ] && mkdir -p "$DISTROPATH/sys"
[ ! -d "$DISTROPATH/sdcard" ] && mkdir -p "$DISTROPATH/sdcard"

mnt --bind /dev "$DISTROPATH/dev"
mnt --bind /sys "$DISTROPATH/sys"
mnt --bind /proc "$DISTROPATH/proc"
mnt -t devpts devpts "$DISTROPATH/dev/pts"
mnt -t tmpfs -o size=256M tmpfs "$DISTROPATH/dev/shm"
mnt --bind /sdcard "$DISTROPATH/sdcard"

if [ -d "$DISTROPATH/etc/pam.d" ]; then
    echo "#%PAM-1.0
auth       sufficient   pam_rootok.so
auth       required     pam_permit.so
account    required     pam_permit.so
session    required     pam_env.so
session    optional     pam_xauth.so
session    required     pam_permit.so" > "$DISTROPATH/etc/pam.d/su"

    cp "$DISTROPATH/etc/pam.d/su" "$DISTROPATH/etc/pam.d/su-l"
fi

if [ -f "$DISTROPATH/root/finalize_setup.sh" ]; then
    echo "[!] First time setup detected. Configuring users & groups..."
    chmod +x "$DISTROPATH/root/finalize_setup.sh"

    if [ -x "$(command -v busybox)" ]; then
        busybox chroot "$DISTROPATH" {} /root/finalize_setup.sh
    else
        /system/bin/chroot "$DISTROPATH" {} /root/finalize_setup.sh
    fi

    echo "[*] Performing Host-Side Security Cleanup (Fedora Fix)..."
    "{}" clean-xattr "$DISTROPATH"

    rm "$DISTROPATH/root/finalize_setup.sh"
fi

echo "[*] Entering Chroot as $TARGET_USER..."
echo "Type 'exit' to leave."

if [ -f "$DISTROPATH/usr/bin/su" ]; then
    SU_CMD="/usr/bin/su"
else
    SU_CMD="/bin/su"
fi

if [ -x "$(command -v busybox)" ]; then
    busybox chroot "$DISTROPATH" $SU_CMD - "$TARGET_USER"
else
    /system/bin/chroot "$DISTROPATH" $SU_CMD - "$TARGET_USER"
fi
"##, path_str, shell_cmd, shell_cmd, exe_str);

    fs::write(script_path, content)?;
    use std::os::unix::fs::PermissionsExt;
    let mut perms = fs::metadata(script_path)?.permissions();
    perms.set_mode(0o755);
    fs::set_permissions(script_path, perms)?;

    Ok(())
}

fn generate_internal_setup_script(install_path: &Path, username: &str, password: &str, distro_name: &str) -> io::Result<()> {
    let name_lower = distro_name.to_lowercase();
    let is_alpine = name_lower.contains("alpine");
    let is_arch = name_lower.contains("arch");
    let is_void = name_lower.contains("void");
    let is_fedora = name_lower.contains("fedora");

    let package_logic = if is_alpine {
        r#"
echo ">>> (Alpine) Updating Repository..."
echo "http://dl-cdn.alpinelinux.org/alpine/edge/main" > /etc/apk/repositories
echo "http://dl-cdn.alpinelinux.org/alpine/edge/community" >> /etc/apk/repositories
apk update
echo ">>> (Alpine) Installing Base Tools..."
apk add bash shadow sudo nano net-tools git
"#
    } else if is_arch {
        r#"
echo ">>> (Arch Linux) Configuring Pacman..."
sed -i 's/^DownloadUser/#DownloadUser/' /etc/pacman.conf
sed -i 's/^#DisableSandbox/DisableSandbox/' /etc/pacman.conf
sed -i 's/^CheckSpace/#CheckSpace/' /etc/pacman.conf
userdel -r alarm 2>/dev/null || true
echo ">>> (Arch Linux) Init Keyring..."
pacman-key --init
pacman-key --populate archlinuxarm
echo ">>> (Arch Linux) Updating..."
pacman -Sy --noconfirm
echo ">>> (Arch Linux) Installing Tools..."
pacman -S --noconfirm sudo nano net-tools git base-devel
"#
    } else if is_void {
        r#"
echo ">>> (Void Linux) Updating..."
xbps-install -S
echo ">>> (Void Linux) Installing Tools..."
xbps-install -y -S sudo nano net-tools git bash shadow ca-certificates
"#
    } else if is_fedora {
        r#"
echo ">>> (Fedora) Updating Repository..."
dnf update -y
echo ">>> (Fedora) Installing Tools..."
dnf install -y nano net-tools sudo git passwd shadow-utils util-linux attr findutils
"#
    } else {
        r#"
echo ">>> (Debian/Ubuntu/Kali) Updating..."
apt update -y
echo ">>> (Debian/Ubuntu/Kali) Installing Tools..."
apt install -y nano net-tools sudo git
"#
    };

    let content = format!(r#"#!/bin/sh
export PATH=/usr/local/sbin:/usr/local/bin:/usr/sbin:/usr/bin:/sbin:/bin
unset TMPDIR TMP TEMP
export LC_ALL=C
mkdir -p /tmp
chmod 1777 /tmp

{}

echo ">>> Configuring Sudo Access..."
if [ -f /etc/sudoers ]; then
    echo '%wheel ALL=(ALL:ALL) ALL' >> /etc/sudoers
fi

echo ">>> Configuring Network Groups..."
sed -i '/:3003:/d' /etc/group
sed -i '/:3004:/d' /etc/group
sed -i '/:1003:/d' /etc/group
sed -i '/^aid_inet:/d' /etc/group
sed -i '/^aid_net_raw:/d' /etc/group
sed -i '/^aid_graphics:/d' /etc/group

groupadd -g 3003 aid_inet
groupadd -g 3004 aid_net_raw
groupadd -g 1003 aid_graphics

if [ -f /etc/debian_version ]; then
    usermod -g 3003 -G 3003,3004 -a _apt 2>/dev/null || true
fi
usermod -a -G aid_inet root 2>/dev/null || usermod -G 3003 -a root

echo ">>> Creating User '{1}'..."
groupadd storage 2>/dev/null || true
groupadd wheel 2>/dev/null || true

useradd -m -g users -G wheel,audio,video,storage,aid_inet -s /bin/bash {1}
echo "{1}:{2}" | chpasswd

echo ">>> Done!"
"#, package_logic, username, password);

    let setup_path = install_path.join("root/finalize_setup.sh");
    fs::write(setup_path, content)?;
    Ok(())
}