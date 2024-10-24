#[macro_use]
extern crate num_derive;
extern crate num_traits;

use num_traits::FromPrimitive;
use std::io;
use std::process::Command;
use std::collections::HashMap;
use tui::widgets::canvas::Rectangle;
use tui::{
    backend::CrosstermBackend,
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Style, Modifier},
    text::{Spans, Span},
    widgets::{Block, Borders, Paragraph, canvas::Canvas},
    Terminal,
};
use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};

#[derive(Clone, PartialEq)]
struct Monitor {
    name: String,
    resolution: (i32, i32),
    available_resolutions: HashMap<(i32, i32), Vec<f32>>,  // Resolutions with vector of framerates
    framerate: f32,
    position: (i32, i32),
    is_primary: bool,
    is_selected: bool,
    left: Option<usize>,
    right: Option<usize>,
    up: Option<usize>,
    down: Option<usize>
}

impl Monitor {
    fn get_framerate(&self, index: usize) -> f32 {
        return self.available_resolutions.get(&self.resolution).expect("No available framerates")[index];
    }

    fn set_framerate(&mut self, index: usize) {
        self.framerate = self.get_framerate(index);
    }

    fn sort_resolutions(&self) -> Vec<&(i32, i32)> {
        let mut sorted_resolutions: Vec<&(i32, i32)> = self.available_resolutions.keys().collect();
        sorted_resolutions.sort_by(|a, b| {
            // First sort by width, then by height if widths are the same
            (b.0, b.1).cmp(&(a.0, a.1))
        });
        return sorted_resolutions;
    }
}

enum FocusedWindow {
    MonitorList,
    MonitorInfo,
}

#[derive(Clone, PartialEq, Debug)]
enum State {
    MonitorEdit,
    MonitorSwap,
    MenuSelect,
    InfoEdit,
}

#[derive(Debug, Copy, Clone, FromPrimitive, PartialEq)]
enum MenuEntry{
    Name,
    Resolution,
    Position,
    Primary,
    Framerate,
    Left,
    Down,
    Up,
    Right,
    Resolutions
}
const MAXMENU: u8 = 11; // update this when adding to menu

fn get_next_menu_item(entry: MenuEntry) -> MenuEntry {
    match FromPrimitive::from_u8(entry as u8 + 1) {
        Some(entry) => entry,
        None => FromPrimitive::from_u8(MAXMENU).unwrap(),
    }
}

fn get_prev_menu_item(entry: MenuEntry) -> MenuEntry {
    match FromPrimitive::from_i8(entry as i8 - 1) {
        Some(entry) => entry,
        None => FromPrimitive::from_u8(0).unwrap(),
    }
}

fn main() -> Result<(), io::Error> {
    // Setup terminal
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    // Get monitor information
    let monitors = get_monitor_info()?;

    // Run the main loop
    let res = ui(&mut terminal, monitors);

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

    Ok(())
}

fn swap_monitors(monitors: &mut Vec<Monitor>, direction: i32, selected_index: usize, current_monitor: usize, current_state: &State) {
    assert!(*current_state == State::MonitorSwap, "Tried to swap monitors when not in monitor edit state, actual state: {:?}", current_state);
    let temp_monitor = monitors[selected_index].clone();
    if direction == 1 {
        monitors[selected_index].position = monitors[current_monitor].position;
        monitors[current_monitor].position.0 += temp_monitor.resolution.0 as i32;
    } else if direction == 2 {
        monitors[selected_index].position.0 += monitors[current_monitor].resolution.0 as i32;
        monitors[current_monitor].position = temp_monitor.position;
    } else if direction == 3 {
        monitors[selected_index].position = monitors[current_monitor].position;
        monitors[current_monitor].position.1 += temp_monitor.resolution.1 as i32;
    } else if direction == 4 {
        monitors[selected_index].position.1 += monitors[current_monitor].resolution.1 as i32;
        monitors[current_monitor].position = temp_monitor.position;
    }
    monitors[selected_index].left = monitors[current_monitor].left;
    monitors[selected_index].right = monitors[current_monitor].right;
    monitors[selected_index].up = monitors[current_monitor].up;
    monitors[selected_index].down = monitors[current_monitor].down;
    monitors[current_monitor].left = temp_monitor.left;
    monitors[current_monitor].right = temp_monitor.right;
    monitors[current_monitor].up = temp_monitor.up;
    monitors[current_monitor].down = temp_monitor.down;

    // update order
    monitors.swap(selected_index, current_monitor);
}

