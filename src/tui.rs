use crate::monitor::*;
use crate::xrandr::*;
use crate::{App, Dir, FocusedWindow, MenuEntry, State};

use std::io;
use std::process::Command;

use ratatui::{
    backend::CrosstermBackend,
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Style, Modifier},
    text::{Span, Line},
    widgets::{Block, Borders, Paragraph, Wrap, canvas::{Canvas, Rectangle}},
    Terminal,
    Frame,
};

use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};

fn main_loop<B: ratatui::backend::Backend>(terminal: &mut Terminal<B>, mut monitors: Monitors, debug: bool, app_states: &mut Vec<Monitors>) -> io::Result<()> {
    // initial setup
    let mut app = App::new(State::MonitorEdit, debug);

    // push a copy of the initial state to the history
    app_states.push((*monitors.clone()).to_vec());

    loop {
        terminal.draw(|f| render_ui::<B>(f, &app, &monitors))?;

        if let Event::Key(key) = event::read()? {
            handle_key_press(key.code, &mut monitors, &mut app, app_states);
        }

        if matches!(app.state, State::Quit) {
            return Ok(());
        }
    }
}


pub fn run_tui(debug: bool) -> Result<(), io::Error> {
    // Get monitor information
    match get_monitor_info(debug) {
        Ok(mut monitors) => {
            // Setup terminal
            enable_raw_mode()?;
            let mut stdout = io::stdout();
            execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
            let backend = CrosstermBackend::new(stdout);
            let mut terminal = Terminal::new(backend)?;

            monitor_proximity(&mut monitors);

            let mut app_states: Vec<Monitors> = Vec::new();

            // Run the main loop
            let _res = main_loop(&mut terminal, monitors, debug, &mut app_states);

            // Restore terminal
            disable_raw_mode()?;
            execute!(
                terminal.backend_mut(),
                LeaveAlternateScreen,
                DisableMouseCapture
            )?;
            terminal.show_cursor()?;

        }
        Err(err) => {
            println!("");
            println!("FATAL: Problem with xrandr, can't be run!");
            println!("Error: {:?}", err);
        }
    }

    println!("");

    Ok(())
}

fn render_debug_popup(f: &mut Frame, monitors: &Monitors) {
    // Create a centered pop-up
    let popup_area = centered_rect(60, 20, f.area());

    // Command display block
    let block = Block::default()
        .title("Constructed Command")
        .borders(Borders::ALL)
        .style(Style::default().fg(Color::White).bg(Color::Black));

    let mut iterator = monitors.iter();
    let mut args: Vec<String> = Vec::new();
    while let Some(element) = iterator.next() {
        args.push("\n> ".to_string());
        args.push("--output".to_string());
        args.push(element.name.to_string());
        if element.is_primary { args.push("--primary".to_string()); }
        args.push("--mode".to_string());
        args.push(format!("{}x{}", element.resolution.0, element.resolution.1));
        args.push("--rate".to_string());
        args.push(element.framerate.to_string());
        args.push("--pos".to_string());
        args.push(format!("{}x{}", element.position.0, element.position.1));
        args.push("--scale".to_string());
        args.push(format!("{:.2}", element.scale));
    };

    let command = format!("xrandr {}", args.join(" "));

    // Command text
    let paragraph = Paragraph::new(command)
        .block(block)
        .style(Style::default().fg(Color::Yellow))
        .wrap(Wrap {trim: true });

    f.render_widget(paragraph, popup_area);
}

