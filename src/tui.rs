use crate::monitor;
use crate::xrandr;
use crate::{App, Dir, FocusedWindow, MenuEntry, State};

use std::io;
use std::process::Command;

use tui::widgets::canvas::Rectangle;
use tui::{
    backend::CrosstermBackend,
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Style, Modifier},
    text::{Spans, Span},
    widgets::{Block, Borders, Paragraph, Wrap, canvas::Canvas},
    Terminal,
};
use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};

pub fn run_tui(debug: bool) -> Result<(), io::Error> {
    // Setup terminal
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    // Get monitor information
    let mut monitors = xrandr::get_monitor_info(debug)?;
    monitor::monitor_proximity(&mut monitors);

    // Run the main loop
    let res = ui(&mut terminal, monitors, debug);

    // Restore terminal
    disable_raw_mode()?;
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture
    )?;
    terminal.show_cursor()?;

    if let Err(err) = res {
        println!("{:?}", err)
    }

    println!("");

    Ok(())
}

fn ui<B: tui::backend::Backend>(terminal: &mut Terminal<B>, mut monitors: Vec<monitor::Monitor>, debug: bool) -> io::Result<()> {
    // initial setup
    let mut app = App::new(debug);

    loop {
        terminal.draw(|f| {
            let size = f.size();

            if matches!(app.state, State::DebugPopup) {
                // Create a centered pop-up
                let popup_area = centered_rect(60, 20, size);

                // Command display block
                let block = Block::default()
                    .title("Constructed Command")
                    .borders(Borders::ALL)
                    .style(Style::default().fg(Color::White).bg(Color::Black));

                let mut iterator = monitors.iter_mut();
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

            } else {
                let chunks = Layout::default()
                    .direction(Direction::Vertical)
                    .constraints(
                        [
                            Constraint::Percentage(70),
                            Constraint::Percentage(30)
                        ]
                        .as_ref())
                    .split(f.size());

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

                draw_monitors(f, monitor_area, &monitors, app);

                let info = generate_monitor_info(&monitors, app);

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
                    .wrap(tui::widgets::Wrap { trim: true });

                if matches!(app.menu_entry, MenuEntry::Framerate | MenuEntry::Resolution) {
                    let bottom_chunks = Layout::default()
                        .direction(Direction::Horizontal)
                        .constraints(
                            [
                                Constraint::Percentage(50), Constraint::Percentage(50)
                            ]
                                .as_ref())
                        .split(chunks[1]);

                    let extra_info = generate_extra_info(&monitors, app);
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
                        .wrap(tui::widgets::Wrap { trim: true });

                    f.render_widget(info_paragraph, bottom_chunks[0]);
                    f.render_widget(extra_paragraph, bottom_chunks[1]);
                } else {
                    f.render_widget(info_paragraph, chunks[1]);
                }
            }

        })?;

        if let Event::Key(key) = event::read()? {
            match key.code {
                // quit
                KeyCode::Char('q') => {
                    return Ok(());
                }
                // debug the command
                KeyCode::Char('d') => {
                    if !matches!(app.state, State::DebugPopup) {
                        app.update_state(State::DebugPopup);
                    }
                }
                // save: send to xrandr
                KeyCode::Char('s') => {
                    if !app.debug {
                        let mut iterator = monitors.iter_mut();
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
                        let output = Command::new("xrandr")
                            .args(args)
                            .output()?;
                        println!("{:?}", output);
                    }
                }
                // horizontal movement
                KeyCode::Char('h') | KeyCode::Char('l') | KeyCode::Left | KeyCode::Right => {
                    match app.state {
                        State::MonitorEdit => {
                            let mut direction: Option<Dir> = None;
                            if (key.code == KeyCode::Char('l')) | (key.code == KeyCode::Right) {
                                if monitors[app.selected_monitor].right.is_some() {
                                    app.selected_monitor = monitors[app.selected_monitor].right.unwrap();
                                    direction = Some(Dir::Right);
                                }
                            } else {
                                if monitors[app.selected_monitor].left.is_some() {
                                    app.selected_monitor = monitors[app.selected_monitor].left.unwrap();
                                    direction = Some(Dir::Left);
                                }
                            }
                            if direction.is_some() && matches!(app.state, State::MonitorSwap)  {
                                app.extra_entry = 0;
                                monitor::swap_monitors(&mut monitors, app.current_monitor, app.selected_monitor, direction.unwrap(), app);
                                app.current_monitor = app.selected_monitor;
                            }
                        }
                        State::MonitorSwap => {
                            let direction: Option<Dir>;
                            let mut pivot_monitor: Option<usize> = None;
                            let mut vert_direction: Option<Dir> = None;
                            let mut swap: bool = false;
                            let mut traverse: bool = false;
                            if (key.code == KeyCode::Char('l')) | (key.code == KeyCode::Right) {
                                if monitors[app.selected_monitor].right.is_some() {
                                    app.selected_monitor = monitors[app.selected_monitor].right.unwrap();
                                    swap = true;
                                } else {
                                    traverse = monitor::traverse_monitors(&mut monitors, app.selected_monitor, Dir::Right);
                                }
                                direction = Some(Dir::Right);
                            } else {
                                if monitors[app.selected_monitor].left.is_some() {
                                    app.selected_monitor = monitors[app.selected_monitor].left.unwrap();
                                    swap = true;
                                } else {
                                    traverse = monitor::traverse_monitors(&mut monitors, app.selected_monitor, Dir::Left);
                                }
                                direction = Some(Dir::Left);
                            }
                            if swap {
                                app.extra_entry = 0;
                                monitor::swap_monitors(&mut monitors, app.current_monitor, app.selected_monitor, direction.unwrap(), app);
                                app.current_monitor = app.selected_monitor;
                            } else if !traverse {
                                //look for up or down
                                if monitors[app.selected_monitor].up.is_some() {
                                    if direction == Some(Dir::Left) && monitors[monitors[app.selected_monitor].up.unwrap()].left.is_none()
                                    || direction == Some(Dir::Right) &&  monitors[monitors[app.selected_monitor].up.unwrap()].right.is_none() {
                                        pivot_monitor = monitors[app.selected_monitor].up;
                                        vert_direction = Some(Dir::Up);
                                    }
                                } else if monitors[app.selected_monitor].down.is_some() {
                                    if direction == Some(Dir::Left) && monitors[monitors[app.selected_monitor].down.unwrap()].left.is_none()
                                    || direction == Some(Dir::Right) &&  monitors[monitors[app.selected_monitor].down.unwrap()].right.is_none() {
                                        pivot_monitor = monitors[app.selected_monitor].down;
                                        vert_direction = Some(Dir::Down);
                                    }
                                }
                                if pivot_monitor.is_some() {
                                    monitor::horizontal_push(&mut monitors, pivot_monitor.unwrap(), vert_direction.unwrap(), direction.unwrap(), app);
                                }
                            }
                        }
                        State::MenuSelect => {
                            if matches!(app.menu_entry, MenuEntry::Scale) {
                                if (key.code == KeyCode::Char('l')) | (key.code == KeyCode::Right) {
                                    monitors[app.selected_monitor].scale += 0.05;
                                } else {
                                    monitors[app.selected_monitor].scale -= 0.05;
                                }
                                let difference = monitors[app.selected_monitor].get_res_difference();
                                monitor::shift_res(&mut monitors, app.current_monitor, difference);
                                monitors[app.selected_monitor].update_scale();
                            }
                        }
                        _ => {} //unimplemented
                    }
                }
                // vertical movement
                KeyCode::Char('j') | KeyCode::Char('k') | KeyCode::Up | KeyCode::Down => {
                    match app.state {
                        State::MonitorEdit  => {
                            let mut direction: Option<Dir> = None;
                            if (key.code == KeyCode::Char('j')) | (key.code == KeyCode::Down) {
                                if monitors[app.selected_monitor].down.is_some() {
                                    app.selected_monitor = monitors[app.selected_monitor].down.unwrap();
                                    direction = Some(Dir::Down);
                                }
                            } else {
                                if monitors[app.selected_monitor].up.is_some() {
                                    app.selected_monitor = monitors[app.selected_monitor].up.unwrap();
                                    direction = Some(Dir::Up);
                                }
                            }
                            if direction.is_some() && matches!(app.state, State::MonitorSwap) {
                                app.extra_entry = 0;
                                monitor::swap_monitors(&mut monitors, app.current_monitor, app.selected_monitor, direction.unwrap(), app);
                                app.current_monitor = app.selected_monitor;
                            }
                        }
                        State::MonitorSwap => {
                            let direction: Option<Dir>;
                            let mut pivot_monitor: Option<usize> = None;
                            let mut vert_direction: Option<Dir> = None;
                            let mut swap: bool = false;
                            let mut traverse: bool = false;
                            if (key.code == KeyCode::Char('j')) | (key.code == KeyCode::Down) {
                                if monitors[app.selected_monitor].down.is_some() {
                                    app.selected_monitor = monitors[app.selected_monitor].down.unwrap();
                                    swap = true;
                                } else {
                                    traverse = monitor::traverse_monitors(&mut monitors, app.selected_monitor, Dir::Down);
                                }
                                direction = Some(Dir::Down);
                            } else {
                                if monitors[app.selected_monitor].up.is_some() {
                                    app.selected_monitor = monitors[app.selected_monitor].up.unwrap();
                                    swap = true;
                                } else {
                                    traverse = monitor::traverse_monitors(&mut monitors, app.selected_monitor, Dir::Up);
                                }
                                direction = Some(Dir::Up);
                            }
                            if swap {
                                app.extra_entry = 0;
                                monitor::swap_monitors(&mut monitors, app.current_monitor, app.selected_monitor, direction.unwrap(), app);
                                app.current_monitor = app.selected_monitor;
                            } else if !traverse {
                                //look for the left or right
                                if monitors[app.selected_monitor].left.is_some() {
                                    if direction == Some(Dir::Up) && monitors[monitors[app.selected_monitor].left.unwrap()].up.is_none()
                                        || direction == Some(Dir::Down) && monitors[monitors[app.selected_monitor].left.unwrap()].down.is_none() {
                                        pivot_monitor = monitors[app.selected_monitor].left;
                                        vert_direction = Some(Dir::Left);
                                    }
                                } else if monitors[app.selected_monitor].right.is_some() {
                                    if direction == Some(Dir::Up) && monitors[monitors[app.selected_monitor].right.unwrap()].up.is_none()
                                        || direction == Some(Dir::Down) && monitors[monitors[app.selected_monitor].right.unwrap()].down.is_none() {
                                        pivot_monitor = monitors[app.selected_monitor].right;
                                        vert_direction = Some(Dir::Right);
                                    }
                                }
                                if pivot_monitor.is_some() {
                                    monitor::vert_push(&mut monitors, pivot_monitor.unwrap(), vert_direction.unwrap(), direction.unwrap(), app);
                                }
                            }
                        }
                        State::MenuSelect => {
                            if (key.code == KeyCode::Char('j')) | (key.code == KeyCode::Down) {
                                app.menu_entry = app.get_next_menu_item();
                            } else {
                                app.menu_entry = app.get_prev_menu_item();
                            }
                            app.extra_entry = 0;
                        }
                        State::InfoEdit => {
                            if (key.code == KeyCode::Char('j')) | (key.code == KeyCode::Down) {
                                let max_length = if app.menu_entry == MenuEntry::Framerate {
                                    monitors[app.selected_monitor].available_resolutions.get(&monitors[app.selected_monitor].resolution).expect("No available framerates").len() - 1
                                } else {
                                    monitors[app.selected_monitor].available_resolutions.keys().len() - 1
                                };
                                if app.extra_entry < max_length { app.extra_entry += 1; }
                            } else {
                                if app.extra_entry > 0 { app.extra_entry -= 1; }
                            }
                        }
                        _ => {} // unimplemented
                    }
                }
                // selection
                KeyCode::Enter => {
                    match app.state {
                        State::MonitorEdit | State::MonitorSwap => {
                            if monitors[app.current_monitor].is_selected && matches!(app.state, State::MonitorEdit) {
                                monitors[app.current_monitor].is_selected = false;
                            } else {
                                app.current_monitor = app.selected_monitor;
                                monitors[app.selected_monitor].is_selected = true;
                            }
                            app.update_state(State::MenuSelect);
                            app.focused_window = FocusedWindow::MonitorInfo;
                        }
                        State::MenuSelect => {
                            if matches!(app.menu_entry, MenuEntry::Framerate | MenuEntry::Resolution) { app.update_state(State::InfoEdit); }
                        }
                        State::InfoEdit => {
                            assert!(matches!(app.menu_entry, MenuEntry::Framerate | MenuEntry::Resolution), "Editing something that's not Framerate or resolution!");
                            if matches!(app.menu_entry, MenuEntry::Resolution) {
                                let old_res = monitors[app.selected_monitor].displayed_resolution;
                                let new_res = monitors[app.selected_monitor].sort_resolutions()[app.extra_entry];
                                let difference = (new_res.0 - old_res.0, new_res.1 - old_res.1);
                                monitor::shift_res(&mut monitors, app.current_monitor, difference);

                                let updated_res = monitors[app.selected_monitor].sort_resolutions()[app.extra_entry];
                                monitors[app.selected_monitor].resolution = *updated_res;
                                let updated_res = monitors[app.selected_monitor].sort_resolutions()[app.extra_entry];
                                monitors[app.selected_monitor].displayed_resolution = *updated_res;
                                monitors[app.selected_monitor].set_framerate(0);
                                monitors[app.selected_monitor].scale = 1.0;
                            } else {
                                monitors[app.selected_monitor].set_framerate(app.extra_entry);
                            }
                        }
                        State::DebugPopup => {
                            app.update_state(app.previous_state);
                        }
                    }
                }
                // move
                KeyCode::Char('m') => {
                    if matches!(app.state, State::MonitorEdit | State::MenuSelect) {
                        if matches!(app.state, State::MonitorEdit) {
                            monitors[app.selected_monitor].is_selected = true;
                            app.current_monitor = app.selected_monitor;
                        }
                        app.update_state(State::MonitorSwap);
                        app.focused_window = FocusedWindow::MonitorList;
                    }
                }
                // set primary
                KeyCode::Char('p') => {
                    if (matches!(app.state, State::MonitorEdit) || matches!(app.state, State::MonitorSwap)) {
                        let mut iterator = monitors.iter_mut();
                        while let Some(element) = iterator.next() {
                            element.is_primary = false;
                        }
                        monitors[app.selected_monitor].is_primary = true;
                    }
                }
                // Deselect
                KeyCode::Esc => {
                    match app.state{
                        State::MenuSelect => {
                            monitors[app.current_monitor].is_selected = false;
                            app.update_state(State::MonitorEdit);
                            app.focused_window = FocusedWindow::MonitorList;
                        }
                        State::MonitorSwap => {
                            app.focused_window = FocusedWindow::MonitorInfo;
                            if matches!(app.previous_state, State::MonitorEdit) {
                                monitors[app.current_monitor].is_selected = false;
                                app.focused_window = FocusedWindow::MonitorList;
                            }
                            app.update_state(app.previous_state.clone());
                        }
                        State::InfoEdit => {
                            app.update_state(State::MenuSelect);
                        }
                        State::DebugPopup => {
                            app.update_state(app.previous_state);
                        }
                        _ => {}
                    }
                }
                _ => {}
            }
        }
    }
}

