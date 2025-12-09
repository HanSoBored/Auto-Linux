#[macro_use]
mod core;
mod types;
mod ui;

use std::env;
use std::io;
use std::process::{Command, exit};
use crossterm::{
    event::{self, Event, KeyCode},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{backend::CrosstermBackend, Terminal};
use types::{App, CurrentScreen, InputMode};

fn try_elevate_privileges() {
    if let Ok(exe_path) = env::current_exe() {
        let status = Command::new("su")
            .arg("-c")
            .arg(exe_path)
            .status();

        match status {
            Ok(s) => {
                if s.success() {
                    exit(0);
                }
            },
            Err(_) => {
            }
        }
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    core::logger::init();

    let device_info = core::device::DeviceInfo::new();
    log_info!("Device Arch: {}", device_info.arch);
    log_info!("Android Ver: {}", device_info.android_ver);
    log_info!("Is Root: {}, Can SU: {}", device_info.is_root, device_info.can_su);

    if !device_info.is_root {
        if device_info.can_su {
            log_info!("Not root. Attempting self-elevation...");
            try_elevate_privileges();

            let err_msg = "Failed to gain root access. Please grant permission to the 'su' request.";
            log_error!("{}", err_msg);
            eprintln!("{}", err_msg);
            exit(1);
        } else {
            let err_msg = "Root access is required, but 'su' binary not found.";
            log_error!("{}", err_msg);
            eprintln!("{}", err_msg);
            exit(1);
        }
    }

    log_info!("Root access confirmed. Initializing TUI...");

    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    let mut app = App::new();

    loop {
        if let CurrentScreen::Installing = app.current_screen {
            app.update_install_state();
        }

        terminal.draw(|f| {
            ui::draw(f, &mut app);
        })?;

        if event::poll(std::time::Duration::from_millis(100))? {
             if let Event::Key(key) = event::read()? {
                match app.current_screen {
                    CurrentScreen::Dashboard => {
                        match key.code {
                            KeyCode::Char('q') => break,
                            KeyCode::Char('i') => app.current_screen = CurrentScreen::DistroSelect,
                            KeyCode::Down | KeyCode::Char('j') => {
                                if !app.installed_distros.is_empty() {
                                    let next = (app.selected_installed_index + 1) % app.installed_distros.len();
                                    app.selected_installed_index = next;
                                }
                            },
                            KeyCode::Up | KeyCode::Char('k') => {
                                if !app.installed_distros.is_empty() {
                                    let prev = if app.selected_installed_index == 0 {
                                        app.installed_distros.len() - 1
                                    } else {
                                        app.selected_installed_index - 1
                                    };
                                    app.selected_installed_index = prev;
                                }
                            },
                            KeyCode::Enter => {
                                if !app.installed_distros.is_empty() {
                                    app.selected_launch_user_index = 0;
                                    app.current_screen = CurrentScreen::LaunchSelect;
                                }
                            },
                            _ => {}
                        }
                    },
                    CurrentScreen::LaunchSelect => {
                        let distro_idx = app.selected_installed_index;
                        let users_len = app.installed_distros[distro_idx].users.len();

                        match key.code {
                            KeyCode::Esc => app.current_screen = CurrentScreen::Dashboard,
                            KeyCode::Down | KeyCode::Char('j') => {
                                app.selected_launch_user_index = (app.selected_launch_user_index + 1) % users_len;
                            },
                            KeyCode::Up | KeyCode::Char('k') => {
                                app.selected_launch_user_index = if app.selected_launch_user_index == 0 {
                                    users_len - 1
                                } else {
                                    app.selected_launch_user_index - 1
                                };
                            },
                            KeyCode::Enter => {
                                let distro = &app.installed_distros[distro_idx];
                                let user = &distro.users[app.selected_launch_user_index];
                                let script = distro.script_path.clone();
                                let user_arg = user.clone();

                                disable_raw_mode()?;
                                execute!(terminal.backend_mut(), LeaveAlternateScreen)?;
                                terminal.show_cursor()?;

                                println!("Launching {} as user {}...", distro.name, user);
                                let status = Command::new("sh")
                                    .arg(script)
                                    .arg(user_arg)
                                    .status();

                                match status {
                                    Ok(_) => println!("\n[Process exited] Press ENTER to return to Dashboard..."),
                                    Err(e) => println!("\n[Error] Failed to launch: {}. Press ENTER to continue...", e),
                                }

                                let mut input = String::new();
                                let _ = io::stdin().read_line(&mut input);

                                // Refresh list distro & users after returning from shell
                                // This is important because first-setup creates new users during shell execution
                                app.refresh_installed_distros();

                                enable_raw_mode()?;
                                execute!(terminal.backend_mut(), EnterAlternateScreen)?;
                                terminal.clear()?;
                                app.current_screen = CurrentScreen::Dashboard;
                            },
                            _ => {}
                        }
                    },
                    CurrentScreen::DistroSelect => {
                        match key.code {
                            KeyCode::Char('q') | KeyCode::Esc => app.current_screen = CurrentScreen::Dashboard,
                            KeyCode::Down | KeyCode::Char('j') => app.next_distro(),
                            KeyCode::Up | KeyCode::Char('k') => app.previous_distro(),
                            KeyCode::Enter => {
                                app.input_username.clear();
                                app.input_password.clear();
                                app.input_mode = InputMode::Username;
                                app.current_screen = CurrentScreen::UserCredentials;
                            },
                            _ => {}
                        }
                    },
                    CurrentScreen::UserCredentials => {
                        match key.code {
                            KeyCode::Esc => app.current_screen = CurrentScreen::DistroSelect,
                            KeyCode::Enter => {
                                match app.input_mode {
                                    InputMode::Username => {
                                        if !app.input_username.is_empty() {
                                            app.input_mode = InputMode::Password;
                                        }
                                    },
                                    InputMode::Password => {
                                        if !app.input_password.is_empty() {
                                            app.start_install();
                                        }
                                    }
                                }
                            },
                            KeyCode::Backspace => {
                                match app.input_mode {
                                    InputMode::Username => { app.input_username.pop(); },
                                    InputMode::Password => { app.input_password.pop(); },
                                }
                            },
                            KeyCode::Char(c) => {
                                match app.input_mode {
                                    InputMode::Username => { app.input_username.push(c); },
                                    InputMode::Password => { app.input_password.push(c); },
                                }
                            },
                            _ => {}
                        }
                    },
                    CurrentScreen::Installing => {
                         if let KeyCode::Char('q') = key.code { break; }
                    },
                    CurrentScreen::Finished => {
                        if let KeyCode::Enter | KeyCode::Char('q') = key.code {
                            app.refresh_installed_distros();
                            app.current_screen = CurrentScreen::Dashboard;
                        }
                    },
                }
             }
        }
    }

    disable_raw_mode()?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen)?;
    terminal.show_cursor()?;

    Ok(())
}