fn render_help_popup(f: &mut Frame) {
    // help window with commands
    let help_popup_area = centered_rect(60, 20, f.area());
    let commands = {[
        ("?", "help"),
        ("<Enter>", "Edit selected monitor information"),
        ("<Esc>", "Stop editing"),
        ("m", "Enter monitor mode"),
        ("r", "Reset to previously saved state (UNIMPLEMENTED)"),
        ("s", "Apply saved changes"),
        ("u", "Undo last change"),
        ("d", "Preview xrandr command"),
    ]};

    let info: Vec<Line> = commands
        .iter()
        .map(|(cmd, desc)| {
            Line::from(vec![
                Span::styled(
                    format!("{}: {}", cmd, desc),
                    Style::default()
                )
            ])
        })
        .collect();

    let info_block = Block::default()
        .title("Commands (Main Mode)")
        .borders(Borders::ALL)
        .style(Style::default().fg(Color::LightBlue));

    let info_paragraph = Paragraph::new(info)
        .block(info_block)
        .style(Style::default().fg(Color::White))
        .wrap(ratatui::widgets::Wrap { trim: true });

    f.render_widget(info_paragraph, help_popup_area);
}

fn render_main_ui(f: &mut Frame, app: &App, monitors: &Monitors) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints(
            [
                Constraint::Percentage(70),
                Constraint::Percentage(30)
            ]
                .as_ref())
        .split(f.area());

    let monitor_block = Block::default()
        .title("Monitors")
        .borders(Borders::ALL)
        .style(Style::default().fg(if matches!(app.focused_window, FocusedWindow::MonitorList) {
            if matches!(app.state, State::MonitorEdit) {
                Color::LightMagenta
            } else {
                Color::Yellow
            }
        } else {
                Color::White
            }));

    let monitor_area = monitor_block.inner(chunks[0]);
    f.render_widget(monitor_block, chunks[0]);

    draw_monitors(f, monitor_area, &monitors, *app);

    let info = generate_monitor_info(&monitors, *app);

    let info_block = Block::default()
        .title("Monitor Info")
        .borders(Borders::ALL)
        .style(Style::default().fg(
            if matches!(app.state, State::MenuSelect | State::InfoEdit) {
                Color::LightMagenta
            } else if matches!(app.focused_window, FocusedWindow::MonitorInfo) {
                Color::Yellow
            } else {
                Color::White
            }));

    let info_paragraph = Paragraph::new(info)
        .block(info_block)
        .style(Style::default().fg(Color::White))
        .wrap(ratatui::widgets::Wrap { trim: true });

    if matches!(app.menu_entry, MenuEntry::Framerate | MenuEntry::Resolution) {
        let bottom_chunks = Layout::default()
            .direction(Direction::Horizontal)
            .constraints(
                [
                    Constraint::Percentage(50), Constraint::Percentage(50)
                ]
                    .as_ref())
            .split(chunks[1]);

        let extra_info = generate_extra_info(&monitors, *app);
        let title = if matches!(app.menu_entry, MenuEntry::Framerate) {"Framerate"} else {"Resolution"};

        let extra_block = Block::default()
            .title(title)
            .borders(Borders::ALL)
            .style(Style::default().fg(
                if matches!(app.state, State::InfoEdit) {
                    Color::LightMagenta
                } else {
                    Color::White
                }));

        let extra_paragraph = Paragraph::new(extra_info)
            .block(extra_block)
            .style(Style::default().fg(Color::White))
            .wrap(ratatui::widgets::Wrap { trim: true });

        f.render_widget(info_paragraph, bottom_chunks[0]);
        f.render_widget(extra_paragraph, bottom_chunks[1]);
    } else {
        f.render_widget(info_paragraph, chunks[1]);
    }
}

fn render_ui<B: ratatui::backend::Backend>(f: &mut Frame, app: &App, monitors: &Monitors) {
    match app.state {
        State::DebugPopup   => render_debug_popup(f, monitors),
        State::HelpPopup    => render_help_popup(f),
        _                   => render_main_ui(f, app, monitors),
    }
}

