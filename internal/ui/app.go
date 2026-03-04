package ui

import (
	"fmt"
	"os"
	"os/exec"
	"strings"

	"charm.land/bubbles/v2/list"
	"charm.land/bubbles/v2/spinner"
	"charm.land/bubbles/v2/textinput"
	tea "charm.land/bubbletea/v2"
	"charm.land/lipgloss/v2"
	"github.com/HanSoBored/Auto-Linux/internal/core"
	"github.com/HanSoBored/Auto-Linux/internal/types"
)

// Define colors
var (
	purple = lipgloss.Color("#7D56F4")
	white  = lipgloss.Color("#FAFAFA")
	gray   = lipgloss.Color("#777777")
	green  = lipgloss.Color("#04B575")
	red    = lipgloss.Color("#FF4C4C")
	blue   = lipgloss.Color("#00B2FF")
	darkBG = lipgloss.Color("#1A1A2E")
)

// Define styles
var (
	titleStyle = lipgloss.NewStyle().
			Bold(true).
			Foreground(white).
			Background(purple).
			Padding(0, 1).
			MarginBottom(1)

	deviceStyle = lipgloss.NewStyle().
			Foreground(lipgloss.Color("#B8B8D1")).
			Italic(true).
			MarginBottom(1)

	itemStyle = lipgloss.NewStyle().
			PaddingLeft(2).
			Foreground(lipgloss.Color("#E0E0E0"))

	selectedItemStyle = lipgloss.NewStyle().
				PaddingLeft(0).
				Foreground(purple).
				Bold(true).
				Underline(true)

	helpStyle = lipgloss.NewStyle().
			Foreground(lipgloss.Color("#6A6A8A")).
			MarginTop(1)

	statusStyle = lipgloss.NewStyle().
			Bold(true).
			Foreground(green)

	errorStyle = lipgloss.NewStyle().
			Bold(true).
			Foreground(red)

	containerStyle = lipgloss.NewStyle().
			Padding(1, 2)

	// New Styles for UI/UX improvement
	borderStyle = lipgloss.NewStyle().
			BorderStyle(lipgloss.RoundedBorder()).
			BorderForeground(purple).
			Padding(1, 2).
			Background(darkBG)

	headerStyle = lipgloss.NewStyle().
			Foreground(purple).
			Bold(true).
			Underline(true).
			MarginBottom(1)
)

type installProgressMsg core.InstallState
type launchExitedMsg struct{}

type familyItem struct {
	family types.DistroFamily
}

func (i familyItem) Title() string       { return i.family.Name }
func (i familyItem) Description() string { return i.family.Description }
func (i familyItem) FilterValue() string { return i.family.Name }

type variantItem struct {
	variant types.Distro
}

func (i variantItem) Title() string       { return i.variant.Name }
func (i variantItem) Description() string { return i.variant.URL }
func (i variantItem) FilterValue() string { return i.variant.Name }

type model struct {
	device           core.DeviceInfo
	screen           types.CurrentScreen
	distroFamilies   []types.DistroFamily
	installedDistros []types.InstalledDistro

	// Bubbles components
	familyList   list.Model
	variantList  list.Model
	spinner      spinner.Model
	usernameInput textinput.Model
	passwordInput textinput.Model

	selectedFamilyIdx    int
	selectedVersionIdx   int
	selectedInstalledIdx int
	selectedUserIdx      int

	inputMode types.InputMode

	installStatus   string
	installProgress float32
	installErr      error
	installDone     bool
	installResult   string

	installChan  chan core.InstallState
	isInstalling bool

	quitting bool
	width    int
	height   int
}

