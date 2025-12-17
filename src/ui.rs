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
        CurrentScreen::DistroVersionSelect => {
            render_family_select(f, app, chunks[1]);
            render_version_select(f, app, chunks[1]);
        }
        CurrentScreen::UserCredentials => {
            render_family_select(f, app, chunks[1]);
            render_user_credentials(f, app, chunks[1]);
        }
        CurrentScreen::Installing => {
            render_dashboard(f, app, chunks[1]);
            render_installing(f, app, chunks[1]);
        }
        CurrentScreen::Finished => {
            render_dashboard(f, app, chunks[1]);
            render_finished(f, app, chunks[1]);
        }
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
            let (distro_display, version_display) = parse_distro_display_name(&d.name);

            let mut display_name = distro_display;
            if !version_display.is_empty() {
                display_name = format!("{} ({})", display_name, version_display);
            }

            let user_desc = if d.users.len() > 1 {
                let other_users: Vec<&str> = d.users.iter().filter(|&u| u != "root").map(|s| s.as_str()).collect();
                format!("(root & {})", other_users.join(", "))
            } else {
                "(root)".to_string()
            };

            let desc = format!("{} {}", display_name, user_desc);
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

fn parse_distro_display_name(name: &str) -> (String, String) {
    let parts: Vec<&str> = name.split('-').collect();
    if parts.is_empty() {
        return (name.to_string(), "".to_string());
    }

    let mut unique_parts: Vec<&str> = Vec::new();
    for part in parts.iter() {
        if unique_parts.is_empty() || unique_parts.last().unwrap() != part {
            unique_parts.push(part);
        }
    }

    let mut distro_name_parts: Vec<String> = Vec::new();
    let mut version_info_parts: Vec<String> = Vec::new();
    let mut version_found = false;

    distro_name_parts.push(
        unique_parts[0].chars().next().unwrap().to_uppercase().to_string() + &unique_parts[0][1..]
    );

    for part in &unique_parts[1..] {
        let is_version_keyword = *part == "rolling" || *part == "latest" || *part == "edge";
        let is_numeric_version = part.chars().any(|c| c.is_digit(10));

        if is_version_keyword || is_numeric_version {
            version_found = true;
        }

        if version_found {
            version_info_parts.push(
                 part.chars().next().unwrap().to_uppercase().to_string() + &part[1..]
            );
        } else {
            distro_name_parts.push(
                part.chars().next().unwrap().to_uppercase().to_string() + &part[1..]
            );
        }
    }
    
    (distro_name_parts.join(" "), version_info_parts.join("-"))
}

fn render_launch_select(f: &mut Frame, app: &App) {
    let distro = &app.installed_distros[app.selected_installed_index];
    let (distro_display, version_display) = parse_distro_display_name(&distro.name);
    let mut display_name = distro_display;
    if !version_display.is_empty() {
        display_name = format!("{} ({})", display_name, version_display);
    }
    let title = format!(" Launch: {} ", display_name);
    let instruction = "Select user to login:";

    let items: Vec<ListItem> = distro.users.iter().map(|u| {
        ListItem::new(format!("  User: {}", u))
    }).collect();

    let item_widths: Vec<usize> = distro.users.iter().map(|u| {
        u.len() + 8 // "  User: {}" + "> "
    }).collect();

    render_list_popup(
        f,
        &title,
        Some(instruction),
        items,
        item_widths,
        app.selected_launch_user_index,
        Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD),
        "> ",
    );
}

fn centered_rect_with_size(width: u16, height: u16, r: ratatui::layout::Rect) -> ratatui::layout::Rect {
    let popup_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length((r.height.saturating_sub(height)) / 2),
            Constraint::Length(height),
            Constraint::Length((r.height.saturating_sub(height)) / 2),
        ])
        .split(r);

    Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Length((r.width.saturating_sub(width)) / 2),
            Constraint::Length(width),
            Constraint::Length((r.width.saturating_sub(width)) / 2),
        ])
        .split(popup_layout[1])[1]
}