pub fn handle_key_press(key: KeyCode, mut monitors: &mut Monitors, mut app: &mut App, app_states: &mut Vec<Monitors>) {
    match key {
        // help
        KeyCode::Char('?') => {
            if matches!(app.state, State::MonitorEdit | State::MonitorSwap | State::MenuSelect | State::InfoEdit) {
                app.update_state(State::HelpPopup);
            }
        }
        // debug the command
        KeyCode::Char('d') => {
            if matches!(app.state, State::MonitorEdit | State::MonitorSwap | State::MenuSelect | State::InfoEdit) {
                app.update_state(State::DebugPopup);
            }
        }
        KeyCode::Char('q') => app.update_state(State::Quit),
        // save: send to xrandr
        KeyCode::Char('s') => send_to_xrandr(&monitors, *app),
        KeyCode::Char('u') => {
            if matches!(app.state, State::MonitorEdit | State::MonitorSwap | State::MenuSelect | State::InfoEdit) {
                if let Some(last_state) = app_states.pop() {
                    *monitors = last_state;

                    // if empty, push a copy of the default state
                    if app_states.len() == 0 {
                        app_states.push((*monitors.clone()).to_vec());
                    }
                }
            }
        }
        // horizontal movement
        KeyCode::Char('h') | KeyCode::Char('l') | KeyCode::Left | KeyCode::Right => {
            let is_right = matches!(key, KeyCode::Char('l') | KeyCode::Right);
            let direction = if is_right { Dir::Right } else { Dir::Left };

            match app.state {
                State::MonitorEdit => handle_monitor_edit(&mut app, &mut monitors, direction),
                State::MonitorSwap => handle_monitor_swap(&mut app, &mut monitors, direction),
                State::MenuSelect if matches!(app.menu_entry, MenuEntry::Scale) => handle_menu_scale(&mut app, &mut monitors, direction),
                _ => {} // Unimplemented
            }
        }

        // verticalmuct movement
        KeyCode::Char('j') | KeyCode::Char('k') | KeyCode::Up | KeyCode::Down => {
            let is_down = matches!(key, KeyCode::Char('j') | KeyCode::Down);
            let direction = if is_down { Dir::Down } else { Dir::Up };

            match app.state {
                State::MonitorEdit  => handle_monitor_edit(&mut app, &mut monitors, direction),
                State::MonitorSwap  => handle_monitor_swap(&mut app, &mut monitors, direction),
                State::MenuSelect   => handle_menu_select(&mut app, is_down),
                State::InfoEdit     => handle_info_edit(&mut app, &monitors, is_down),
                _ => {} // Unimplemented
            }
        }
        // selection
        KeyCode::Enter => {
            match app.state {
                State::MonitorEdit => {
                    if monitors[app.current_idx].is_selected {
                        monitors[app.current_idx].is_selected = false;
                    } else {
                        app.current_idx = app.selected_idx;
                        monitors[app.selected_idx].is_selected = true;
                    }
                    app.update_state(State::MenuSelect);
                    app.focused_window = FocusedWindow::MonitorInfo;
                }
                State::MonitorSwap => {
                    app.focused_window = FocusedWindow::MonitorInfo;
                    if matches!(app.previous_state, State::MonitorEdit) {
                        monitors[app.current_idx].is_selected = false;
                        app.focused_window = FocusedWindow::MonitorList;
                    }
                    app.update_state(app.previous_state);
                }
                State::MenuSelect => if matches!(app.menu_entry, MenuEntry::Framerate | MenuEntry::Resolution) { app.update_state(State::InfoEdit); },
                State::InfoEdit => {
                    assert!(matches!(app.menu_entry, MenuEntry::Framerate | MenuEntry::Resolution), "Editing something that's not Framerate or resolution!");
                    app_states.push((*monitors.clone()).to_vec());
                    if matches!(app.menu_entry, MenuEntry::Resolution) {
                        let old_res = monitors[app.selected_idx].displayed_resolution;
                        let new_res = monitors[app.selected_idx].sort_resolutions()[app.extra_entry];
                        let difference = (new_res.0 - old_res.0, new_res.1 - old_res.1);
                        shift_res(&mut monitors, app.current_idx, difference);

                        let updated_res = monitors[app.selected_idx].sort_resolutions()[app.extra_entry];
                        monitors[app.selected_idx].resolution = *updated_res;
                        let updated_res = monitors[app.selected_idx].sort_resolutions()[app.extra_entry];
                        monitors[app.selected_idx].displayed_resolution = *updated_res;
                        monitors[app.selected_idx].set_framerate(0);
                        monitors[app.selected_idx].scale = 1.0;
                    } else {
                        monitors[app.selected_idx].set_framerate(app.extra_entry);
                    }
                }
                State::DebugPopup | State::HelpPopup => app.update_state(app.previous_state),
                _ => {} //unimplemented
            }
        }
        // move
        KeyCode::Char('m') => {
            if matches!(app.state, State::MonitorEdit | State::MenuSelect) {
                app_states.push((*monitors.clone()).to_vec());
                if matches!(app.state, State::MonitorEdit) {
                    monitors[app.selected_idx].is_selected = true;
                    app.current_idx = app.selected_idx;
                }
                app.update_state(State::MonitorSwap);
                app.focused_window = FocusedWindow::MonitorList;
            }
        }
        // set primary
        KeyCode::Char('p') => {
            if matches!(app.state, State::MonitorEdit | State::MonitorSwap) {
                app_states.push((*monitors.clone()).to_vec());
                let mut iterator = monitors.iter_mut();
                while let Some(element) = iterator.next() {
                    element.is_primary = false;
                }
                monitors[app.selected_idx].is_primary = true;
            }
        }
        // Deselect
        KeyCode::Esc => {
            match app.state{
                State::MenuSelect => {
                    monitors[app.current_idx].is_selected = false;
                    app.update_state(State::MonitorEdit);
                    app.focused_window = FocusedWindow::MonitorList;
                }
                State::MonitorSwap => {
                    app.focused_window = FocusedWindow::MonitorInfo;
                    if matches!(app.previous_state, State::MonitorEdit) {
                        monitors[app.current_idx].is_selected = false;
                        app.focused_window = FocusedWindow::MonitorList;
                    }
                    app.update_state(app.previous_state);
                    app_states.push((*monitors.clone()).to_vec());
                }
                State::InfoEdit => {
                    app.update_state(State::MenuSelect);
                }
                State::DebugPopup => {
                    app.update_state(app.previous_state);
                }
                State::HelpPopup => {
                    app.update_state(app.previous_state);
                }
                _ => {}
            }
        }
        _ => {}
    }
}

