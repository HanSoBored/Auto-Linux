use ratatui::{
    layout::{Constraint, Direction, Layout},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, List, ListItem, ListState, Paragraph, Gauge, Clear},
    Frame,
};
use crate::types::{App, CurrentScreen, InputMode};

pub fn draw(f: &mut Frame, app: &mut App) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3),
            Constraint::Min(5),
            Constraint::Length(3),
        ])
        .split(f.area());

    let title = Paragraph::new(" AutoLinux Installer (Rust Edition) ")
        .style(Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD))
        .block(Block::default().borders(Borders::ALL));
    f.render_widget(title, chunks[0]);

    match app.current_screen {
        CurrentScreen::Dashboard => render_dashboard(f, app, chunks[1]),
        CurrentScreen::DistroFamilySelect => render_family_select(f, app, chunks[1]),
        CurrentScreen::DistroVersionSelect => render_version_select(f, app, chunks[1]),
        CurrentScreen::UserCredentials => render_user_credentials(f, app, chunks[1]),
        CurrentScreen::Installing => render_installing(f, app, chunks[1]),
        CurrentScreen::Finished => render_finished(f, app, chunks[1]),
        CurrentScreen::LaunchSelect => {
            render_dashboard(f, app, chunks[1]);
            render_launch_select(f, app);
        }
    }

    let footer_text = match app.current_screen {
        CurrentScreen::Dashboard => "q: Quit | i: Install New | UP/DOWN: Select Installed | ENTER: Launch",
        CurrentScreen::DistroFamilySelect => "UP/DOWN: Select Family | ENTER: Choose | q/ESC: Back",
        CurrentScreen::DistroVersionSelect => "UP/DOWN: Select Version | ENTER: Choose | ESC: Back",
        CurrentScreen::LaunchSelect => "UP/DOWN: Select User | ENTER: Start Shell | ESC: Cancel",
        CurrentScreen::UserCredentials => "Type to input | ENTER to Switch/Confirm | ESC to Back",
        CurrentScreen::Installing => "Installing... Press 'q' to force quit",
        CurrentScreen::Finished => "Press ENTER to exit",
    };
    let footer = Paragraph::new(footer_text)
        .style(Style::default().fg(Color::Gray))
        .block(Block::default().borders(Borders::ALL));
    f.render_widget(footer, chunks[2]);
}

fn render_dashboard(f: &mut Frame, app: &App, area: ratatui::layout::Rect) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(6),
            Constraint::Min(5),
        ])
        .split(area);

    let device = &app.device;
    let root_status_span = if device.is_root {
        Span::styled("Native Root (UID 0)", Style::default().fg(Color::Green).add_modifier(Modifier::BOLD))
    } else if device.can_su {
        Span::styled(format!("Granted (Type: {})", device.root_type), Style::default().fg(Color::LightGreen))
    } else {
        Span::styled("Non-Root", Style::default().fg(Color::Red))
    };

    let mode_span = Span::styled("Chroot (Full Access)", Style::default().fg(Color::Cyan));

    let info_text = vec![
        Line::from(vec![Span::raw("Device Architecture : "), Span::styled(&device.arch, Style::default().fg(Color::Magenta))]),
        Line::from(vec![Span::raw("Android Version     : "), Span::styled(&device.android_ver, Style::default().fg(Color::Blue))]),
        Line::from(vec![Span::raw("Root Access         : "), root_status_span]),
        Line::from(vec![Span::raw("Install Mode        : "), mode_span]),
    ];

    let final_text = vec![
        info_text[0].clone(),
        info_text[1].clone(),
        info_text[2].clone(),
        info_text[3].clone(),
        Line::from(""),
        Line::from("Welcome to AutoLinux. Select an installed distro or press 'i' to install a new one."),
    ];


    let info_block = Paragraph::new(final_text)
        .block(Block::default().title(" System Info ").borders(Borders::ALL));
    f.render_widget(info_block, chunks[0]);

    let items: Vec<ListItem> = app.installed_distros
        .iter()
        .map(|d| {
            let user_count = d.users.len().saturating_sub(1);
            let desc = format!("{} (Users: root + {})", d.name, user_count);
            ListItem::new(Line::from(vec![
                Span::styled("   ", Style::default().fg(Color::Green)),
                Span::raw(desc)
            ]))
        })
        .collect();

    let title = if app.installed_distros.is_empty() {
        " Installed Distros (None found) "
    } else {
        " Installed Distros (Select to Launch) "
    };

    let list = List::new(items)
        .block(Block::default().title(title).borders(Borders::ALL))
        .highlight_style(Style::default().bg(Color::DarkGray).add_modifier(Modifier::BOLD))
        .highlight_symbol("→ ");

    let mut state = ListState::default();
    if !app.installed_distros.is_empty() {
        state.select(Some(app.selected_installed_index));
    }

    f.render_stateful_widget(list, chunks[1], &mut state);
}

