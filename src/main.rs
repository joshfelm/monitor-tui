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

#[derive(Debug, Clone, Copy)]
struct App {
    state: State,
    previous_state: State,
    selected_monitor: usize,
    current_monitor: usize,
    focused_window: FocusedWindow,
    menu_entry: MenuEntry,
    extra_entry: usize
}

impl App {
    fn new() -> App {
        App {
            selected_monitor: 0,
            current_monitor: 0,
            focused_window: FocusedWindow::MonitorList,
            state: State::MonitorEdit,
            previous_state: State::MonitorEdit,

            menu_entry: MenuEntry::Name,
            extra_entry: 0,
        }
    }

    fn update_state(&mut self, new_state: State) {
        self.previous_state = self.state;
        self.state = new_state;
    }
}

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

#[derive(Debug, Clone, Copy)]
enum FocusedWindow {
    MonitorList,
    MonitorInfo,
}

#[derive(PartialEq)]
enum Dir {
    Left,
    Right,
    Up,
    Down,
}

#[derive(Clone, PartialEq, Debug, Copy)]
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

fn swap_monitors(monitors: &mut Vec<Monitor>, direction: Dir, app: App) {
    assert!(app.state == State::MonitorSwap, "Tried to swap monitors when not in monitor edit state, actual state: {:?}", app.state);
    let temp_monitor = monitors[app.selected_monitor].clone();
    if direction == Dir::Right {
        monitors[app.selected_monitor].position = monitors[app.current_monitor].position;
        monitors[app.current_monitor].position.0 += temp_monitor.resolution.0 as i32;
    } else if direction == Dir::Left {
        monitors[app.selected_monitor].position.0 += monitors[app.current_monitor].resolution.0 as i32;
        monitors[app.current_monitor].position = temp_monitor.position;
    } else if direction == Dir::Down {
        monitors[app.selected_monitor].position = monitors[app.current_monitor].position;
        monitors[app.current_monitor].position.1 += temp_monitor.resolution.1 as i32;
    } else if direction == Dir::Up {
        monitors[app.selected_monitor].position.1 += monitors[app.current_monitor].resolution.1 as i32;
        monitors[app.current_monitor].position = temp_monitor.position;
    }
    monitors[app.selected_monitor].left = monitors[app.current_monitor].left;
    monitors[app.selected_monitor].right = monitors[app.current_monitor].right;
    monitors[app.selected_monitor].up = monitors[app.current_monitor].up;
    monitors[app.selected_monitor].down = monitors[app.current_monitor].down;
    monitors[app.current_monitor].left = temp_monitor.left;
    monitors[app.current_monitor].right = temp_monitor.right;
    monitors[app.current_monitor].up = temp_monitor.up;
    monitors[app.current_monitor].down = temp_monitor.down;

    // update order
    monitors.swap(app.selected_monitor, app.current_monitor);
}

// TODO: for the < 0 check, if should recursively adjust all connected monitors by the same amount
fn vert_push(monitors: &mut Vec<Monitor>, pivot_monitor: usize, dir: Dir, vert_dir: Dir, app:App) {
    if dir == Dir::Left {
        monitors[pivot_monitor].right = None;
        monitors[app.selected_monitor].left = None;
    } else if dir == Dir::Right {
        monitors[pivot_monitor].left = None;
        monitors[app.selected_monitor].right = None;
    }
    if monitors[pivot_monitor].position.0 > monitors[app.selected_monitor].position.0 {
        monitors[pivot_monitor].position.0 = monitors[app.selected_monitor].position.0
    }
    if vert_dir == Dir::Down {
        monitors[app.selected_monitor].position = (monitors[pivot_monitor].position.0, monitors[pivot_monitor].position.1 + monitors[pivot_monitor].resolution.1);
        monitors[pivot_monitor].down = Some(app.selected_monitor);
        monitors[app.selected_monitor].up = Some(pivot_monitor);
    } else if vert_dir == Dir::Up {
        let new_pos_1 = monitors[pivot_monitor].position.1 - monitors[pivot_monitor].resolution.1;
        if new_pos_1 < 0 {
            monitors[pivot_monitor].position.1 = monitors[app.selected_monitor].resolution.1;
        }
        monitors[app.selected_monitor].position = (monitors[pivot_monitor].position.0, monitors[pivot_monitor].position.1 - monitors[app.selected_monitor].resolution.1);
        monitors[pivot_monitor].up = Some(app.selected_monitor);
        monitors[app.selected_monitor].down = Some(pivot_monitor);
    }
}

fn horizontal_push(monitors: &mut Vec<Monitor>, pivot_monitor: usize, dir: Dir, vert_dir: Dir, app:App) {
    if dir == Dir::Up {
        monitors[pivot_monitor].down = None;
        monitors[app.selected_monitor].up = None;
    } else if dir == Dir::Down {
        monitors[pivot_monitor].up = None;
        monitors[app.selected_monitor].down = None;
    }
    if monitors[pivot_monitor].position.1 > monitors[app.selected_monitor].position.1 {
        monitors[pivot_monitor].position.1 = monitors[app.selected_monitor].position.1
    }
    if vert_dir == Dir::Right {
        monitors[app.selected_monitor].position = (monitors[pivot_monitor].position.0 + monitors[pivot_monitor].resolution.0, monitors[pivot_monitor].position.1);
        monitors[pivot_monitor].right = Some(app.selected_monitor);
        monitors[app.selected_monitor].left = Some(pivot_monitor);
    } else if vert_dir == Dir::Left {
        let new_pos_1 = monitors[pivot_monitor].position.0 - monitors[pivot_monitor].resolution.0;
        if new_pos_1 < 0 {
            monitors[pivot_monitor].position.0 = monitors[app.selected_monitor].resolution.0;
        }
        monitors[app.selected_monitor].position = (monitors[pivot_monitor].position.0 - monitors[app.selected_monitor].resolution.0, monitors[pivot_monitor].position.1);
        monitors[pivot_monitor].left = Some(app.selected_monitor);
        monitors[app.selected_monitor].right = Some(pivot_monitor);
    }
}

