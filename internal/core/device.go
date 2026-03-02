package core

import (
	"os"
	"os/exec"
	"runtime"
	"strings"
)

type DeviceInfo struct {
	Arch       string
	IsRoot     bool
	CanSU      bool
	RootType   string
	AndroidVer string
}

func NewDeviceInfo() DeviceInfo {
	return DeviceInfo{
		Arch:       runtime.GOARCH,
		IsRoot:     checkCurrentUserRoot(),
		CanSU:      checkSUAccess(),
		RootType:   getSUVariant(),
		AndroidVer: getAndroidVersion(),
	}
}

func checkCurrentUserRoot() bool {
	output, err := exec.Command("id", "-u").Output()
	if err != nil {
		return false
	}
	return strings.TrimSpace(string(output)) == "0"
}

// fileExists checks if a file exists and is executable
func fileExists(path string) bool {
	info, err := os.Stat(path)
	if err != nil {
		return false
	}
	// Check if file is executable (at least for owner)
	mode := info.Mode()
	return mode.IsRegular() && (mode&0o111 != 0)
}

func checkSUAccess() bool {
	// First try PATH lookup using manual search instead of exec.LookPath
	pathDirs := []string{"/system/bin", "/system/xbin", "/sbin", "/data/adb/ksu/bin", "/data/adb/apatch/bin"}
	for _, dir := range pathDirs {
		suPath := dir + "/su"
		if fileExists(suPath) {
			return true
		}
	}
	
	// Also check current PATH
	if path := os.Getenv("PATH"); path != "" {
		for _, dir := range strings.Split(path, ":") {
			suPath := dir + "/su"
			if fileExists(suPath) {
				return true
			}
		}
	}
	
	return false
}

func getSUVariant() string {
	output, err := exec.Command("su", "-v").Output()
	if err != nil {
		return "None"
	}
	versionStr := strings.ToLower(string(output))
	if strings.Contains(versionStr, "magisk") {
		return "Magisk"
	} else if strings.Contains(versionStr, "kernelsu") {
		return "KernelSU"
	} else if strings.Contains(versionStr, "apatch") {
		return "APatch"
	}
	return "Generic"
}

func getAndroidVersion() string {
	output, err := exec.Command("getprop", "ro.build.version.release").Output()
	if err != nil {
		return "Unknown"
	}
	return strings.TrimSpace(string(output))
}