func NewModel() model {
	device := core.NewDeviceInfo()
	families := core.GetAllFamilies(device.Arch)
	installed := core.ScanInstalledDistros()

	// Setup Family List
	familyItems := make([]list.Item, len(families))
	for i, f := range families {
		familyItems[i] = familyItem{family: f}
	}
	fl := list.New(familyItems, list.NewDefaultDelegate(), 0, 0)
	fl.Title = "Distribution Families"
	fl.SetShowStatusBar(false)
	fl.SetFilteringEnabled(true)
	fl.Styles.Title = titleStyle
	// Enhanced list styles for v2
	fl.Styles.PaginationStyle = selectedItemStyle
	fl.Styles.HelpStyle = helpStyle
	fl.Styles.ActivePaginationDot = lipgloss.NewStyle().Foreground(purple)
	fl.Styles.InactivePaginationDot = lipgloss.NewStyle().Foreground(gray)

	// Setup Spinner
	s := spinner.New()
	s.Spinner = spinner.Dot
	s.Style = lipgloss.NewStyle().Foreground(purple)

	ui := textinput.New()
	ui.Placeholder = "Username"
	ui.Focus()
	ui.CharLimit = 32
	ui.SetWidth(20)

	pi := textinput.New()
	pi.Placeholder = "Password"
	pi.EchoMode = textinput.EchoPassword
	pi.EchoCharacter = '•'
	pi.CharLimit = 32
	pi.SetWidth(20)

	return model{
		device:           device,
		screen:           types.ScreenDashboard,
		distroFamilies:   families,
		installedDistros: installed,
		familyList:       fl,
		spinner:          s,
		usernameInput:    ui,
		passwordInput:    pi,
		installChan:      make(chan core.InstallState),
	}
}

func (m model) Init() tea.Cmd {
	return m.spinner.Tick
}

func (m model) startInstall() tea.Cmd {
	return func() tea.Msg {
		distro := m.distroFamilies[m.selectedFamilyIdx].Variants[m.selectedVersionIdx]
		go func() {
			core.InstallDistro(distro, m.usernameInput.Value(), m.passwordInput.Value(), m.installChan)
		}()
		return m.waitForProgress()()
	}
}

func (m model) waitForProgress() tea.Cmd {
	return func() tea.Msg {
		state, ok := <-m.installChan
		if !ok {
			return nil
		}
		return installProgressMsg(state)
	}
}

func (m model) Update(msg tea.Msg) (tea.Model, tea.Cmd) {
	var cmd tea.Cmd

	switch msg := msg.(type) {
	case tea.WindowSizeMsg:
		m.width = msg.Width
		m.height = msg.Height

		h, v := containerStyle.GetFrameSize()
		m.familyList.SetSize(msg.Width-h-4, msg.Height-v-4)
		if m.variantList.Width() > 0 {
			m.variantList.SetSize(msg.Width-h-4, msg.Height-v-4)
		}

	case tea.MouseClickMsg:
		// Handle mouse clicks for navigation
		switch m.screen {
		case types.ScreenDashboard:
			if msg.Button == tea.MouseWheelUp && len(m.installedDistros) > 0 {
				m.selectedInstalledIdx = (m.selectedInstalledIdx - 1 + len(m.installedDistros)) % len(m.installedDistros)
			} else if msg.Button == tea.MouseWheelDown && len(m.installedDistros) > 0 {
				m.selectedInstalledIdx = (m.selectedInstalledIdx + 1) % len(m.installedDistros)
			}
		case types.ScreenDistroFamilySelect:
			if msg.Button == tea.MouseWheelUp {
				m.familyList.CursorUp()
			} else if msg.Button == tea.MouseWheelDown {
				m.familyList.CursorDown()
			}
		case types.ScreenDistroVersionSelect:
			if msg.Button == tea.MouseWheelUp {
				m.variantList.CursorUp()
			} else if msg.Button == tea.MouseWheelDown {
				m.variantList.CursorDown()
			}
		case types.ScreenLaunchSelect:
			distro := m.installedDistros[m.selectedInstalledIdx]
			if msg.Button == tea.MouseWheelUp && len(distro.Users) > 0 {
				m.selectedUserIdx = (m.selectedUserIdx - 1 + len(distro.Users)) % len(distro.Users)
			} else if msg.Button == tea.MouseWheelDown && len(distro.Users) > 0 {
				m.selectedUserIdx = (m.selectedUserIdx + 1) % len(distro.Users)
			}
		}

	case tea.KeyPressMsg:
		switch msg.String() {
		case "ctrl+c":
			m.quitting = true
			return m, tea.Quit
		case "q":
			// 'q' only quits on Dashboard, Finished, and LaunchSelect screens
			if m.screen == types.ScreenDashboard || m.screen == types.ScreenFinished || m.screen == types.ScreenLaunchSelect {
				m.quitting = true
				return m, tea.Quit
			}
		}

		switch m.screen {
		case types.ScreenDashboard:
			return m.updateDashboard(msg)
		case types.ScreenDistroFamilySelect:
			return m.updateFamilySelect(msg)
		case types.ScreenDistroVersionSelect:
			return m.updateVersionSelect(msg)
		case types.ScreenUserCredentials:
			return m.updateUserCredentials(msg)
		case types.ScreenLaunchSelect:
			return m.updateLaunchSelect(msg)
		case types.ScreenFinished:
			if msg.String() == "enter" {
				m.installedDistros = core.ScanInstalledDistros()
				m.screen = types.ScreenDashboard
			}
		}

	case installProgressMsg:
		m.installStatus = msg.Status
		m.installProgress = msg.Progress
		if msg.Error != nil {
			m.installErr = msg.Error
			m.screen = types.ScreenFinished
			m.isInstalling = false
			return m, nil
		}
		if msg.Done {
			m.installDone = true
			m.installResult = msg.Result
			m.screen = types.ScreenFinished
			m.isInstalling = false
			return m, nil
		}
		return m, m.waitForProgress()

	case spinner.TickMsg:
		m.spinner, cmd = m.spinner.Update(msg)
		return m, cmd

	case launchExitedMsg:
		m.installedDistros = core.ScanInstalledDistros()
		m.screen = types.ScreenDashboard
		return m, nil
	}

	return m, nil
}

