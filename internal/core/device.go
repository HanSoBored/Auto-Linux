package core

import (
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

func checkSUAccess() bool {
	_, err := exec.LookPath("su")
	if err == nil {
		return true
	}
	knownPaths := []string{
		"/sbin/su",
		"/system/bin/su",
		"/system/xbin/su",
		"/data/adb/ksu/bin/su",
		"/data/adb/apatch/bin/su",
	}
	for _, path := range knownPaths {
		if _, err := exec.Command(path, "-v").Output(); err == nil {
			return true
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