// helper functions
fn handle_monitor_edit(app: &mut App, monitors: &mut Monitors, direction: Dir) {
    if let Some(new_idx) = get_adjacent_monitor(monitors, app.selected_idx, direction) {
        app.selected_idx = new_idx;

        if matches!(app.state, State::MonitorSwap) {
            app.extra_entry = 0;
            swap_monitors(monitors, app.current_idx, new_idx, direction);
            app.current_idx = new_idx;
        }
    }
}

fn handle_monitor_swap(app: &mut App, monitors: &mut Monitors, direction: Dir) {
    let mut swap = false;
    let mut traverse = false;

    if let Some(new_idx) = get_adjacent_monitor(monitors, app.selected_idx, direction) {
        app.selected_idx = new_idx;
        swap = true;
    } else {
        traverse = traverse_monitors(monitors, app.selected_idx, direction);
    }

    if swap {
        app.extra_entry = 0;
        swap_monitors(monitors, app.current_idx, app.selected_idx, direction);
        app.current_idx = app.selected_idx;
    } else if !traverse {
        match direction {
            Dir::Left | Dir::Right => {
                if let Some((pivot_monitor, vert_direction)) = find_vertical_pivot(monitors, app.selected_idx, direction) {
                    horizontal_push(monitors, pivot_monitor, vert_direction, direction, *app);
                }
            }
            Dir::Up | Dir::Down => {
                if let Some((pivot_monitor, vert_direction)) = find_horizontal_pivot(monitors, app.selected_idx, direction) {
                    vert_push(monitors, pivot_monitor, vert_direction, direction, *app);
                }
            }
        }
    }
}

