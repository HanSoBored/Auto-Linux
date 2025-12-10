use crate::core::{device::DeviceInfo, distro::{Distro, DistroFamily}};
use std::sync::mpsc::Receiver;
use crate::core::install::InstallState;
use std::path::PathBuf;

#[derive(Debug, Clone)]
pub struct InstalledDistro {
    pub name: String,
    #[allow(dead_code)]
    pub path: PathBuf,
    pub script_path: PathBuf,
    pub users: Vec<String>,
}

#[derive(Debug, Clone)]
pub enum CurrentScreen {
    Dashboard,
    DistroFamilySelect,
    DistroVersionSelect,
    UserCredentials,
    Installing,
    Finished,
    LaunchSelect,
}

#[derive(PartialEq)]
pub enum InputMode {
    Username,
    Password,
}

pub struct App {
    pub device: DeviceInfo,
    pub current_screen: CurrentScreen,

    pub distro_families: Vec<DistroFamily>,
    pub selected_family_index: usize,
    pub selected_version_index: usize,

    pub installed_distros: Vec<InstalledDistro>,
    pub selected_installed_index: usize,
    pub selected_launch_user_index: usize,

    pub input_username: String,
    pub input_password: String,
    pub input_mode: InputMode,

    pub install_status: String,
    pub install_progress: f32,
    pub install_rx: Option<Receiver<InstallState>>,
}

impl App {
    pub fn new() -> Self {
        let device = DeviceInfo::new();
        let distro_families = Distro::get_all_families(&device.arch);
        let installed_distros = Distro::scan_installed_distros();

        Self {
            device,
            current_screen: CurrentScreen::Dashboard,
            distro_families,
            selected_family_index: 0,
            selected_version_index: 0,

            installed_distros,
            selected_installed_index: 0,
            selected_launch_user_index: 0,

            input_username: String::new(),
            input_password: String::new(),
            input_mode: InputMode::Username,
            install_status: String::new(),
            install_progress: 0.0,
            install_rx: None,
        }
    }

    pub fn refresh_installed_distros(&mut self) {
        self.installed_distros = Distro::scan_installed_distros();
    }

    pub fn next_family(&mut self) {
        if !self.distro_families.is_empty() && self.selected_family_index < self.distro_families.len() - 1 {
            self.selected_family_index += 1;
        }
    }

    pub fn previous_family(&mut self) {
        if self.selected_family_index > 0 {
            self.selected_family_index -= 1;
        }
    }

    pub fn next_version(&mut self) {
        if let Some(family) = self.distro_families.get(self.selected_family_index) {
            if !family.variants.is_empty() && self.selected_version_index < family.variants.len() - 1 {
                self.selected_version_index += 1;
            }
        }
    }

    pub fn previous_version(&mut self) {
        if self.selected_version_index > 0 {
            self.selected_version_index -= 1;
        }
    }

    pub fn get_selected_distro(&self) -> Option<&Distro> {
        self.distro_families.get(self.selected_family_index)
            .and_then(|f| f.variants.get(self.selected_version_index))
    }

    pub fn start_install(&mut self) {
        if let Some(distro_ref) = self.get_selected_distro() {
            let distro = distro_ref.clone();
             let username = self.input_username.clone();
             let password = self.input_password.clone();
             use std::sync::mpsc::channel;
             use std::thread;
             let (tx, rx) = channel();
             self.install_rx = Some(rx);
             self.current_screen = CurrentScreen::Installing;
             thread::spawn(move || {
                let result = crate::core::install::install_distro(&distro, &username, &password, |state| {
                    let _ = tx.send(state);
                });
                if let Err(e) = result {
                    let _ = tx.send(InstallState::Error(e.to_string()));
                }
             });
        }
    }

    pub fn update_install_state(&mut self) {
        use std::sync::mpsc::TryRecvError;
        if let Some(rx) = &self.install_rx {
            loop {
                match rx.try_recv() {
                    Ok(state) => match state {
                        InstallState::Starting => self.install_status = "Initializing...".to_string(),
                        InstallState::Downloading(pct) => {
                            self.install_status = format!("Downloading Rootfs... {:.1}%", pct);
                            self.install_progress = pct;
                        },
                        InstallState::Extracting => self.install_status = "Extracting Archive (This takes CPU)...".to_string(),
                        InstallState::Configuring => self.install_status = "Configuring Environment...".to_string(),
                        InstallState::Finished(path) => {
                            self.install_status = format!("Success! Run: sh {}", path);
                            self.current_screen = CurrentScreen::Finished;
                        },
                        InstallState::Error(err) => {
                            self.install_status = format!("ERROR: {}", err);
                        }
                    },
                    Err(TryRecvError::Empty) => break,
                    Err(TryRecvError::Disconnected) => break,
                }
            }
        }
    }
}