func (m model) updateDashboard(msg tea.KeyPressMsg) (tea.Model, tea.Cmd) {
	switch msg.String() {
	case "i":
		m.screen = types.ScreenDistroFamilySelect
		m.selectedFamilyIdx = 0
	case "up", "k":
		if len(m.installedDistros) > 0 {
			m.selectedInstalledIdx = (m.selectedInstalledIdx - 1 + len(m.installedDistros)) % len(m.installedDistros)
		}
	case "down", "j":
		if len(m.installedDistros) > 0 {
			m.selectedInstalledIdx = (m.selectedInstalledIdx + 1) % len(m.installedDistros)
		}
	case "enter":
		if len(m.installedDistros) > 0 {
			m.selectedUserIdx = 0
			m.screen = types.ScreenLaunchSelect
		}
	}
	return m, nil
}

func (m model) updateFamilySelect(msg tea.KeyPressMsg) (tea.Model, tea.Cmd) {
	// Handle 'esc' BEFORE passing to list to prevent list's default behavior
	if msg.String() == "esc" && !m.familyList.SettingFilter() {
		m.screen = types.ScreenDashboard
		return m, nil
	}

	var cmd tea.Cmd
	m.familyList, cmd = m.familyList.Update(msg)

	if msg.String() == "enter" && !m.familyList.SettingFilter() {
		i, ok := m.familyList.SelectedItem().(familyItem)
		if ok {
			// Find index in original slice
			for idx, f := range m.distroFamilies {
				if f.Name == i.family.Name {
					m.selectedFamilyIdx = idx
					break
				}
			}

			// Prepare variants list with enhanced styles
			variants := m.distroFamilies[m.selectedFamilyIdx].Variants
			variantItems := make([]list.Item, len(variants))
			for i, v := range variants {
				variantItems[i] = variantItem{variant: v}
			}

			m.variantList = list.New(variantItems, list.NewDefaultDelegate(), m.familyList.Width(), m.familyList.Height())
			m.variantList.Title = fmt.Sprintf("Select %s Version", i.family.Name)
			m.variantList.SetShowStatusBar(false)
			m.variantList.Styles.Title = titleStyle
			// Enhanced list styles for v2
			m.variantList.Styles.PaginationStyle = selectedItemStyle
			m.variantList.Styles.HelpStyle = helpStyle
			m.variantList.Styles.ActivePaginationDot = lipgloss.NewStyle().Foreground(purple)
			m.variantList.Styles.InactivePaginationDot = lipgloss.NewStyle().Foreground(gray)

			m.screen = types.ScreenDistroVersionSelect
		}
	}

	return m, cmd
}

