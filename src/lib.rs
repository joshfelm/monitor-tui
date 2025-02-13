pub mod monitor;
pub mod xrandr;
pub mod debug;
pub mod tui;

#[cfg(test)]
mod tests;

pub use monitor::Monitor;

// shared structures
use num_derive::FromPrimitive;
use num_traits::FromPrimitive;

#[derive(Debug, PartialEq, Clone, Copy)]
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
    ConnectionPopup,
    Quit,
}

#[derive(Debug, Copy, Clone)]
pub struct App {
    pub state: State,
    pub previous_state: State,
    pub selected_idx: usize,
    pub current_idx: usize,
    pub focused_window: FocusedWindow,
    pub menu_entry: MenuEntry,
    pub extra_entry: usize,
    pub debug: bool,
}

impl App {
    pub fn new(start_state: State, dbg: bool) -> App {
        App {
            selected_idx: 0,
            current_idx: 0,
            focused_window: FocusedWindow::MonitorList,
            state: start_state,
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
const MAXMENU: u8 = 10; // update this when adding to menu

