package core

import (
	"archive/tar"
	"compress/gzip"
	"fmt"
	"io"
	"net/http"
	"os"
	"path/filepath"
	"strings"

	"github.com/HanSoBored/Auto-Linux/internal/types"
	"github.com/ulikunitz/xz"
	"golang.org/x/sys/unix"
)

type InstallState struct {
	Status   string
	Progress float32
	Done     bool
	Error    error
	Result   string
}

func InstallDistro(distro types.Distro, username, password string, progress chan<- InstallState) {
	folderName := fmt.Sprintf("%s-%s-%s",
		strings.ToLower(strings.Split(distro.Name, " ")[0]),
		distro.Codename,
		distro.Version,
	)

	baseDir := "/data/local/rootfs"
	installPath := filepath.Join(baseDir, folderName)
	startScriptPath := filepath.Join(baseDir, fmt.Sprintf("start-%s.sh", folderName))

	progress <- InstallState{Status: "Initializing..."}

	if _, err := os.Stat(installPath); err == nil {
		os.RemoveAll(installPath)
	}
	if err := os.MkdirAll(installPath, 0755); err != nil {
		progress <- InstallState{Error: fmt.Errorf("failed to create install dir: %w", err)}
		return
	}

	// Download
	isXZ := strings.HasSuffix(distro.URL, ".xz")
	filename := "rootfs.tar.gz"
	if isXZ {
		filename = "rootfs.tar.xz"
	}
	tarPath := filepath.Join(installPath, filename)

	progress <- InstallState{Status: "Downloading Rootfs...", Progress: 0}

	resp, err := http.Get(distro.URL)
	if err != nil {
		progress <- InstallState{Error: fmt.Errorf("download failed: %w", err)}
		return
	}
	defer resp.Body.Close()

	if resp.StatusCode != http.StatusOK {
		progress <- InstallState{Error: fmt.Errorf("HTTP error %d: %s", resp.StatusCode, resp.Status)}
		return
	}

	out, err := os.Create(tarPath)
	if err != nil {
		progress <- InstallState{Error: fmt.Errorf("failed to create tar file: %w", err)}
		return
	}

	contentLength := resp.ContentLength
	var downloaded int64

	buf := make([]byte, 8192)
	for {
		n, err := resp.Body.Read(buf)
		if n > 0 {
			out.Write(buf[:n])
			downloaded += int64(n)
			if contentLength > 0 {
				pct := (float32(downloaded) / float32(contentLength)) * 100.0
				progress <- InstallState{Status: "Downloading Rootfs...", Progress: pct}
			}
		}
		if err == io.EOF {
			break
		}
		if err != nil {
			out.Close()
			progress <- InstallState{Error: fmt.Errorf("download error: %w", err)}
			return
		}
	}
	out.Close()

	// Extract
	progress <- InstallState{Status: "Extracting Archive ...", Progress: 100}
	if err := unpackArchive(tarPath, installPath, isXZ); err != nil {
		progress <- InstallState{Error: fmt.Errorf("extraction failed: %w", err)}
		return
	}
	os.Remove(tarPath)

	// OCI check & flatten
	if _, err := os.Stat(filepath.Join(installPath, "blobs")); err == nil {
		progress <- InstallState{Status: "OCI Image detected. Processing..."}
		if err := handleOCIExtraction(installPath); err != nil {
			progress <- InstallState{Error: fmt.Errorf("OCI extraction failed: %w", err)}
			return
		}
	}

	if err := flattenNestedRootfs(installPath); err != nil {
		// Non-critical
	}

	progress <- InstallState{Status: "Cleaning Security Attributes..."}
	CleanSecurityXattrsRecursive(installPath)

	// Configure
	progress <- InstallState{Status: "Configuring Environment..."}
	os.MkdirAll(filepath.Join(installPath, "sdcard"), 0755)
	os.MkdirAll(filepath.Join(installPath, "dev/shm"), 0755)

	resolvPath := filepath.Join(installPath, "etc/resolv.conf")
	os.MkdirAll(filepath.Dir(resolvPath), 0755)
	os.WriteFile(resolvPath, []byte(`nameserver 8.8.8.8
nameserver 8.8.4.4
`), 0644)

	// Generate scripts
	if err := GenerateStartScript(startScriptPath, installPath, distro.Name); err != nil {
		progress <- InstallState{Error: fmt.Errorf("failed to generate start script: %w", err)}
		return
	}
	if err := GenerateInternalSetupScript(installPath, username, password, distro.Name); err != nil {
		progress <- InstallState{Error: fmt.Errorf("failed to generate setup script: %w", err)}
		return
	}

	progress <- InstallState{Status: "Success!", Done: true, Result: startScriptPath}
}