func (m model) updateVersionSelect(msg tea.KeyPressMsg) (tea.Model, tea.Cmd) {
	// Handle 'esc' BEFORE passing to list to prevent list's default behavior
	if msg.String() == "esc" && !m.variantList.SettingFilter() {
		m.screen = types.ScreenDistroFamilySelect
		return m, nil
	}

	var cmd tea.Cmd
	m.variantList, cmd = m.variantList.Update(msg)

	if msg.String() == "enter" && !m.variantList.SettingFilter() {
		i, ok := m.variantList.SelectedItem().(variantItem)
		if ok {
			family := m.distroFamilies[m.selectedFamilyIdx]
			for idx, v := range family.Variants {
				if v.Name == i.variant.Name {
					m.selectedVersionIdx = idx
					break
				}
			}
			m.inputMode = types.InputUsername
			m.usernameInput.Focus()
			m.screen = types.ScreenUserCredentials
		}
	}

	return m, cmd
}

func (m model) updateUserCredentials(msg tea.KeyPressMsg) (tea.Model, tea.Cmd) {
	var cmd tea.Cmd
	if m.inputMode == types.InputUsername {
		m.usernameInput, cmd = m.usernameInput.Update(msg)
		if msg.String() == "enter" && m.usernameInput.Value() != "" {
			m.inputMode = types.InputPassword
			m.passwordInput.Focus()
		} else if msg.String() == "esc" {
			m.screen = types.ScreenDistroVersionSelect
		}
	} else {
		m.passwordInput, cmd = m.passwordInput.Update(msg)
		if msg.String() == "enter" && m.passwordInput.Value() != "" {
			if !m.isInstalling {
				m.screen = types.ScreenInstalling
				m.isInstalling = true
				return m, m.startInstall()
			}
		} else if msg.String() == "esc" {
			m.inputMode = types.InputUsername
			m.usernameInput.Focus()
		}
	}
	return m, cmd
}

func (m model) updateLaunchSelect(msg tea.KeyPressMsg) (tea.Model, tea.Cmd) {
	distro := m.installedDistros[m.selectedInstalledIdx]
	switch msg.String() {
	case "esc":
		m.screen = types.ScreenDashboard
	case "up", "k":
		m.selectedUserIdx = (m.selectedUserIdx - 1 + len(distro.Users)) % len(distro.Users)
	case "down", "j":
		m.selectedUserIdx = (m.selectedUserIdx + 1) % len(distro.Users)
	case "enter":
		user := distro.Users[m.selectedUserIdx]
		script := distro.ScriptPath
		return m, tea.ExecProcess(exec.Command("sh", script, user), func(err error) tea.Msg {
			return launchExitedMsg{}
		})
	}
	return m, nil
}

func (m model) View() tea.View {
	var v tea.View
	v.AltScreen = true
	v.MouseMode = tea.MouseModeCellMotion
	v.WindowTitle = "Auto-Linux - Android Linux Installer"
	
	// Better Color Management (v2 feature)
	v.BackgroundColor = darkBG
	v.ForegroundColor = white

	if m.quitting {
		return v
	}

	if m.screen == types.ScreenDistroFamilySelect {
		v.Content = containerStyle.Render(m.familyList.View())
		v.WindowTitle = "Select Distribution Family - Auto-Linux"
		return v
	}
	if m.screen == types.ScreenDistroVersionSelect {
		v.Content = containerStyle.Render(m.variantList.View())
		v.WindowTitle = "Select Version - Auto-Linux"
		return v
	}

	var header string
	title := titleStyle.Render(" AUTO-LINUX ")
	deviceInfo := deviceStyle.Render(fmt.Sprintf(" %s | %s | Root: %v ",
		m.device.Arch, m.device.AndroidVer, m.device.IsRoot))

	header = lipgloss.JoinHorizontal(lipgloss.Center, title, deviceInfo)

	var content string
	var help string

	switch m.screen {
	case types.ScreenDashboard:
		content = m.dashboardView()
		help = "[i] install new • [enter] launch • [q] quit"
	case types.ScreenUserCredentials:
		content = m.userCredentialsView()
		help = "[enter] next/start • [esc] back"
	case types.ScreenInstalling:
		content = m.installingView()
		help = "installing... please wait"
	case types.ScreenFinished:
		content = m.finishedView()
		help = "[enter] return to dashboard"
	case types.ScreenLaunchSelect:
		content = m.launchSelectView()
		help = "[↑/↓] select user • [enter] launch • [esc] back"
	}

	mainBox := borderStyle.Width(m.width - 6).Render(content)

	fullView := lipgloss.JoinVertical(lipgloss.Left,
		header,
		mainBox,
		helpStyle.Render("  " + help),
	)

	v.Content = containerStyle.Render(fullView)

	// Progress Bar for installing screen (v2 native progress bar)
	if m.screen == types.ScreenInstalling && m.installProgress > 0 {
		v.ProgressBar = tea.NewProgressBar(tea.ProgressBarDefault, int(m.installProgress))
	}

	return v
}