fn generate_extra_info(
    monitors: &Vec<Monitor>,
    selected_index: usize,
    menu_entry: MenuEntry,
    extra_entry: usize,
    current_state: State,
) -> Vec<Spans> {
    if let Some(monitor) = monitors.get(selected_index) {
        if menu_entry == MenuEntry::Framerate {
            if let Some(framerates) = monitor.available_resolutions.get(&monitor.resolution) {
                let framerate_spans: Vec<Spans> = framerates
                    .iter()
                    .enumerate()
                    .map(|(i, fr)| {
                        Spans::from(vec![
                            Span::styled(
                                format!("Option {}: {}hz", i, fr),
                                if extra_entry == i {
                                    Style::default()
                                        .add_modifier(Modifier::BOLD)
                                        .fg(if matches!(current_state, State::InfoEdit) {
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
        } else if menu_entry == MenuEntry::Resolution {
            if let Some(resolutions) = Some(monitor.sort_resolutions()) {
                let resolution_spans: Vec<Spans> = resolutions
                    .iter()
                    .enumerate()
                    .map(|(i, res)| {
                        Spans::from(vec![
                            Span::styled(
                                format!("Option {}: {}x{}", i, res.0, res.1),
                                if extra_entry == i {
                                    Style::default()
                                        .add_modifier(Modifier::BOLD)
                                        .fg(if matches!(current_state, State::InfoEdit) {
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

fn generate_monitor_info(
    monitors: &Vec<Monitor>,
    selected_index: usize,
    menu_entry: MenuEntry,
    current_state: State
) -> Vec<Spans> {
    if let Some(monitor) = monitors.get(selected_index) {
        vec![
            Spans::from(vec![
                Span::styled(
                    format!("Name: {}", monitor.name),
                    if menu_entry == MenuEntry::Name {
                        Style::default()
                            .add_modifier(Modifier::BOLD)
                            .fg(if matches!(current_state, State::InfoEdit) {
                                    Color::LightMagenta
                                } else if matches!(current_state, State::MenuSelect) {
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
                    if menu_entry == MenuEntry::Resolution {
                        Style::default()
                            .add_modifier(Modifier::BOLD)
                            .fg(if matches!(current_state, State::InfoEdit) {
                                    Color::LightMagenta
                                } else if matches!(current_state, State::MenuSelect) {
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
                    if menu_entry == MenuEntry::Position {
                        Style::default()
                            .add_modifier(Modifier::BOLD)
                            .fg(if matches!(current_state, State::InfoEdit) {
                                    Color::LightMagenta
                                } else if matches!(current_state, State::MenuSelect) {
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
                    if menu_entry == MenuEntry::Primary {
                        Style::default()
                            .add_modifier(Modifier::BOLD)
                            .fg(if matches!(current_state, State::InfoEdit) {
                                    Color::LightMagenta
                                } else if matches!(current_state, State::MenuSelect) {
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
                    if menu_entry == MenuEntry::Framerate {
                        Style::default()
                            .add_modifier(Modifier::BOLD)
                            .fg(if matches!(current_state, State::InfoEdit) {
                                    Color::LightMagenta
                                } else if matches!(current_state, State::MenuSelect) {
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
                    if menu_entry == MenuEntry::Left {
                        Style::default()
                            .add_modifier(Modifier::BOLD)
                            .fg(if matches!(current_state, State::InfoEdit) {
                                Color::LightMagenta
                            } else if matches!(current_state, State::MenuSelect) {
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
                    if menu_entry == MenuEntry::Down {
                        Style::default()
                            .add_modifier(Modifier::BOLD)
                            .fg(if matches!(current_state, State::InfoEdit) {
                                Color::LightMagenta
                            } else if matches!(current_state, State::MenuSelect) {
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
                    if menu_entry == MenuEntry::Up {
                        Style::default()
                            .add_modifier(Modifier::BOLD)
                            .fg(if matches!(current_state, State::InfoEdit) {
                                    Color::LightMagenta
                                } else if matches!(current_state, State::MenuSelect) {
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
                    if menu_entry == MenuEntry::Right {
                        Style::default()
                            .add_modifier(Modifier::BOLD)
                            .fg(if matches!(current_state, State::InfoEdit) {
                                Color::LightMagenta
                            } else if matches!(current_state, State::MenuSelect) {
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
                    if menu_entry == MenuEntry::Resolutions {
                        Style::default()
                            .add_modifier(Modifier::BOLD)
                            .fg(if matches!(current_state, State::InfoEdit) {
                                    Color::LightMagenta
                                } else if matches!(current_state, State::MenuSelect) {
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

fn ui<B: tui::backend::Backend>(terminal: &mut Terminal<B>, mut monitors: Vec<Monitor>) -> io::Result<()> {
    // initial setup
    let mut selected_index = 0;
    let mut current_monitor = 0;
    let mut focused_window = FocusedWindow::MonitorList;
    let mut current_state = State::MonitorEdit;
    let mut previous_state = State::MonitorEdit;

    let mut menu_entry = MenuEntry::Name;
    let mut extra_index: usize = 0;

    loop {
        terminal.draw(|f| {
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
                .style(Style::default().fg(if matches!(focused_window, FocusedWindow::MonitorList) {
                    if matches!(current_state, State::MonitorEdit) {
                        Color::LightMagenta
                    } else {
                        Color::Yellow
                    }
                } else {
                    Color::White
                }));

            let monitor_area = monitor_block.inner(chunks[0]);
            f.render_widget(monitor_block, chunks[0]);

            draw_monitors(f, monitor_area, &monitors, selected_index);

            let info = generate_monitor_info(&monitors, selected_index, menu_entry, current_state.clone());

            let info_block = Block::default()
                .title("Monitor Info")
                .borders(Borders::ALL)
                .style(Style::default().fg(
                if matches!(current_state, State::MenuSelect | State::InfoEdit) {
                    Color::LightMagenta
                } else if matches!(focused_window, FocusedWindow::MonitorInfo) {
                    Color::Yellow
                } else {
                    Color::White
                }));

            let info_paragraph = Paragraph::new(info)
                .block(info_block)
                .style(Style::default().fg(Color::White))
                .wrap(tui::widgets::Wrap { trim: true });

            if matches!(menu_entry, MenuEntry::Framerate | MenuEntry::Resolution) {
                let bottom_chunks = Layout::default()
                    .direction(Direction::Horizontal)
                    .constraints(
                        [
                            Constraint::Percentage(50), Constraint::Percentage(50)
                        ]
                            .as_ref())
                    .split(chunks[1]);

                let extra_info = generate_extra_info(&monitors, selected_index, menu_entry, extra_index, current_state.clone());
                let title = if matches!(menu_entry, MenuEntry::Framerate) {"Framerate"} else {"Resolution"};

                let extra_block = Block::default()
                    .title(title)
                    .borders(Borders::ALL)
                    .style(Style::default().fg(
                        if matches!(current_state, State::InfoEdit) {
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
        })?;

        if let Event::Key(key) = event::read()? {
            match key.code {
                // quit
                KeyCode::Char('q') => return Ok(()),
                // save: send to xrandr
                KeyCode::Char('s') => {
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
                    };
                    let output = Command::new("xrandr")
                        .args(args)
                        .output()?;
                    println!("{:?}", output);
                }
                // horizontal movement
                KeyCode::Char('h') | KeyCode::Char('l') | KeyCode::Left | KeyCode::Right => {
                    if matches!(current_state, State::MonitorEdit) || matches!(current_state, State::MonitorSwap) {
                        let mut direction = 0;
                        if (key.code == KeyCode::Char('l')) | (key.code == KeyCode::Right) {
                            if monitors[selected_index].right.is_some() {
                                selected_index = monitors[selected_index].right.unwrap();
                                extra_index = 0;
                                direction = 1;
                            }
                        } else {
                            if monitors[selected_index].left.is_some() {
                                selected_index = monitors[selected_index].left.unwrap();
                                extra_index = 0;
                                direction = 2;
                            }
                        }
                        if direction > 0 && matches!(current_state, State::MonitorSwap)  {
                            swap_monitors(&mut monitors, direction, selected_index, current_monitor, &current_state);
                            current_monitor = selected_index;
                        }
                    }
                }
                // vertical movement
                KeyCode::Char('j') | KeyCode::Char('k') | KeyCode::Up | KeyCode::Down => {
                    match current_state {
                        State::MonitorEdit | State::MonitorSwap => {
                            let mut direction = 0;
                            if (key.code == KeyCode::Char('j')) | (key.code == KeyCode::Down) {
                                if monitors[selected_index].down.is_some() {
                                    selected_index = monitors[selected_index].down.unwrap();
                                    extra_index = 0;
                                    direction = 3;
                                }
                            } else {
                                if monitors[selected_index].up.is_some() {
                                    selected_index = monitors[selected_index].up.unwrap();
                                    extra_index = 0;
                                    direction = 4;
                                }
                            }
                            if direction > 0 && matches!(current_state, State::MonitorSwap) {
                                swap_monitors(&mut monitors, direction, selected_index, current_monitor, &current_state);
                                current_monitor = selected_index;
                            }
                        }
                        State::MenuSelect => {
                            if (key.code == KeyCode::Char('j')) | (key.code == KeyCode::Down) {
                                menu_entry = get_next_menu_item(menu_entry);
                            } else {
                                menu_entry = get_prev_menu_item(menu_entry);
                            }
                            extra_index = 0;
                        }
                        State::InfoEdit => {
                            if (key.code == KeyCode::Char('j')) | (key.code == KeyCode::Down) {
                                let max_length = if menu_entry == MenuEntry::Framerate {
                                    monitors[selected_index].available_resolutions.get(&monitors[selected_index].resolution).expect("No available framerates").len() - 1
                                } else {
                                    monitors[selected_index].available_resolutions.keys().len() - 1
                                };
                                if extra_index < max_length { extra_index += 1; }
                            } else {
                                if extra_index > 0 { extra_index -= 1; }
                            }
                        }
                    }
                }
                // selection
                KeyCode::Enter => {
                    match current_state {
                        State::MonitorEdit | State::MonitorSwap => {
                            if monitors[current_monitor].is_selected {
                                monitors[current_monitor].is_selected = false;
                            } else {
                                current_monitor = selected_index;
                                monitors[selected_index].is_selected = true;
                            }
                            previous_state = current_state;
                            current_state = State::MenuSelect;
                            focused_window = FocusedWindow::MonitorInfo;
                        }
                        State::MenuSelect => {
                            if matches!(menu_entry, MenuEntry::Framerate | MenuEntry::Resolution) { current_state = State::InfoEdit; }
                        }
                        State::InfoEdit => {
                            assert!(matches!(menu_entry, MenuEntry::Framerate | MenuEntry::Resolution), "Editing something that's not Framerate or resolution!");
                            if matches!(menu_entry, MenuEntry::Resolution) {
                                monitors[selected_index].resolution = *monitors[selected_index].sort_resolutions()[extra_index];
                            }
                            monitors[selected_index].set_framerate(extra_index);
                        }
                    }
                }
                // move
                KeyCode::Char('m') => {
                    if matches!(current_state, State::MonitorEdit | State::MenuSelect) {
                        if matches!(current_state, State::MonitorEdit) {
                            monitors[selected_index].is_selected = true;
                            current_monitor = selected_index;
                        }
                        previous_state = current_state;
                        current_state = State::MonitorSwap;
                        focused_window = FocusedWindow::MonitorList;
                    }
                }
                // set primary
                KeyCode::Char('p') => {
                    if (matches!(current_state, State::MonitorEdit) || matches!(current_state, State::MonitorSwap)) {
                        let mut iterator = monitors.iter_mut();
                        while let Some(element) = iterator.next() {
                            element.is_primary = false;
                        }
                        monitors[selected_index].is_primary = true;
                    }
                }
                // Deselect
                KeyCode::Esc => {
                    if matches!(current_state, State::MenuSelect) {
                        monitors[current_monitor].is_selected = false;
                        current_state = State::MonitorEdit;
                        focused_window = FocusedWindow::MonitorList;
                    } else if matches!(current_state, State::MonitorSwap) {
                        focused_window = FocusedWindow::MonitorInfo;
                        if matches!(previous_state, State::MonitorEdit) {
                            monitors[current_monitor].is_selected = false;
                            focused_window = FocusedWindow::MonitorList;
                        }
                        current_state = previous_state.clone();
                    } else if matches!(current_state, State::InfoEdit) {
                        current_state = State::MenuSelect;
                    }
                }
                _ => {}
            }
        }
    }
}

// get initial monitor information from xrandr
fn get_monitor_info() -> io::Result<Vec<Monitor>> {
    let output = Command::new("xrandr")
        .arg("--query")
        .output()?;

    let stdout = String::from_utf8_lossy(&output.stdout);
    let mut selected_framerate = 0.0;
    let mut monitors: Vec<Monitor> = Vec::new();
    let mut current_monitor: Option<Monitor> = None;
    let mut current_resolutions: HashMap<(i32, i32), Vec<f32>> = HashMap::new();  // HashMap to store resolutions and their framerates

    for line in stdout.lines() {
        // include ends with mm to make sure we only get monitors that are connected AND in use
        if line.contains(" connected") && line.ends_with("mm") {
            // Push the previous monitor to the list if there was one
            if let Some(monitor) = current_monitor.take() {
                monitors.push(Monitor {
                    name: monitor.name,
                    resolution: monitor.resolution,
                    position: monitor.position,
                    is_primary: monitor.is_primary,
                    framerate: selected_framerate,
                    available_resolutions: current_resolutions,
                    is_selected: monitor.is_selected,
                    left: monitor.left,
                    right: monitor.right,
                    up: monitor.up,
                    down: monitor.down
                });
            }

            // Reset for the next monitor
            current_resolutions = HashMap::new();

            // Parse monitor name and primary status
            let parts: Vec<&str> = line.split_whitespace().collect();
            let name = parts[0].to_string();
            let is_primary = parts.contains(&"primary");
            let res_pos_part: Vec<&str> = if is_primary { parts[3].split("+").collect() } else { parts[2].split("+").collect() };
            let resolution_part = res_pos_part[0];
            let position_part = res_pos_part[1];

            let resolution: Vec<i32> = resolution_part.split('x')
                .map(|s| s.parse().unwrap_or(0))
                .collect();
            let position: Vec<i32> = position_part.split('+')
                .map(|s| s.parse().unwrap_or(0))
                .collect();
            let position2: Vec<i32> = res_pos_part[2].split('+')
                .map(|s| s.parse().unwrap_or(0))
                .collect();

            current_monitor = Some(Monitor {
                name,
                resolution: (resolution[0], resolution[1]),
                position: (position[0], position2[0]),
                framerate: selected_framerate,
                is_primary,
                available_resolutions: HashMap::new(),
                is_selected: false,
                left: None,
                right: None,
                up: None,
                down: None
            });
        } else if line.contains(" disconnected") {
            // Push the previous monitor to the list if there was one
            if let Some(monitor) = current_monitor.take() {
                monitors.push(Monitor {
                    name: monitor.name,
                    resolution: monitor.resolution,
                    position: monitor.position,
                    framerate: selected_framerate,
                    is_primary: monitor.is_primary,
                    available_resolutions: current_resolutions,
                    is_selected: monitor.is_selected,
                    left: monitor.left,
                    right: monitor.right,
                    up: monitor.up,
                    down: monitor.down
                });
            }

            // Reset for the next monitor
            current_resolutions = HashMap::new();

            current_monitor = None;
        } else if let Some(_) = current_monitor.as_mut() {
            // Parse the resolution and framerates
            let parts: Vec<&str> = line.trim().split_whitespace().collect();

            // Check if the first part is in the format of a resolution (e.g., "2560x1600")
            if let Some(res_part) = parts.get(0) {
                if res_part.contains('x') {
                    let res: Vec<i32> = res_part
                        .split('x')
                        .map(|s| s.parse().unwrap_or(0))
                        .collect();

                    if res.len() == 2 {
                        let resolution = (res[0], res[1]);

                        // Parse framerates from subsequent parts
                        let mut framerates: Vec<f32> = Vec::new();
                        for rate in parts.iter().skip(1) {
                            // Remove any trailing '+' and check for '*' to mark it as selected
                            let cleaned_rate = rate.trim_end_matches(['+'].as_ref());
                            let is_selected = cleaned_rate.ends_with('*');
                            let cleaned_rate = rate.trim_end_matches(['+','*'].as_ref());
                            if let Ok(framerate) = cleaned_rate.parse::<f32>() {
                                if is_selected {
                                    selected_framerate = framerate;
                                }
                                framerates.push(framerate);
                            }
                        }

                        // Insert the resolution and framerates into the hashmap
                        current_resolutions.entry(resolution)
                            .or_insert_with(Vec::new)
                            .extend(framerates);
                    }
                }
            }
        }
    }

    // Push the last monitor after the loop
    if let Some(monitor) = current_monitor.take() {
        monitors.push(Monitor {
            name: monitor.name,
            resolution: monitor.resolution,
            position: monitor.position,
            framerate: selected_framerate,
            is_primary: monitor.is_primary,
            available_resolutions: current_resolutions,
            is_selected: monitor.is_selected,
            left: monitor.left,
            right: monitor.right,
            up: monitor.up,
            down: monitor.down
        });
    }

    // setup proximity sensor. TODO: allow for margin of error
    for i in 0..monitors.len() {
        for j in 0..monitors.len() {
            if i == j {
                continue;
            }

            if monitors[j].position.0 == (monitors[i].position.0 + monitors[j].resolution.0) && monitors[j].position.1 == monitors[i].position.1  {
                monitors[i].right = Some(j);
                monitors[j].left = Some(i);
            } else if monitors[j].position.1  == (monitors[i].position.1 + monitors[j].resolution.1) && monitors[j].position.0 == monitors[i].position.0  {
                monitors[i].down = Some(j);
                monitors[j].up = Some(i);
            } else if monitors[j].position.0  == (monitors[i].position.0 - monitors[j].resolution.0) && monitors[j].position.1 == monitors[i].position.1  {
                monitors[i].left = Some(j);
                monitors[j].right = Some(i);
            } else if monitors[j].position.1  == (monitors[i].position.1 - monitors[j].resolution.1)  && monitors[j].position.0 == monitors[i].position.0  {
                monitors[i].up = Some(j);
                monitors[j].down = Some(i);
            }
        }
    }
    Ok(monitors)
}

fn draw_monitors<B: tui::backend::Backend>(f: &mut tui::Frame<B>, area: Rect, monitors: &[Monitor], selected_index: usize) {
    let total_width: f64 = monitors.iter().map(|m| m.position.0 + m.resolution.0 as i32).max().unwrap_or(0).into();
    let total_height: f64 = monitors.iter().map(|m| m.position.1 + m.resolution.1 as i32).max().unwrap_or(0).into();

    let scale_x = 0.9;
    let scale_y = 0.5;

    let monitor_data: Vec<_> = monitors.iter().enumerate().map(|(i, m)| {
        (i, m.position, m.resolution, m.is_selected, m.is_primary, m.name.clone())
    }).collect();

    let canvas = Canvas::default()
        .x_bounds([0.0, total_width])
        .y_bounds([0.0, total_height])
        .paint(move |ctx| {
            for (i, position, resolution, is_selected, is_primary, mut name) in monitor_data.iter().cloned() {
                let x = position.0 as f64 * scale_x + total_width * (1.0 - scale_x)/2.0;
                let y = total_height - (position.1 as f64 * scale_y + total_height * (1.0 - scale_y)/2.0);
                let width = resolution.0 as f64 * scale_x;
                let height = resolution.1 as f64 * scale_y * -1.0;

                let color = if is_selected {
                    Color::LightMagenta
                } else if i == selected_index {
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
                        name = format!("<{}*>", name);
                    } else {
                        name = format!("<{}>", name);
                    }
                } else if is_primary {
                    name = format!("{}*", name);
                }
                ctx.print(
                    x + width/2.0,
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

