package core

import (
	"bufio"
	"fmt"
	"os"
	"path/filepath"
	"strconv"
	"strings"

	"github.com/HanSoBored/Auto-Linux/internal/types"
)

func GetAllFamilies(arch string) []types.DistroFamily {
	return []types.DistroFamily{
		getUbuntuFamily(arch),
		getDebianFamily(arch),
		getAlpineFamily(arch),
		getArchFamily(arch),
		getKaliFamily(arch),
		getFedoraFamily(arch),
		getVoidFamily(arch),
	}
}

func getUbuntuFamily(arch string) types.DistroFamily {
	debArch := "armhf"
	switch arch {
	case "arm64", "aarch64":
		debArch = "arm64"
	case "amd64", "x86_64":
		debArch = "amd64"
	}

	return types.DistroFamily{
		Name:        "Ubuntu",
		Description: "Popular, User-Friendly, Debian-Based.",
		Variants: []types.Distro{
			{Name: "Ubuntu 20.04 LTS (Focal Fossa)", Codename: "focal", Version: "20.04.5", URL: fmt.Sprintf("https://cdimage.ubuntu.com/ubuntu-base/releases/20.04/release/ubuntu-base-20.04.5-base-%s.tar.gz", debArch)},
			{Name: "Ubuntu 22.04 LTS (Jammy Jellyfish)", Codename: "jammy", Version: "22.04.5", URL: fmt.Sprintf("https://cdimage.ubuntu.com/ubuntu-base/releases/22.04/release/ubuntu-base-22.04.5-base-%s.tar.gz", debArch)},
			{Name: "Ubuntu 24.04 LTS (Noble Numbat)", Codename: "noble", Version: "24.04.3", URL: fmt.Sprintf("https://cdimage.ubuntu.com/ubuntu-base/releases/24.04/release/ubuntu-base-24.04.3-base-%s.tar.gz", debArch)},
			{Name: "Ubuntu 26.04 LTS (Resolute Raccoon)", Codename: "resolute", Version: "26.04", URL: fmt.Sprintf("https://cdimage.ubuntu.com/ubuntu-base/releases/26.04/snapshot1/ubuntu-base-26.04-base-%s.tar.gz", debArch)},
		},
	}
}

func getDebianFamily(arch string) types.DistroFamily {
	if arch == "arm64" || arch == "aarch64" {
		return types.DistroFamily{
			Name:        "Debian",
			Description: "Stable, Reliable, Widely-Used Server Distro.",
			Variants: []types.Distro{
				{Name: "Debian 11 (Bullseye)", Codename: "bullseye", Version: "11", URL: "https://github.com/HSB-Tools/Debootstrap-Linux/releases/download/debian/debian-bullseye-aarch64.tar.gz"},
				{Name: "Debian 12 (Bookworm)", Codename: "bookworm", Version: "12", URL: "https://github.com/HSB-Tools/Debootstrap-Linux/releases/download/debian/debian-bookworm-aarch64.tar.gz"},
				{Name: "Debian 13 (Trixie)", Codename: "trixie", Version: "13", URL: "https://github.com/HSB-Tools/Debootstrap-Linux/releases/download/debian/debian-trixie-aarch64.tar.gz"},
			},
		}
	}
	return types.DistroFamily{Name: "Debian", Description: "AArch64 Only for now", Variants: []types.Distro{}}
}

func getAlpineFamily(arch string) types.DistroFamily {
	alpArch := "armv7"
	switch arch {
	case "arm64", "aarch64":
		alpArch = "aarch64"
	case "amd64", "x86_64":
		alpArch = "x86_64"
	}

	return types.DistroFamily{
		Name:        "Alpine Linux",
		Description: "Security-oriented, Lightweight (musl libc & busybox).",
		Variants: []types.Distro{
			{Name: "Alpine 3.20", Codename: "release", Version: "3.20.8", URL: fmt.Sprintf("https://dl-cdn.alpinelinux.org/alpine/v3.20/releases/%s/alpine-minirootfs-3.20.8-%s.tar.gz", alpArch, alpArch)},
			{Name: "Alpine 3.21", Codename: "release", Version: "3.21.5", URL: fmt.Sprintf("https://dl-cdn.alpinelinux.org/alpine/v3.21/releases/%s/alpine-minirootfs-3.21.5-%s.tar.gz", alpArch, alpArch)},
			{Name: "Alpine 3.22", Codename: "release", Version: "3.22.2", URL: fmt.Sprintf("https://dl-cdn.alpinelinux.org/alpine/v3.22/releases/%s/alpine-minirootfs-3.22.2-%s.tar.gz", alpArch, alpArch)},
			{Name: "Alpine 3.23", Codename: "release", Version: "3.23.0", URL: fmt.Sprintf("https://dl-cdn.alpinelinux.org/alpine/v3.23/releases/%s/alpine-minirootfs-3.23.0-%s.tar.gz", alpArch, alpArch)},
			{Name: "Alpine Edge", Codename: "edge", Version: "rolling", URL: fmt.Sprintf("https://dl-cdn.alpinelinux.org/alpine/edge/releases/%s/alpine-minirootfs-20251016-%s.tar.gz", alpArch, alpArch)},
		},
	}
}

func getArchFamily(arch string) types.DistroFamily {
	if arch == "arm64" || arch == "aarch64" {
		return types.DistroFamily{
			Name:        "Arch Linux",
			Description: "Rolling release, Lightweight, Pacman Package Manager.",
			Variants: []types.Distro{
				{Name: "Arch Linux ARM (Generic)", Codename: "rolling", Version: "latest", URL: "http://os.archlinuxarm.org/os/ArchLinuxARM-aarch64-latest.tar.gz"},
			},
		}
	}
	return types.DistroFamily{Name: "Arch Linux", Description: "AArch64 Only for now", Variants: []types.Distro{}}
}