func (m model) dashboardView() string {
	var s strings.Builder
	s.WriteString(headerStyle.Render("Installed Distributions") + "\n\n")
	if len(m.installedDistros) == 0 {
		s.WriteString(itemStyle.Foreground(gray).Render("(None found)"))
		s.WriteString("\n")
	} else {
		for i, distro := range m.installedDistros {
			if i == m.selectedInstalledIdx {
				s.WriteString(selectedItemStyle.Render(" ➜ " + distro.Name))
			} else {
				s.WriteString(itemStyle.Render(distro.Name))
			}
			s.WriteString("\n")
		}
	}
	return s.String()
}

func (m model) userCredentialsView() string {
	var s strings.Builder
	s.WriteString(headerStyle.Render("User Configuration") + "\n\n")

	usernameLabel := "  Username: "
	passwordLabel := "  Password: "

	if m.inputMode == types.InputUsername {
		s.WriteString(lipgloss.NewStyle().Foreground(purple).Render(" ➜ Username: "))
		s.WriteString(m.usernameInput.View())
		s.WriteString("\n" + passwordLabel + lipgloss.NewStyle().Foreground(gray).Render("(wait)"))
	} else {
		s.WriteString(usernameLabel + m.usernameInput.Value() + "\n")
		s.WriteString(lipgloss.NewStyle().Foreground(purple).Render(" ➜ Password: "))
		s.WriteString(m.passwordInput.View())
	}
	s.WriteString("\n")
	return s.String()
}

func (m model) installingView() string {
	var s strings.Builder
	s.WriteString(headerStyle.Render("Installation in Progress") + "\n\n")
	
	s.WriteString(fmt.Sprintf("%s %s\n\n", m.spinner.View(), statusStyle.Render(m.installStatus)))

	barWidth := m.width - 20
	if barWidth > 60 {
		barWidth = 60
	}
	prog := int(float32(barWidth) * (m.installProgress / 100.0))
	
	filled := lipgloss.NewStyle().Foreground(purple).Render(strings.Repeat("█", prog))
	empty := lipgloss.NewStyle().Foreground(lipgloss.Color("#333333")).Render(strings.Repeat("░", barWidth-prog))
	
	s.WriteString(fmt.Sprintf(" [%s%s] %.1f%%", filled, empty, m.installProgress))
	s.WriteString("\n")
	return s.String()
}

func (m model) finishedView() string {
	var s strings.Builder
	if m.installErr != nil {
		s.WriteString(errorStyle.Render(" ✖ Installation Failed ") + "\n\n")
		s.WriteString(lipgloss.NewStyle().Foreground(white).Render(m.installErr.Error()))
	} else {
		s.WriteString(statusStyle.Render(" ✔ Installation Successful ") + "\n\n")
		s.WriteString("The distribution is now ready for use.\n\n")
		s.WriteString(fmt.Sprintf("Script: %s", lipgloss.NewStyle().Foreground(blue).Render(m.installResult)))
	}
	s.WriteString("\n")
	return s.String()
}

func (m model) launchSelectView() string {
	distro := m.installedDistros[m.selectedInstalledIdx]
	var s strings.Builder
	s.WriteString(headerStyle.Render("Launch "+distro.Name) + "\n\n")
	s.WriteString(lipgloss.NewStyle().Foreground(gray).Render("Select user account:") + "\n\n")
	for i, user := range distro.Users {
		if i == m.selectedUserIdx {
			s.WriteString(selectedItemStyle.Render(" ➜ " + user))
		} else {
			s.WriteString(itemStyle.Render(user))
		}
		s.WriteString("\n")
	}
	return s.String()
}

func Run() {
	m := NewModel()
	p := tea.NewProgram(m)
	if _, err := p.Run(); err != nil {
		fmt.Printf("Error running program: %v", err)
		os.Exit(1)
	}
}