fn handle_menu_scale(app: &mut App, monitors: &mut Monitors, direction: Dir) {
    let scale_delta = if direction == Dir::Right { 0.05 } else { -0.05 };
    monitors[app.selected_idx].scale += scale_delta;

    let difference = monitors[app.selected_idx].get_res_difference();
    shift_res(monitors, app.current_idx, difference);
    monitors[app.selected_idx].update_scale();
}

fn get_adjacent_monitor(monitors: &Monitors, idx: usize, direction: Dir) -> Option<usize> {
    match direction {
        Dir::Right  => monitors[idx].right,
        Dir::Left   => monitors[idx].left,
        Dir::Up     => monitors[idx].up,
        Dir::Down   => monitors[idx].down,
    }
}

fn handle_menu_select(app: &mut App, is_down: bool) {
    app.menu_entry = if is_down { app.get_next_menu_item() } else { app.get_prev_menu_item() };
    app.extra_entry = 0;
}

fn handle_info_edit(app: &mut App, monitors: &Monitors, is_down: bool) {
    let max_length = if app.menu_entry == MenuEntry::Framerate {
        monitors[app.selected_idx]
            .available_resolutions
            .get(&monitors[app.selected_idx].resolution)
            .expect("No available framerates")
            .len()
        - 1
    } else {
        monitors[app.selected_idx].available_resolutions.keys().len() - 1
    };

    if is_down {
        if app.extra_entry < max_length {
            app.extra_entry += 1;
        }
    } else if app.extra_entry > 0 {
        app.extra_entry -= 1;
    }
}

fn find_horizontal_pivot(monitors: &Monitors, idx: usize, direction: Dir) -> Option<(usize, Dir)> {
    if let Some(left) = monitors[idx].left {
        if direction == Dir::Up && monitors[left].up.is_none()
        || direction == Dir::Down && monitors[left].down.is_none()
        {
            return Some((left, Dir::Left));
        }
    }
    if let Some(right) = monitors[idx].right {
        if direction == Dir::Up && monitors[right].up.is_none()
        || direction == Dir::Down && monitors[right].down.is_none()
        {
            return Some((right, Dir::Right));
        }
    }
    None
}

fn find_vertical_pivot(monitors: &Monitors, idx: usize, direction: Dir) -> Option<(usize, Dir)> {
    if let Some(up) = monitors[idx].up {
        if direction == Dir::Left && monitors[up].left.is_none()
        || direction == Dir::Right && monitors[up].right.is_none()
        {
            return Some((up, Dir::Up));
        }
    }
    if let Some(down) = monitors[idx].down {
        if direction == Dir::Left && monitors[down].left.is_none()
        || direction == Dir::Right && monitors[down].right.is_none()
        {
            return Some((down, Dir::Down));
        }
    }
    None
}

fn send_to_xrandr(monitors: &Monitors, app: App) {
    if !app.debug && matches!(app.state, State::MonitorEdit | State::MonitorSwap | State::MenuSelect | State::InfoEdit) {
        let mut iterator = monitors.iter();
        let mut args: Vec<String> = Vec::new();
        while let Some(element) = iterator.next() {
            args.push("--output".to_string());
            args.push(element.name.to_string());
            if element.is_primary { args.push("--primary".to_string()); }
            args.push("--mode".to_string());
            args.push(format!("{}x{}", element.resolution.0, element.resolution.1));
            args.push("--rate".to_string());
            args.push(element.framerate.to_string());
            args.push("--pos".to_string());
            args.push(format!("{}x{}", element.position.0, element.position.1));
            args.push("--scale".to_string());
            args.push(format!("{:.2}", element.scale));
        };

        // TODO: display this in a popup
        let output = Command::new("xrandr")
            .args(args)
            .output()
            .expect("failed to execute process");

        println!("{:?}", output);
    }
}