fn render_launch_select(f: &mut Frame, app: &App) {
    let distro = &app.installed_distros[app.selected_installed_index];

    let block = Block::default().title(format!(" Launch: {} ", distro.name)).borders(Borders::ALL);
    let area = centered_rect(60, 40, f.area());
    f.render_widget(Clear, area);
    f.render_widget(block, area);

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Length(2), Constraint::Min(1)])
        .margin(1)
        .split(area);

    f.render_widget(Paragraph::new("Select user to login:"), chunks[0]);

    let items: Vec<ListItem> = distro.users.iter().map(|u| {
        ListItem::new(format!("  User: {}", u))
    }).collect();

    let list = List::new(items)
        .highlight_style(Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD))
        .highlight_symbol("> ");

    let mut state = ListState::default();
    state.select(Some(app.selected_launch_user_index));

    f.render_stateful_widget(list, chunks[1], &mut state);
}

fn centered_rect(percent_x: u16, percent_y: u16, r: ratatui::layout::Rect) -> ratatui::layout::Rect {
    let popup_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Percentage((100 - percent_y) / 2),
            Constraint::Percentage(percent_y),
            Constraint::Percentage((100 - percent_y) / 2),
        ])
        .split(r);

    Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage((100 - percent_x) / 2),
            Constraint::Percentage(percent_x),
            Constraint::Percentage((100 - percent_x) / 2),
        ])
        .split(popup_layout[1])[1]
}

fn render_family_select(f: &mut Frame, app: &mut App, area: ratatui::layout::Rect) {
    let items: Vec<ListItem> = app.distro_families
        .iter()
        .map(|f| {
            let count = f.variants.len();
            ListItem::new(vec![
                Line::from(Span::styled(format!(" {} ", f.name), Style::default().add_modifier(Modifier::BOLD))),
                Line::from(Span::styled(format!("    {} ({} variants)", f.description, count), Style::default().fg(Color::Gray))),
            ])
        })
        .collect();

    let list = List::new(items)
        .block(Block::default().title(" Select Distribution Family ").borders(Borders::ALL))
        .highlight_style(Style::default().bg(Color::Blue).fg(Color::Black).add_modifier(Modifier::BOLD))
        .highlight_symbol(">> ");

    let mut state = ListState::default();
    state.select(Some(app.selected_family_index));
    f.render_stateful_widget(list, area, &mut state);
}

fn render_version_select(f: &mut Frame, app: &mut App, area: ratatui::layout::Rect) {
    let family = &app.distro_families[app.selected_family_index];

    let items: Vec<ListItem> = family.variants
        .iter()
        .map(|d| {
            ListItem::new(Line::from(vec![
                Span::styled(format!("{} ", d.name), Style::default().add_modifier(Modifier::BOLD)),
                Span::styled(format!("({})", d.version), Style::default().fg(Color::DarkGray)),
            ]))
        })
        .collect();

    let title = format!(" Select {} Version ", family.name);
    let list = List::new(items)
        .block(Block::default().title(title).borders(Borders::ALL))
        .highlight_style(Style::default().bg(Color::Green).fg(Color::Black).add_modifier(Modifier::BOLD))
        .highlight_symbol("→ ");

    let mut state = ListState::default();
    state.select(Some(app.selected_version_index));
    f.render_stateful_widget(list, area, &mut state);
}

fn render_installing(f: &mut Frame, app: &mut App, area: ratatui::layout::Rect) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3),
            Constraint::Length(3),
        ])
        .margin(2)
        .split(area);

    let status = Paragraph::new(app.install_status.clone())
        .style(Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD))
        .block(Block::default().borders(Borders::NONE));
    f.render_widget(status, chunks[0]);

    let gauge = Gauge::default()
        .block(Block::default().borders(Borders::ALL).title(" Progress "))
        .gauge_style(Style::default().fg(Color::Green).bg(Color::Black))
        .percent(app.install_progress as u16);
    f.render_widget(gauge, chunks[1]);
}

fn render_finished(f: &mut Frame, app: &mut App, area: ratatui::layout::Rect) {
    let text = vec![
        Line::from(Span::styled("Installation Complete!", Style::default().fg(Color::Green).add_modifier(Modifier::BOLD))),
        Line::from(""),
        Line::from(format!("Launch command: {}", app.install_status)),
        Line::from(""),
        Line::from("On first launch, it will configure 'aid_inet' and add user 'han' automatically."),
        Line::from("Press ENTER to exit."),
    ];

    let p = Paragraph::new(text)
        .block(Block::default().borders(Borders::ALL).style(Style::default().fg(Color::White)));
    f.render_widget(p, area);
}

fn render_user_credentials(f: &mut Frame, app: &mut App, area: ratatui::layout::Rect) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3),
            Constraint::Length(3),
            Constraint::Min(1),
        ])
        .margin(1)
        .split(area);

    let (user_color, pass_color) = match app.input_mode {
        InputMode::Username => (Color::Yellow, Color::White),
        InputMode::Password => (Color::White, Color::Yellow),
    };

    let user_block = Paragraph::new(app.input_username.as_str())
        .style(Style::default().fg(user_color))
        .block(Block::default().borders(Borders::ALL).title(" Username "));

    let masked_pass: String = app.input_password.chars().map(|_| '*').collect();
    let pass_block = Paragraph::new(masked_pass.as_str())
        .style(Style::default().fg(pass_color))
        .block(Block::default().borders(Borders::ALL).title(" Password "));

    f.render_widget(user_block, chunks[0]);
    f.render_widget(pass_block, chunks[1]);

    let info = Paragraph::new("Enter the username and password for the new Linux system.")
        .style(Style::default().fg(Color::Gray));
    f.render_widget(info, chunks[2]);
}