fn render_list_popup(
    f: &mut Frame,
    title: &str,
    instruction: Option<&str>,
    items: Vec<ListItem>,
    item_widths: Vec<usize>,
    selected_index: usize,
    highlight_style: Style,
    highlight_symbol: &str,
) {
    let instruction_len = instruction.map_or(0, |s| s.len());
    let instruction_height = instruction.map_or(0, |_| 2); // Height for the instruction paragraph

    // --- Calculate dynamic width ---
    let max_item_width = item_widths.iter().max().copied().unwrap_or(0);
    let mut content_width = std::cmp::max(max_item_width, title.len());
    content_width = std::cmp::max(content_width, instruction_len);
    // Add padding for borders and margin
    let popup_width = std::cmp::max(content_width, 20) as u16 + 4;

    // --- Calculate dynamic height ---
    let mut popup_height = items.len() as u16 + instruction_height;
    popup_height += 2; // for block borders
    popup_height = popup_height.max(5); // Ensure a minimum height

    let popup_area = centered_rect_with_size(popup_width, popup_height, f.area());
    f.render_widget(Clear, popup_area);

    let block = Block::default().title(title.to_string()).borders(Borders::ALL);
    let inner_area = block.inner(popup_area);
    f.render_widget(block, popup_area);

    let constraints = if instruction.is_some() {
        vec![Constraint::Length(instruction_height), Constraint::Min(0)]
    } else {
        vec![Constraint::Min(0)]
    };

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .margin(1)
        .constraints(constraints)
        .split(popup_area);
    
    let list_area = if let Some(instr) = instruction {
        f.render_widget(Paragraph::new(instr), chunks[0]);
        chunks[1]
    } else {
        inner_area
    };

    let list = List::new(items)
        .highlight_style(highlight_style)
        .highlight_symbol(highlight_symbol);

    let mut state = ListState::default();
    state.select(Some(selected_index));
    f.render_stateful_widget(list, list_area, &mut state);
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
        .highlight_style(Style::default().bg(Color::DarkGray).fg(Color::White).add_modifier(Modifier::BOLD))
        .highlight_symbol(">> ");

    let mut state = ListState::default();
    state.select(Some(app.selected_family_index));
    f.render_stateful_widget(list, area, &mut state);
}

fn render_version_select(f: &mut Frame, app: &mut App, _area: ratatui::layout::Rect) {
    let family = &app.distro_families[app.selected_family_index];
    let title = format!(" Select {} Version ", family.name);

    let items: Vec<ListItem> = family.variants
        .iter()
        .map(|d| {
            ListItem::new(Line::from(vec![
                Span::styled(format!("{} ", d.name), Style::default().add_modifier(Modifier::BOLD)),
                Span::styled(format!("({})", d.version), Style::default().fg(Color::DarkGray)),
            ]))
        })
        .collect();
    
    let item_widths: Vec<usize> = family.variants
        .iter()
        .map(|d| d.name.len() + d.version.len() + 5) // " name (version)" + highlight symbol
        .collect();

    render_list_popup(
        f,
        &title,
        None,
        items,
        item_widths,
        app.selected_version_index,
        Style::default().bg(Color::DarkGray).fg(Color::White).add_modifier(Modifier::BOLD),
        "→ ",
    );
}

fn render_installing(f: &mut Frame, app: &mut App, _area: ratatui::layout::Rect) {
    let popup_width = 70;
    let popup_height = 8;

    let popup_area = centered_rect_with_size(popup_width, popup_height, f.area());
    f.render_widget(Clear, popup_area);

    let block = Block::default().title(" Installing Distro ").borders(Borders::ALL);
    f.render_widget(block, popup_area);

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .margin(1)
        .constraints([
            Constraint::Length(3),
            Constraint::Length(3),
        ])
        .split(popup_area);

    let status = Paragraph::new(app.install_status.clone())
        .style(Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD))
        .block(Block::default().borders(Borders::NONE));
    f.render_widget(status, chunks[0]);

    let gauge = Gauge::default()
        .block(Block::default().borders(Borders::ALL).title(" Progress "))
        .gauge_style(Style::default().fg(Color::DarkGray).bg(Color::White))
        .ratio((app.install_progress / 100.0) as f64);
    f.render_widget(gauge, chunks[1]);
}

fn render_finished(f: &mut Frame, app: &mut App, _area: ratatui::layout::Rect) {
    let popup_width = 80;
    let popup_height = 11;
    let popup_area = centered_rect_with_size(popup_width, popup_height, f.area());

    f.render_widget(Clear, popup_area);

    let text = vec![
        Line::from(Span::styled("Installation Complete!", Style::default().fg(Color::Green).add_modifier(Modifier::BOLD))),
        Line::from(""),
        Line::from(format!("Launch command: {}", app.install_status)),
        Line::from(""),
        Line::from("On first launch, it will configure 'aid_inet' and add user 'han' automatically."),
        Line::from("Press ENTER to exit."),
    ];

    let p = Paragraph::new(text)
        .block(Block::default().title(" Finished ").borders(Borders::ALL).style(Style::default().fg(Color::White)))
        .alignment(ratatui::layout::Alignment::Center)
        .wrap(ratatui::widgets::Wrap { trim: false });
        
    f.render_widget(p, popup_area);
}

fn render_user_credentials(f: &mut Frame, app: &mut App, _area: ratatui::layout::Rect) {
    let popup_width = 65;
    let popup_height = 10;

    let popup_area = centered_rect_with_size(popup_width, popup_height, f.area());
    f.render_widget(Clear, popup_area);

    let block = Block::default().title(" Create New User ").borders(Borders::ALL);
    f.render_widget(block, popup_area);

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .margin(1)
        .constraints([
            Constraint::Length(3),
            Constraint::Length(3),
            Constraint::Min(1),
        ])
        .split(popup_area);

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