// Generate the Line to draw extra information (e.g. framerate)
fn generate_extra_info(
    monitors: &Monitors,
    app: App,
) -> Vec<Line> {
    if let Some(monitor) = monitors.get(app.selected_idx) {
        if app.menu_entry == MenuEntry::Framerate {
            if let Some(framerates) = monitor.available_resolutions.get(&monitor.resolution) {
                let framerate_line: Vec<Line> = framerates
                    .iter()
                    .enumerate()
                    .map(|(i, fr)| {
                        Line::from(vec![
                            Span::styled(
                                format!("Option {}: {}hz", i, fr),
                                if app.extra_entry == i {
                                    Style::default()
                                        .add_modifier(Modifier::BOLD)
                                        .fg(if matches!(app.state, State::InfoEdit) {
                                            Color::Yellow
                                        } else {
                                                Color::White
                                            }
                                        )
                                } else {
                                    Style::default()
                                }
                            )
                        ])
                    })
                    .collect();
                framerate_line
            } else {
                vec![Line::from("No available framerates")]
            }
        } else if app.menu_entry == MenuEntry::Resolution {
            if let Some(resolutions) = Some(monitor.sort_resolutions()) {
                let resolution_line: Vec<Line> = resolutions
                    .iter()
                    .enumerate()
                    .map(|(i, res)| {
                        Line::from(vec![
                            Span::styled(
                                format!("Option {}: {}x{}", i, res.0, res.1),
                                if app.extra_entry == i {
                                    Style::default()
                                        .add_modifier(Modifier::BOLD)
                                        .fg(if matches!(app.state, State::InfoEdit) {
                                            Color::Yellow
                                        } else {
                                                Color::White
                                            }
                                        )
                                } else {
                                    Style::default()
                                }
                            )
                        ])
                    })
                    .collect();
                resolution_line
            } else {
                vec![Line::from("No available resolutions")]
            }
        } else {
            vec![Line::from("Nothing to see here!")]
        }
    } else {
        vec![Line::from("Nothing to see here!")]
    }
}

// Generate the Line from monitor info
fn generate_monitor_info(
    monitors: &Monitors,
    app: App
) -> Vec<Line> {
    fn get_style(app: App, entry: MenuEntry) -> Style {
        if app.menu_entry == entry {
            Style::default()
                .add_modifier(Modifier::BOLD)
                .fg(match app.state {
                    State::InfoEdit => Color::LightMagenta,
                    State::MenuSelect => Color::Yellow,
                    _ => Color::White,
                })
        } else {
            Style::default()
        }
    }

    fn format_monitor_info(label: &str, value: String, style: Style) -> Line {
        Line::from(vec![Span::styled(format!("{label}: {value}"), style)])
    }

    if let Some(monitor) = monitors.get(app.selected_idx) {
        vec![
            format_monitor_info("Name", monitor.name.clone(), get_style(app, MenuEntry::Name)),
            format_monitor_info(
                "Resolution",
                format!("{}x{}", monitor.resolution.0, monitor.resolution.1),
                get_style(app, MenuEntry::Resolution),
            ),
            format_monitor_info(
                "Scale",
                format!("{:.2}", monitor.resolution.0 as f32 / monitor.displayed_resolution.0 as f32),
                get_style(app, MenuEntry::Scale),
            ),
            format_monitor_info(
                "Position",
                format!("({}, {})", monitor.position.0, monitor.position.1),
                get_style(app, MenuEntry::Position),
            ),
            format_monitor_info(
                "Primary",
                if monitor.is_primary { "Yes".to_string() } else { "No".to_string() },
                get_style(app, MenuEntry::Primary),
            ),
            format_monitor_info(
                "Framerate",
                format!("{}hz", monitor.framerate),
                get_style(app, MenuEntry::Framerate),
            ),
            format_monitor_info(
                "Left",
                monitor.left.map_or("None".to_string(), |idx| monitors[idx].name.clone()),
                get_style(app, MenuEntry::Left),
            ),
            format_monitor_info(
                "Down",
                monitor.down.map_or("None".to_string(), |idx| monitors[idx].name.clone()),
                get_style(app, MenuEntry::Down),
            ),
            format_monitor_info(
                "Up",
                monitor.up.map_or("None".to_string(), |idx| monitors[idx].name.clone()),
                get_style(app, MenuEntry::Up),
            ),
            format_monitor_info(
                "Right",
                monitor.right.map_or("None".to_string(), |idx| monitors[idx].name.clone()),
                get_style(app, MenuEntry::Right),
            ),
            format_monitor_info(
                "Resolutions",
                format!(
                    "{:?}",
                    monitor
                        .available_resolutions
                        .get(&monitor.resolution)
                        .expect("No available framerates")
                        .len()
                ),
                get_style(app, MenuEntry::Resolutions),
            ),
        ]
    } else {
        vec![Line::from("No monitor selected")]
    }
}

