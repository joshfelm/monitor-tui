pub mod monitor;
pub mod xrandr;
pub mod debug;
pub mod tui;

pub use monitor::Monitor;

use std::process::Output;

// shared structures
use num_derive::FromPrimitive;
use num_traits::FromPrimitive;

#[derive(PartialEq, Clone, Copy)]
pub enum Dir {
    Left,
    Right,
    Up,
    Down,
}

#[derive(Clone, PartialEq, Debug, Copy)]
pub enum State {
    MonitorEdit,
    MonitorSwap,
    MenuSelect,
    InfoEdit,
    DebugPopup,
    HelpPopup,
}

#[derive(Debug, Copy, Clone)]
pub struct App {
    state: State,
    previous_state: State,
    selected_idx: usize,
    current_monitor: usize,
    focused_window: FocusedWindow,
    menu_entry: MenuEntry,
    extra_entry: usize,
    debug: bool,
}

impl App {
    fn new(dbg: bool) -> App {
        App {
            selected_idx: 0,
            current_monitor: 0,
            focused_window: FocusedWindow::MonitorList,
            state: State::MonitorEdit,
            previous_state: State::MonitorEdit,

            menu_entry: MenuEntry::Name,
            extra_entry: 0,
            debug: dbg,
        }
    }

    fn update_state(&mut self, new_state: State) {
        self.previous_state = self.state;
        self.state = new_state;
    }

    fn get_next_menu_item(&mut self) -> MenuEntry {
        match FromPrimitive::from_u8(self.menu_entry as u8 + 1) {
            Some(entry) => entry,
            None => FromPrimitive::from_u8(MAXMENU).unwrap(),
        }
    }

    fn get_prev_menu_item(&mut self) -> MenuEntry {
        match FromPrimitive::from_i8(self.menu_entry as i8 - 1) {
            Some(entry) => entry,
            None => FromPrimitive::from_u8(0).unwrap(),
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub enum FocusedWindow {
    MonitorList,
    MonitorInfo,
}

#[derive(Debug, Copy, Clone, FromPrimitive, PartialEq)]
pub enum MenuEntry{
    Name,
    Resolution,
    Scale,
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