func unpackArchive(archivePath, dest string, isXZ bool) error {
	f, err := os.Open(archivePath)
	if err != nil {
		return err
	}
	defer f.Close()

	var r io.Reader = f
	if isXZ {
		r, err = xz.NewReader(f)
		if err != nil {
			return err
		}
	} else {
		r, err = gzip.NewReader(f)
		if err != nil {
			return err
		}
	}

	tr := tar.NewReader(r)
	for {
		header, err := tr.Next()
		if err == io.EOF {
			break
		}
		if err != nil {
			return err
		}

		target := filepath.Join(dest, header.Name)
		switch header.Typeflag {
		case tar.TypeDir:
			if err := os.MkdirAll(target, 0755); err != nil {
				return err
			}
		case tar.TypeReg:
			f, err := os.OpenFile(target, os.O_CREATE|os.O_RDWR, os.FileMode(header.Mode))
			if err != nil {
				return err
			}
			if _, err := io.Copy(f, tr); err != nil {
				f.Close()
				return err
			}
			f.Close()
		case tar.TypeSymlink:
			if err := os.Symlink(header.Linkname, target); err != nil {
				// Ignore errors on symlinks if they exist
			}
		case tar.TypeLink:
			targetLink := filepath.Join(dest, header.Linkname)
			if err := os.Link(targetLink, target); err != nil {
				// Ignore
			}
		}
	}
	return nil
}

func handleOCIExtraction(basePath string) error {
	blobsDir := filepath.Join(basePath, "blobs", "sha256")
	entries, err := os.ReadDir(blobsDir)
	if err != nil {
		return err
	}

	var maxSize int64
	var bestBlob string
	for _, entry := range entries {
		info, err := entry.Info()
		if err == nil && info.Size() > maxSize {
			maxSize = info.Size()
			bestBlob = filepath.Join(blobsDir, entry.Name())
		}
	}

	if bestBlob == "" {
		return fmt.Errorf("no layers found")
	}

	tempPath := filepath.Join(basePath, "layer.tar.gz")
	if err := os.Rename(bestBlob, tempPath); err != nil {
		return err
	}

	// Clean other files
	all, _ := os.ReadDir(basePath)
	for _, e := range all {
		p := filepath.Join(basePath, e.Name())
		if p != tempPath {
			os.RemoveAll(p)
		}
	}

	if err := unpackArchive(tempPath, basePath, false); err != nil {
		return err
	}
	os.Remove(tempPath)
	return nil
}

func CleanSecurityXattrsRecursive(path string) {
	filepath.Walk(path, func(p string, info os.FileInfo, err error) error {
		if err != nil {
			return nil
		}
		name := info.Name()
		if info.IsDir() && (name == "proc" || name == "sys" || name == "dev" || name == "sdcard") {
			return filepath.SkipDir
		}

		unix.Lremovexattr(p, "security.ima")
		unix.Lremovexattr(p, "security.selinux")
		unix.Lremovexattr(p, "security.capability")
		return nil
	})
}

func flattenNestedRootfs(path string) error {
	entries, err := os.ReadDir(path)
	if err != nil {
		return err
	}

	if len(entries) == 1 && entries[0].IsDir() {
		nestedDir := filepath.Join(path, entries[0].Name())
		subEntries, _ := os.ReadDir(nestedDir)
		for _, e := range subEntries {
			os.Rename(filepath.Join(nestedDir, e.Name()), filepath.Join(path, e.Name()))
		}
		os.Remove(nestedDir)
	}
	return nil
}

func GenerateStartScript(scriptPath, installPath, distroName string) error {
	isAlpine := strings.Contains(strings.ToLower(distroName), "alpine")
	shellCmd := "/bin/bash"
	if isAlpine {
		shellCmd = "/bin/sh"
	}

	exe, _ := os.Executable()
	if exe == "" {
		exe = "/data/local/rootfs/autolinux"
	}

	content := fmt.Sprintf(`#!/bin/sh
DISTROPATH="%s"
TARGET_USER="${1:-root}"

mnt() {
    if [ -x "$(command -v busybox)" ]; then
        busybox mount "$@"
    else
        /system/bin/mount "$@"
    fi
}

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
    echo "#%%PAM-1.0
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
        busybox chroot "$DISTROPATH" %s /root/finalize_setup.sh
    else
        /system/bin/chroot "$DISTROPATH" %s /root/finalize_setup.sh
    fi

    echo "[*] Performing Host-Side Security Cleanup..."
    "%s" clean-xattr "$DISTROPATH"

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
`, installPath, shellCmd, shellCmd, exe)

	return os.WriteFile(scriptPath, []byte(content), 0755)
}