// Generate the spans to draw extra information (e.g. framerate)
fn generate_extra_info(
    monitors: &Vec<monitor::Monitor>,
    app: App,
) -> Vec<Spans> {
    if let Some(monitor) = monitors.get(app.selected_monitor) {
        if app.menu_entry == MenuEntry::Framerate {
            if let Some(framerates) = monitor.available_resolutions.get(&monitor.resolution) {
                let framerate_spans: Vec<Spans> = framerates
                    .iter()
                    .enumerate()
                    .map(|(i, fr)| {
                        Spans::from(vec![
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
                framerate_spans
            } else {
                vec![Spans::from("No available framerates")]
            }
        } else if app.menu_entry == MenuEntry::Resolution {
            if let Some(resolutions) = Some(monitor.sort_resolutions()) {
                let resolution_spans: Vec<Spans> = resolutions
                    .iter()
                    .enumerate()
                    .map(|(i, res)| {
                        Spans::from(vec![
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
                resolution_spans
            } else {
                vec![Spans::from("No available resolutions")]
            }
        } else {
            vec![Spans::from("Nothing to see here!")]
        }
    } else {
        vec![Spans::from("Nothing to see here!")]
    }
}

// Generate the spans from monitor info
fn generate_monitor_info(
    monitors: &Vec<monitor::Monitor>,
    app: App
) -> Vec<Spans> {
    if let Some(monitor) = monitors.get(app.selected_monitor) {
        vec![
            Spans::from(vec![
                Span::styled(
                    format!("Name: {}", monitor.name),
                    if app.menu_entry == MenuEntry::Name {
                        Style::default()
                            .add_modifier(Modifier::BOLD)
                            .fg(if matches!(app.state, State::InfoEdit) {
                                    Color::LightMagenta
                                } else if matches!(app.state, State::MenuSelect) {
                                    Color::Yellow
                                } else {
                                    Color::White
                                }
                            )
                    } else {
                        Style::default()
                    }
                )
            ]),
            Spans::from(vec![
                Span::styled(
                    format!("Resolution: {}x{}", monitor.resolution.0, monitor.resolution.1),
                    if app.menu_entry == MenuEntry::Resolution {
                        Style::default()
                            .add_modifier(Modifier::BOLD)
                            .fg(if matches!(app.state, State::InfoEdit) {
                                    Color::LightMagenta
                                } else if matches!(app.state, State::MenuSelect) {
                                    Color::Yellow
                                } else {
                                    Color::White
                                }
                            )
                    } else {
                        Style::default()
                    },
                )
            ]),
            Spans::from(vec![
                Span::styled(
                    format!("Scale: {:.2}", monitor.displayed_resolution.0 as f32/monitor.resolution.0 as f32),
                    if app.menu_entry == MenuEntry::Scale {
                        Style::default()
                            .add_modifier(Modifier::BOLD)
                            .fg(if matches!(app.state, State::InfoEdit) {
                                    Color::LightMagenta
                                } else if matches!(app.state, State::MenuSelect) {
                                    Color::Yellow
                                } else {
                                    Color::White
                                }
                            )
                    } else {
                        Style::default()
                    },
                )
            ]),
            Spans::from(vec![
                Span::styled(
                    format!("Position: ({}, {})", monitor.position.0, monitor.position.1),
                    if app.menu_entry == MenuEntry::Position {
                        Style::default()
                            .add_modifier(Modifier::BOLD)
                            .fg(if matches!(app.state, State::InfoEdit) {
                                    Color::LightMagenta
                                } else if matches!(app.state, State::MenuSelect) {
                                    Color::Yellow
                                } else {
                                    Color::White
                                }
                            )
                    } else {
                        Style::default()
                    },
                )
            ]),
            Spans::from(vec![
                Span::styled(
                    format!("Primary: {}", if monitor.is_primary { "Yes" } else { "No" }),
                    if app.menu_entry == MenuEntry::Primary {
                        Style::default()
                            .add_modifier(Modifier::BOLD)
                            .fg(if matches!(app.state, State::InfoEdit) {
                                    Color::LightMagenta
                                } else if matches!(app.state, State::MenuSelect) {
                                    Color::Yellow
                                } else {
                                    Color::White
                                }
                            )
                    } else {
                        Style::default()
                    },
                )
            ]),
            Spans::from(vec![
                Span::styled(
                    format!("Framerate: {}hz", monitor.framerate),
                    if app.menu_entry == MenuEntry::Framerate {
                        Style::default()
                            .add_modifier(Modifier::BOLD)
                            .fg(if matches!(app.state, State::InfoEdit) {
                                    Color::LightMagenta
                                } else if matches!(app.state, State::MenuSelect) {
                                    Color::Yellow
                                } else {
                                    Color::White
                                }
                            )
                    } else {
                        Style::default()
                    },
                )
            ]),
            Spans::from(vec![
                Span::styled(
                    format!(
                        "left: {}",
                        if monitor.left != None {
                            monitors[monitor.left.unwrap()].name.to_string()
                        } else {
                            "None".to_string()
                        }
                    ),
                    if app.menu_entry == MenuEntry::Left {
                        Style::default()
                            .add_modifier(Modifier::BOLD)
                            .fg(if matches!(app.state, State::InfoEdit) {
                                Color::LightMagenta
                            } else if matches!(app.state, State::MenuSelect) {
                                    Color::Yellow
                                } else {
                                    Color::White
                                }
                            )
                    } else {
                        Style::default()
                    },
                )
            ]),
            Spans::from(vec![
                Span::styled(
                    format!(
                        "down: {}",
                        if monitor.down != None {
                            monitors[monitor.down.unwrap()].name.to_string()
                        } else {
                            "None".to_string()
                        }
                    ),
                    if app.menu_entry == MenuEntry::Down {
                        Style::default()
                            .add_modifier(Modifier::BOLD)
                            .fg(if matches!(app.state, State::InfoEdit) {
                                Color::LightMagenta
                            } else if matches!(app.state, State::MenuSelect) {
                                    Color::Yellow
                                } else {
                                    Color::White
                                }
                            )
                    } else {
                        Style::default()
                    },
                )
            ]),
            Spans::from(vec![
                Span::styled(
                    format!(
                        "up: {}",
                        if monitor.up != None {
                            monitors[monitor.up.unwrap()].name.to_string()
                        } else {
                            "None".to_string()
                        }
                    ),
                    if app.menu_entry == MenuEntry::Up {
                        Style::default()
                            .add_modifier(Modifier::BOLD)
                            .fg(if matches!(app.state, State::InfoEdit) {
                                    Color::LightMagenta
                                } else if matches!(app.state, State::MenuSelect) {
                                    Color::Yellow
                                } else {
                                    Color::White
                                }
                            )
                    } else {
                        Style::default()
                    },
                )
            ]),
            Spans::from(vec![
                Span::styled(
                    format!(
                        "right: {}",
                        if monitor.right != None {
                            monitors[monitor.right.unwrap()].name.to_string()
                        } else {
                            "None".to_string()
                        }
                    ),
                    if app.menu_entry == MenuEntry::Right {
                        Style::default()
                            .add_modifier(Modifier::BOLD)
                            .fg(if matches!(app.state, State::InfoEdit) {
                                Color::LightMagenta
                            } else if matches!(app.state, State::MenuSelect) {
                                    Color::Yellow
                                } else {
                                    Color::White
                                }
                            )
                    } else {
                        Style::default()
                    },
                )
            ]),
            Spans::from(vec![
                Span::styled(
                    format!("Resolutions: {:?}", monitor.available_resolutions.get(&monitor.resolution).expect("No available framerates").len()),
                    if app.menu_entry == MenuEntry::Resolutions {
                        Style::default()
                            .add_modifier(Modifier::BOLD)
                            .fg(if matches!(app.state, State::InfoEdit) {
                                    Color::LightMagenta
                                } else if matches!(app.state, State::MenuSelect) {
                                    Color::Yellow
                                } else {
                                    Color::White
                                }
                            )
                    } else {
                        Style::default()
                    },
                )
            ]),
        ]
    } else {
        vec![Spans::from("No monitor selected")]
    }
}

// Helper function to create a centered rectangle
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
fn draw_monitors<B: tui::backend::Backend>(f: &mut tui::Frame<B>, area: Rect, monitors: &[monitor::Monitor], app: App) {
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
                } else if i == app.selected_monitor {
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

