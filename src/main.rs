use std::io;
use std::process::Command;
use tui::widgets::canvas::Rectangle;
use tui::{
    backend::CrosstermBackend,
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Style},
    text::Span,
    widgets::{Block, Borders, Paragraph, canvas::Canvas},
    Terminal,
};
use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};

#[derive(Clone)]
struct Monitor {
    name: String,
    resolution: (u32, u32),
    position: (i32, i32),
    resolution_string: String,
    position_string: String,
    is_primary: bool,
    is_selected: bool,
}

enum FocusedWindow {
    MonitorList,
    MonitorInfo,
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
    let res = run_app(&mut terminal, monitors);

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

fn run_app<B: tui::backend::Backend>(terminal: &mut Terminal<B>, mut monitors: Vec<Monitor>) -> io::Result<()> {
    let mut selected_index = 0;
    let mut current_monitor = 0;
    let mut focused_window = FocusedWindow::MonitorList;
    let _primary_index = 0;

    loop {
        terminal.draw(|f| {
            let chunks = Layout::default()
                .direction(Direction::Vertical)
                .constraints([Constraint::Percentage(70), Constraint::Percentage(30)].as_ref())
                .split(f.size());

            let monitor_block = Block::default()
                .title("Monitors")
                .borders(Borders::ALL)
                .style(Style::default().fg(if matches!(focused_window, FocusedWindow::MonitorList) {
                    Color::Yellow
                } else {
                    Color::White
                }));

            let monitor_area = monitor_block.inner(chunks[0]);
            f.render_widget(monitor_block, chunks[0]);

            draw_monitors(f, monitor_area, &monitors, selected_index);

            let info = if let Some(monitor) = monitors.get(selected_index) {
                format!(
                    "Name: {}\nResolution: {}x{}\nPosition: ({}, {})\nResolution string: {}\nPosition string: {}\nPrimary: {}",
                    monitor.name,
                    monitor.resolution.0,
                    monitor.resolution.1,
                    monitor.position.0,
                    monitor.position.1,
                    monitor.resolution_string,
                    monitor.position_string,
                    if monitor.is_primary { "Yes" } else { "No" }
                )
            } else {
                "No monitor selected".to_string()
            };

            let info_block = Block::default()
                .title("Monitor Info")
                .borders(Borders::ALL)
                .style(Style::default().fg(if matches!(focused_window, FocusedWindow::MonitorInfo) {
                    Color::Yellow
                } else {
                    Color::White
                }));

            let info_paragraph = Paragraph::new(info)
                .block(info_block)
                .wrap(tui::widgets::Wrap { trim: true });

            f.render_widget(info_paragraph, chunks[1]);
        })?;

        if let Event::Key(key) = event::read()? {
            match key.code {
                KeyCode::Char('q') => return Ok(()),
                KeyCode::Char('h') | KeyCode::Char('l') => {
                    if matches!(focused_window, FocusedWindow::MonitorList) {
                        if key.code == KeyCode::Char('l') {
                            selected_index = (selected_index + monitors.len() - 1) % monitors.len();
                        } else {
                            selected_index = (selected_index + 1) % monitors.len();
                        }
                    }
                }
                KeyCode::Char('j') | KeyCode::Char('k') => {
                    focused_window = match focused_window {
                        FocusedWindow::MonitorList => FocusedWindow::MonitorInfo,
                        FocusedWindow::MonitorInfo => FocusedWindow::MonitorList,
                    };
                }
                KeyCode::Enter => {
                    if matches!(focused_window, FocusedWindow::MonitorList) {
                        if monitors[current_monitor].is_selected && current_monitor != selected_index {
                            monitors[current_monitor].is_selected = false;
                            let temp_position = monitors[selected_index].position;
                            monitors[selected_index].position = monitors[current_monitor].position;
                            monitors[current_monitor].position = temp_position;
                            // update order
                            monitors.swap(selected_index, current_monitor);
                        } else {
                            current_monitor = selected_index;
                            monitors[selected_index].is_selected = !monitors[selected_index].is_selected;
                        }
                    }
                }
                _ => {}
            }
        }
    }
}

fn get_monitor_info() -> io::Result<Vec<Monitor>> {
    let output = Command::new("xrandr")
        .arg("--query")
        .output()?;

    let stdout = String::from_utf8_lossy(&output.stdout);
    let monitors: Vec<Monitor> = stdout
        .lines()
        .filter(|line| line.contains(" connected"))
        .map(|line| {
            let parts: Vec<&str> = line.split_whitespace().collect();
            let name = parts[0].to_string();
            let is_primary = parts.contains(&"primary");
            let res_pos_part: Vec<&str> = if is_primary { parts[3].split("+").collect() } else { parts[2].split("+").collect() };
            let resolution_part = res_pos_part[0];
            let position_part = res_pos_part[1];

            let resolution: Vec<u32> = resolution_part.split('x')
                .map(|s| s.parse().unwrap_or(0))
                .collect();
            let position: Vec<i32> = position_part.split('+')
                .map(|s| s.parse().unwrap_or(0))
                .collect();
            let position2: Vec<i32> = res_pos_part[2].split('+')
                .map(|s| s.parse().unwrap_or(0))
                .collect();

            Monitor {
                name,
                resolution: (resolution[0], resolution[1]),
                position: (position[0], position2[0]),
                resolution_string: resolution_part.to_string(),
                position_string: parts[2].to_string(),
                is_primary,
                is_selected: false,
            }
        })
        .collect();

    Ok(monitors)
}


fn draw_monitors<B: tui::backend::Backend>(f: &mut tui::Frame<B>, area: Rect, monitors: &[Monitor], selected_index: usize) {
    let total_width: i32 = monitors.iter().map(|m| m.position.0 + m.resolution.0 as i32).max().unwrap_or(0);
    let total_height: i32 = monitors.iter().map(|m| m.position.1 + m.resolution.1 as i32).max().unwrap_or(0);

    let scale_x = 0.9;
    let scale_y = 0.5;

    let monitor_data: Vec<_> = monitors.iter().enumerate().map(|(i, m)| {
        (i, m.position, m.resolution, m.is_selected, m.is_primary, m.name.clone())
    }).collect();

    let canvas = Canvas::default()
        .x_bounds([0.0, total_width as f64])
        .y_bounds([0.0, total_height as f64])
        .paint(move |ctx| {
            for (i, position, resolution, is_selected, is_primary, mut name) in monitor_data.iter().cloned() {
                let x = position.0 as f64 * scale_x + total_width as f64 * (1 as f64-scale_x)/2 as f64;
                let y = total_height as f64 - (position.1 as f64 * scale_y as f64 + total_height as f64 * (1 as f64-scale_y)/2 as f64);
                let width = resolution.0 as f64 * scale_x as f64;
                let height = resolution.1 as f64 * scale_y as f64 * -1 as f64;

                let color = if i == selected_index {
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
                    name = format!("<{}>", name);
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

