package main

import (
	"fmt"
	"os"
	"os/exec"

	"github.com/HanSoBored/Auto-Linux/internal/core"
	"github.com/HanSoBored/Auto-Linux/internal/ui"
)

func main() {
	if len(os.Args) >= 3 && os.Args[1] == "clean-xattr" {
		targetPath := os.Args[2]
		fmt.Printf(">>> (Auto-Linux Internal) Stripping xattrs from: %s\n", targetPath)
		core.CleanSecurityXattrsRecursive(targetPath)
		fmt.Println(">>> (Auto-Linux Internal) Cleanup complete.")
		return
	}

	device := core.NewDeviceInfo()
	if !device.IsRoot {
		if device.CanSU {
			fmt.Println("Not root. Attempting self-elevation...")
			tryElevatePrivileges()
		} else {
			fmt.Println("Root access is required, but 'su' binary not found.")
			os.Exit(1)
		}
	}

	ui.Run()
}

func tryElevatePrivileges() {
	exe, _ := os.Executable()
	cmd := exec.Command("su", "-c", exe)
	cmd.Stdin = os.Stdin
	cmd.Stdout = os.Stdout
	cmd.Stderr = os.Stderr
	_ = cmd.Run()
	os.Exit(0)
}