// Helper function to create a centered rectangle for popups
fn centered_rect(percent_x: u16, percent_y: u16, r: Rect) -> Rect {
    let popup_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints(
            [
                Constraint::Percentage((100 - percent_y) / 2),
                Constraint::Percentage(percent_y),
                Constraint::Percentage((100 - percent_y) / 2),
            ]
                .as_ref(),
        )
        .split(r);

    let horizontal_layout = Layout::default()
        .direction(Direction::Horizontal)
        .constraints(
            [
                Constraint::Percentage((100 - percent_x) / 2),
                Constraint::Percentage(percent_x),
                Constraint::Percentage((100 - percent_x) / 2),
            ]
                .as_ref(),
        )
        .split(popup_layout[1]);

    horizontal_layout[1]
}

// draw monitors as defined
fn draw_monitors(f: &mut ratatui::Frame, area: Rect, monitors: &[Monitor], app: App) {
    let total_width: f64 = monitors.iter().map(|m| m.position.0 + m.displayed_resolution.0 as i32).max().unwrap_or(0).into();
    let total_height: f64 = monitors.iter().map(|m| m.position.1 + m.displayed_resolution.1 as i32).max().unwrap_or(0).into();

    let scale_x = 0.9;
    let scale_y = 0.5;

    let monitor_data: Vec<_> = monitors.iter().enumerate().map(|(i, m)| {
        (i, m.position, m.displayed_resolution, m.is_selected, m.is_primary, m.name.clone())
    }).collect();

    let canvas = Canvas::default()
        .x_bounds([0.0, total_width])
        .y_bounds([0.0, total_height])
        .paint(move |ctx| {
            for (i, position, displayed_resolution, is_selected, is_primary, mut name) in monitor_data.iter().cloned() {
                let x = position.0 as f64 * scale_x + total_width * (1.0 - scale_x)/2.0;
                let y = total_height - (position.1 as f64 * scale_y + total_height * (1.0 - scale_y)/2.0);
                let width = displayed_resolution.0 as f64 * scale_x;
                let height = displayed_resolution.1 as f64 * scale_y * -1.0;

                let color = if is_selected {
                    Color::LightMagenta
                } else if i == app.selected_idx {
                    Color::Yellow
                } else if is_primary {
                    Color::Green
                } else {
                    Color::White
                };

                ctx.draw(&Rectangle {
                    x,
                    y,
                    width,
                    height,
                    color,
                });

                // Draw monitor name
                if is_selected {
                    if is_primary {
                        name = format!("<{}* ({}x{})>", name, displayed_resolution.0, displayed_resolution.1);
                    } else {
                        name = format!("<{} ({}x{})>", name, displayed_resolution.0, displayed_resolution.1);
                    }
                } else if is_primary {
                    name = format!("{}* ({}x{})", name, displayed_resolution.0, displayed_resolution.1);
                } else {
                    name = format!("{} ({}x{})", name, displayed_resolution.0, displayed_resolution.1)
                }
                ctx.print(
                    x + width/2.0 - (name.chars().count()*2) as f64,
                    y + height/2.0,
                    Span::styled(
                        name,
                        Style::default().fg(Color::Black).bg(color)
                    ),
                );
            }
        });

    f.render_widget(canvas, area);
}