fn generate_extra_info(
    monitors: &Vec<Monitor>,
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

fn generate_monitor_info(
    monitors: &Vec<Monitor>,
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

fn ui<B: tui::backend::Backend>(terminal: &mut Terminal<B>, mut monitors: Vec<Monitor>) -> io::Result<()> {
    // initial setup
    let mut app = App::new();

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
                    if matches!(app.state, State::MonitorEdit) {
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
                            swap_monitors(&mut monitors, direction.unwrap(), app);
                            app.current_monitor = app.selected_monitor;
                        }
                    } else if matches!(app.state, State::MonitorSwap) {
                        let direction: Option<Dir>;
                        let mut pivot_monitor: Option<usize> = None;
                        let mut vert_direction: Option<Dir> = None;
                        let mut swap: bool = false;
                        if (key.code == KeyCode::Char('l')) | (key.code == KeyCode::Right) {
                            if monitors[app.selected_monitor].right.is_some() {
                                app.selected_monitor = monitors[app.selected_monitor].right.unwrap();
                                swap = true;
                            }
                            direction = Some(Dir::Right);
                        } else {
                            if monitors[app.selected_monitor].left.is_some() {
                                app.selected_monitor = monitors[app.selected_monitor].left.unwrap();
                                swap = true;
                            }
                            direction = Some(Dir::Left);
                        }
                        if swap {
                            app.extra_entry = 0;
                            swap_monitors(&mut monitors, direction.unwrap(), app);
                            app.current_monitor = app.selected_monitor;
                        } else {
                            //look for up or down
                            if monitors[app.selected_monitor].up.is_some() {
                                pivot_monitor = monitors[app.selected_monitor].up;
                                vert_direction = Some(Dir::Up);
                            } else if monitors[app.selected_monitor].down.is_some() {
                                pivot_monitor = monitors[app.selected_monitor].down;
                                vert_direction = Some(Dir::Down);
                            }
                            if pivot_monitor.is_some() {
                                horizontal_push(&mut monitors, pivot_monitor.unwrap(), vert_direction.unwrap(), direction.unwrap(), app);
                            }
                        }
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
                                swap_monitors(&mut monitors, direction.unwrap(), app);
                                app.current_monitor = app.selected_monitor;
                            }
                        }
                        State::MonitorSwap => {
                            let direction: Option<Dir>;
                            let mut pivot_monitor: Option<usize> = None;
                            let mut vert_direction: Option<Dir> = None;
                            let mut swap: bool = false;
                            if (key.code == KeyCode::Char('j')) | (key.code == KeyCode::Down) {
                                if monitors[app.selected_monitor].down.is_some() {
                                    app.selected_monitor = monitors[app.selected_monitor].down.unwrap();
                                    swap = true;
                                }
                                direction = Some(Dir::Down);
                            } else {
                                if monitors[app.selected_monitor].up.is_some() {
                                    app.selected_monitor = monitors[app.selected_monitor].up.unwrap();
                                    swap = true;
                                }
                                direction = Some(Dir::Up);
                            }
                            if swap {
                                app.extra_entry = 0;
                                swap_monitors(&mut monitors, direction.unwrap(), app);
                                app.current_monitor = app.selected_monitor;
                            } else {
                                //look for the left or right
                                if monitors[app.selected_monitor].left.is_some() {
                                    pivot_monitor = monitors[app.selected_monitor].left;
                                    vert_direction = Some(Dir::Left);
                                } else if monitors[app.selected_monitor].right.is_some() {
                                    pivot_monitor = monitors[app.selected_monitor].right;
                                    vert_direction = Some(Dir::Right);
                                }
                                if pivot_monitor.is_some() {
                                    vert_push(&mut monitors, pivot_monitor.unwrap(), vert_direction.unwrap(), direction.unwrap(), app);
                                }
                            }
                        }
                        State::MenuSelect => {
                            if (key.code == KeyCode::Char('j')) | (key.code == KeyCode::Down) {
                                app.menu_entry = get_next_menu_item(app.menu_entry);
                            } else {
                                app.menu_entry = get_prev_menu_item(app.menu_entry);
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
                    }
                }
                // selection
                KeyCode::Enter => {
                    match app.state {
                        State::MonitorEdit | State::MonitorSwap => {
                            if monitors[app.current_monitor].is_selected {
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
                                monitors[app.selected_monitor].resolution = *monitors[app.selected_monitor].sort_resolutions()[app.extra_entry];
                                monitors[app.selected_monitor].set_framerate(0);
                            } else {
                                monitors[app.selected_monitor].set_framerate(app.extra_entry);
                            }
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
                    if matches!(app.state, State::MenuSelect) {
                        monitors[app.current_monitor].is_selected = false;
                        app.update_state(State::MonitorEdit);
                        app.focused_window = FocusedWindow::MonitorList;
                    } else if matches!(app.state, State::MonitorSwap) {
                        app.focused_window = FocusedWindow::MonitorInfo;
                        if matches!(app.previous_state, State::MonitorEdit) {
                            monitors[app.current_monitor].is_selected = false;
                            app.focused_window = FocusedWindow::MonitorList;
                        }
                        app.update_state(app.previous_state.clone());
                    } else if matches!(app.state, State::InfoEdit) {
                        app.update_state(State::MenuSelect);
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

fn draw_monitors<B: tui::backend::Backend>(f: &mut tui::Frame<B>, area: Rect, monitors: &[Monitor], app: App) {
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

