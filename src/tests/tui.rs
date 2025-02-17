use crossterm::event::KeyCode;
use crate::*;
use crate::monitor::*;
use crate::xrandr::*;
use crate::tui::*;

// test menu navigation
mod menu {
    use super::*;
    #[test]
    fn test_update_menu_left() {
        let mut app = App::new(State::MonitorEdit, true);
        let mut monitors = get_monitor_info(true).unwrap();
        let mut app_states: Vec<Monitors> = Vec::new();
        monitor_proximity(&mut monitors);

        assert_eq!(app.current_idx, 0);
        assert_eq!(app.state, State::MonitorEdit);

        handle_key_press(KeyCode::Char('l'), &mut monitors, &mut app, &mut app_states);

        assert_eq!(app.selected_idx, 1);
    }

    #[test]
    fn test_update_menu_right() {
        let mut app = App::new(State::MonitorEdit, true);
        let mut monitors = get_monitor_info(true).unwrap();
        let mut app_states: Vec<Monitors> = Vec::new();
        monitor_proximity(&mut monitors);

        handle_key_press(KeyCode::Char('l'), &mut monitors, &mut app, &mut app_states);
        assert_eq!(app.selected_idx, 1);
        handle_key_press(KeyCode::Char('h'), &mut monitors, &mut app, &mut app_states);
        assert_eq!(app.selected_idx, 0);
    }

    #[test]
    fn select_monitor_sets_menu_select_state() {
        let mut app = App::new(State::MonitorEdit, true);
        let mut monitors = get_monitor_info(true).unwrap();
        let mut app_states: Vec<Monitors> = Vec::new();
        handle_key_press(KeyCode::Enter, &mut monitors, &mut app, &mut app_states);
        assert_eq!(app.state, State::MenuSelect);
    }

    #[test]
    fn navigate_monitor_menu_to_resolution() {
        let mut app = App::new(State::MonitorEdit, true);
        let mut monitors = get_monitor_info(true).unwrap();
        let mut app_states: Vec<Monitors> = Vec::new();
        handle_key_press(KeyCode::Enter, &mut monitors, &mut app, &mut app_states);
        // navigate to resolutions
        handle_key_press(KeyCode::Char('j'), &mut monitors, &mut app, &mut app_states);
        assert_eq!(app.state, State::MenuSelect);
        assert_eq!(app.menu_entry, MenuEntry::Resolution);
    }
    #[test]
    fn menu_shoudnt_underflow() {
        let mut app = App::new(State::MenuSelect, true);
        let mut monitors = get_monitor_info(true).unwrap();
        let mut app_states: Vec<Monitors> = Vec::new();
        monitor_proximity(&mut monitors);

        assert_eq!(app.menu_entry, MenuEntry::Position);

        // make sure we don't underflow
        handle_key_press(KeyCode::Char('k'), &mut monitors, &mut app, &mut app_states);
        assert_eq!(app.menu_entry, MenuEntry::Position);
    }

    #[test]
    fn menu_shouldnt_overflow() {
        let mut app = App::new(State::MenuSelect, true);
        let mut monitors = get_monitor_info(true).unwrap();
        let mut app_states: Vec<Monitors> = Vec::new();
        monitor_proximity(&mut monitors);

        assert_eq!(app.menu_entry, MenuEntry::Position);

        app.menu_entry = MenuEntry::Resolutions;

        // make sure we don't overflow
        handle_key_press(KeyCode::Char('j'), &mut monitors, &mut app, &mut app_states);
        assert_eq!(app.menu_entry, MenuEntry::Resolutions);
    }

    #[test]
    fn test_full_menu_navigation() {
        let mut app = App::new(State::MenuSelect, true);
        let mut monitors = get_monitor_info(true).unwrap();
        let mut app_states: Vec<Monitors> = Vec::new();
        monitor_proximity(&mut monitors);

        assert_eq!(app.menu_entry, MenuEntry::Position);

        // make sure we don't underflow
        handle_key_press(KeyCode::Char('j'), &mut monitors, &mut app, &mut app_states);
        assert_eq!(app.menu_entry, MenuEntry::Resolution);
        handle_key_press(KeyCode::Char('j'), &mut monitors, &mut app, &mut app_states);
        assert_eq!(app.menu_entry, MenuEntry::Framerate);
        handle_key_press(KeyCode::Char('j'), &mut monitors, &mut app, &mut app_states);
        assert_eq!(app.menu_entry, MenuEntry::Scale);
        handle_key_press(KeyCode::Char('j'), &mut monitors, &mut app, &mut app_states);
        assert_eq!(app.menu_entry, MenuEntry::Primary);
        handle_key_press(KeyCode::Char('j'), &mut monitors, &mut app, &mut app_states);
        assert_eq!(app.menu_entry, MenuEntry::Left);
        handle_key_press(KeyCode::Char('j'), &mut monitors, &mut app, &mut app_states);
        assert_eq!(app.menu_entry, MenuEntry::Down);
        handle_key_press(KeyCode::Char('j'), &mut monitors, &mut app, &mut app_states);
        assert_eq!(app.menu_entry, MenuEntry::Up);
        handle_key_press(KeyCode::Char('j'), &mut monitors, &mut app, &mut app_states);
        assert_eq!(app.menu_entry, MenuEntry::Right);
        handle_key_press(KeyCode::Char('j'), &mut monitors, &mut app, &mut app_states);
        assert_eq!(app.menu_entry, MenuEntry::Resolutions);

        // test up
        handle_key_press(KeyCode::Char('k'), &mut monitors, &mut app, &mut app_states);
        assert_eq!(app.menu_entry, MenuEntry::Right);
        handle_key_press(KeyCode::Char('k'), &mut monitors, &mut app, &mut app_states);
        assert_eq!(app.menu_entry, MenuEntry::Up);

        // that's enough probably
    }
}

