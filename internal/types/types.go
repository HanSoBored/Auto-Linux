package types

import "path/filepath"

type CurrentScreen int

const (
	ScreenDashboard CurrentScreen = iota
	ScreenDistroFamilySelect
	ScreenDistroVersionSelect
	ScreenUserCredentials
	ScreenInstalling
	ScreenFinished
	ScreenLaunchSelect
)

type InputMode int

const (
	InputUsername InputMode = iota
	InputPassword
)

type Distro struct {
	Name     string
	Codename string
	Version  string
	URL      string
}

type DistroFamily struct {
	Name        string
	Description string
	Variants    []Distro
}

type InstalledDistro struct {
	Name       string
	Path       string
	ScriptPath string
	Users      []string
}

func (d *InstalledDistro) BasePath() string {
	return filepath.Dir(d.Path)
}