func getKaliFamily(arch string) types.DistroFamily {
	debArch := "armhf"
	switch arch {
	case "arm64", "aarch64":
		debArch = "arm64"
	case "amd64", "x86_64":
		debArch = "amd64"
	}

	return types.DistroFamily{
		Name:        "Kali Linux",
		Description: "Security-Focused Distro for Penetration Testing.",
		Variants: []types.Distro{
			{Name: "Kali Linux 2025.1", Codename: "kali-rolling", Version: "2025.1c", URL: fmt.Sprintf("https://kali.download/nethunter-images/kali-2025.1c/rootfs/kali-nethunter-rootfs-nano-%s.tar.xz", debArch)},
			{Name: "Kali Linux 2025.2", Codename: "kali-rolling", Version: "2025.2", URL: fmt.Sprintf("https://kali.download/nethunter-images/kali-2025.2/rootfs/kali-nethunter-rootfs-nano-%s.tar.xz", debArch)},
			{Name: "Kali Linux 2025.3", Codename: "kali-rolling", Version: "2025.3", URL: fmt.Sprintf("https://kali.download/nethunter-images/kali-2025.3/rootfs/kali-nethunter-rootfs-nano-%s.tar.xz", debArch)},
			{Name: "Kali Linux (Current)", Codename: "kali-rolling", Version: "latest", URL: fmt.Sprintf("https://kali.download/nethunter-images/current/rootfs/kali-nethunter-rootfs-nano-%s.tar.xz", debArch)},
		},
	}
}

func getFedoraFamily(arch string) types.DistroFamily {
	fedArch := "armhfp"
	switch arch {
	case "arm64", "aarch64":
		fedArch = "aarch64"
	case "amd64", "x86_64":
		fedArch = "x86_64"
	}

	return types.DistroFamily{
		Name:        "Fedora",
		Description: "Cutting-Edge, Community-Driven Red Hat Distro.",
		Variants: []types.Distro{
			{Name: "Fedora 40", Codename: "40", Version: "40-1.14", URL: fmt.Sprintf("https://archives.fedoraproject.org/pub/archive/fedora/linux/releases/40/Container/%s/images/Fedora-Container-Base-Generic.%s-40-1.14.oci.tar.xz", fedArch, fedArch)},
			{Name: "Fedora 41", Codename: "41", Version: "41-1.4", URL: fmt.Sprintf("https://mirror.twds.com.tw/fedora/fedora/linux/releases/41/Container/%s/images/Fedora-Container-Base-Generic-41-1.4.%s.oci.tar.xz", fedArch, fedArch)},
			{Name: "Fedora 42 (Adams)", Codename: "adams", Version: "42-1.1", URL: fmt.Sprintf("https://mirror.twds.com.tw/fedora/fedora/linux/releases/42/Container/%s/images/Fedora-Container-Base-Generic-42-1.1.%s.oci.tar.xz", fedArch, fedArch)},
			{Name: "Fedora 43", Codename: "43", Version: "43-1.6", URL: fmt.Sprintf("https://mirror.twds.com.tw/fedora/fedora/linux/releases/43/Container/%s/images/Fedora-Container-Base-Generic-43-1.6.%s.oci.tar.xz", fedArch, fedArch)},
		},
	}
}

func getVoidFamily(arch string) types.DistroFamily {
	voidArch := "armv7l-musl"
	switch arch {
	case "arm64", "aarch64":
		voidArch = "aarch64"
	case "amd64", "x86_64":
		voidArch = "x86_64-musl"
	}

	return types.DistroFamily{
		Name:        "Void Linux",
		Description: "Modern Linux Distro with Rolling Releases and XBPS.",
		Variants: []types.Distro{
			{Name: "Void Linux (20240314)", Codename: "rolling", Version: "20240314", URL: fmt.Sprintf("https://repo-default.voidlinux.org/live/20240314/void-%s-ROOTFS-20240314.tar.xz", voidArch)},
			{Name: "Void Linux (20250202)", Codename: "rolling", Version: "20250202", URL: fmt.Sprintf("https://repo-default.voidlinux.org/live/20250202/void-%s-ROOTFS-20250202.tar.xz", voidArch)},
		},
	}
}

func ScanInstalledDistros() []types.InstalledDistro {
	results := []types.InstalledDistro{}
	basePath := "/data/local/rootfs"
	entries, err := os.ReadDir(basePath)
	if err != nil {
		return results
	}

	for _, entry := range entries {
		if entry.IsDir() {
			folderName := entry.Name()
			scriptPath := filepath.Join(basePath, fmt.Sprintf("start-%s.sh", folderName))
			path := filepath.Join(basePath, folderName)

			if _, err := os.Stat(scriptPath); err == nil {
				if _, err := os.Stat(filepath.Join(path, "etc")); err == nil {
					users := getUsersFromPasswd(filepath.Join(path, "etc", "passwd"))
					results = append(results, types.InstalledDistro{
						Name:       folderName,
						Path:       path,
						ScriptPath: scriptPath,
						Users:      users,
					})
				}
			}
		}
	}
	return results
}

func getUsersFromPasswd(passwdPath string) []string {
	users := []string{"root"}
	file, err := os.Open(passwdPath)
	if err != nil {
		return users
	}
	defer file.Close()

	scanner := bufio.NewScanner(file)
	for scanner.Scan() {
		line := scanner.Text()
		parts := strings.Split(line, ":")
		if len(parts) >= 3 {
			uid, err := strconv.Atoi(parts[2])
			if err == nil {
				if uid >= 1000 && uid < 60000 {
					users = append(users, parts[0])
				}
			}
		}
	}
	return users
}