mod state {
    use super::*;
    #[test]
    fn test_starting_state() {
        let app = App::new(State::MonitorEdit, true);
        let mut monitors = get_monitor_info(true).unwrap();
        monitor_proximity(&mut monitors);

        assert_eq!(app.state, State::MonitorEdit);
    }

    #[test]
    fn m_key_sets_monitor_swap_state() {
        let mut app = App::new(State::MonitorEdit, true);
        let mut monitors = get_monitor_info(true).unwrap();
        let mut app_states: Vec<Monitors> = Vec::new();
        monitor_proximity(&mut monitors);
        handle_key_press(KeyCode::Char('m'), &mut monitors, &mut app, &mut app_states);
        assert_eq!(app.state, State::MonitorSwap);
    }

    #[test]
    fn enter_in_monitor_swap_returns_to_monitor_edit_state() {
        let mut app = App::new(State::MonitorEdit, true);
        let mut monitors = get_monitor_info(true).unwrap();
        let mut app_states: Vec<Monitors> = Vec::new();
        handle_key_press(KeyCode::Char('m'), &mut monitors, &mut app, &mut app_states);
        handle_key_press(KeyCode::Enter, &mut monitors, &mut app, &mut app_states);
        assert_eq!(app.state, State::MonitorEdit);
    }

    #[test]
    fn navigate_to_info_edit_state() {
        let mut app = App::new(State::MonitorEdit, true);
        let mut monitors = get_monitor_info(true).unwrap();
        let mut app_states: Vec<Monitors> = Vec::new();
        handle_key_press(KeyCode::Enter, &mut monitors, &mut app, &mut app_states);
        // navigate to resolutions
        handle_key_press(KeyCode::Char('j'), &mut monitors, &mut app, &mut app_states);
        handle_key_press(KeyCode::Enter, &mut monitors, &mut app, &mut app_states);
        assert_eq!(app.state, State::InfoEdit);
    }
    #[test]
    fn debug_popup() {
        let mut app = App::new(State::MonitorEdit, true);
        let mut monitors = get_monitor_info(true).unwrap();
        let mut app_states: Vec<Monitors> = Vec::new();
        handle_key_press(KeyCode::Char('d'), &mut monitors, &mut app, &mut app_states);
        assert_eq!(app.state, State::DebugPopup);
    }

    #[test]
    fn esc_from_debug() {
        let mut app = App::new(State::MonitorEdit, true);
        let mut monitors = get_monitor_info(true).unwrap();
        let mut app_states: Vec<Monitors> = Vec::new();
        app.update_state(State::InfoEdit);
        handle_key_press(KeyCode::Char('d'), &mut monitors, &mut app, &mut app_states);
        handle_key_press(KeyCode::Esc, &mut monitors, &mut app, &mut app_states);
        assert_eq!(app.state, State::InfoEdit);
    }

    #[test]
    fn esc_from_info_edit() {
        let mut app = App::new(State::MonitorEdit, true);
        let mut monitors = get_monitor_info(true).unwrap();
        let mut app_states: Vec<Monitors> = Vec::new();
        app.update_state(State::InfoEdit);
        handle_key_press(KeyCode::Esc, &mut monitors, &mut app, &mut app_states);
        assert_eq!(app.state, State::MenuSelect);
    }

    #[test]
    fn debug_on_qmark() {
        let mut app = App::new(State::MonitorEdit, true);
        let mut monitors = get_monitor_info(true).unwrap();
        let mut app_states: Vec<Monitors> = Vec::new();
        handle_key_press(KeyCode::Char('?'), &mut monitors, &mut app, &mut app_states);
        assert_eq!(app.state, State::HelpPopup);
    }

    #[test]
    fn quit_on_q() {
        let mut app = App::new(State::MonitorEdit, true);
        let mut monitors = get_monitor_info(true).unwrap();
        let mut app_states: Vec<Monitors> = Vec::new();
        handle_key_press(KeyCode::Char('q'), &mut monitors, &mut app, &mut app_states);
        assert_eq!(app.state, State::Quit);
    }
}