func GenerateInternalSetupScript(installPath, username, password, distroName string) error {
	nameLower := strings.ToLower(distroName)
	isAlpine := strings.Contains(nameLower, "alpine")
	isArch := strings.Contains(nameLower, "arch")
	isVoid := strings.Contains(nameLower, "void")
	isFedora := strings.Contains(nameLower, "fedora")
	isDebianFamily := !isAlpine && !isArch && !isVoid && !isFedora

	networkGroupsLogic := `
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
`

	packageLogic := ""
	if isAlpine {
		packageLogic = `
echo ">>> (Alpine) Updating Repository..."
echo "http://dl-cdn.alpinelinux.org/alpine/edge/main" > /etc/apk/repositories
echo "http://dl-cdn.alpinelinux.org/alpine/edge/community" >> /etc/apk/repositories
apk update
echo ">>> (Alpine) Installing Base Tools..."
apk add bash shadow sudo nano net-tools git
`
	} else if isArch {
		packageLogic = `
echo ">>> (Arch Linux) Configuring Pacman..."
sed -i 's/^DownloadUser/#DownloadUser/' /etc/pacman.conf
sed -i 's/^#DisableSandbox/DisableSandbox/' /etc/pacman.conf
sed -i 's/^CheckSpace/#CheckSpace/' /etc/pacman.conf
userdel -r alarm 2>/dev/null || true
echo ">>> (Arch Linux) Init Keyring..."
pacman-key --init
pacman-key --populate archlinuxarm
echo ">>> (Arch Linux) Updating..."
pacman -Syyu --noconfirm
echo ">>> (Arch Linux) Installing Tools..."
pacman -S --noconfirm sudo nano net-tools git base-devel
`
	} else if isVoid {
		packageLogic = `
echo ">>> (Void Linux) Updating..."
xbps-install -S
echo ">>> (Void Linux) Installing Tools..."
xbps-install -y -S sudo nano net-tools git bash shadow ca-certificates openssl
`
	} else if isFedora {
		packageLogic = `
echo ">>> (Fedora) Updating Repository..."
dnf update -y
echo ">>> (Fedora) Installing Tools..."
dnf install -y nano net-tools sudo git passwd shadow-utils util-linux attr findutils
`
	} else {
		packageLogic = `
echo ">>> (Debian/Ubuntu/Kali) Updating..."
apt update -y
echo ">>> (Debian/Ubuntu/Kali) Installing Tools..."
apt install -y nano net-tools sudo git
`
	}

	userCreationLogic := fmt.Sprintf(`
echo ">>> Creating User '%s'..."
groupadd storage 2>/dev/null || true
groupadd wheel 2>/dev/null || true
useradd -m -g users -G wheel,audio,video,storage,aid_inet -s /bin/bash %s
echo "%s:%s" | chpasswd
`, username, username, username, password)

	if isVoid {
		userCreationLogic = fmt.Sprintf(`
echo ">>> Creating User '%s'..."
groupadd storage 2>/dev/null || true
groupadd wheel 2>/dev/null || true
useradd -m -g users -G wheel,audio,video,storage,aid_inet -s /bin/bash %s
PASS_HASH=$(echo "%s" | openssl passwd -6 -stdin)
echo "%s:$PASS_HASH" | chpasswd -e
`, username, username, password, username)
	}

	orderedLogic := networkGroupsLogic + packageLogic
	if !isDebianFamily {
		orderedLogic = packageLogic + networkGroupsLogic
	}

	content := fmt.Sprintf(`#!/bin/sh
export PATH=/usr/local/sbin:/usr/local/bin:/usr/sbin:/usr/bin:/sbin:/bin
unset TMPDIR TMP TEMP
export LC_ALL=C
mkdir -p /tmp
chmod 1777 /tmp

%s

echo ">>> Configuring Sudo Access..."
if [ -f /etc/sudoers ]; then
    echo '%%wheel ALL=(ALL:ALL) ALL' >> /etc/sudoers
fi

%s

echo ">>> Done!"
`, orderedLogic, userCreationLogic)

	setupPath := filepath.Join(installPath, "root/finalize_setup.sh")
	os.MkdirAll(filepath.Dir(setupPath), 0755)
	return os.WriteFile(setupPath, []byte(content), 0755)
}
