#[cfg(test)]
mod tests {
    use crossterm::event::KeyCode;
    use crate::*;
    use crate::monitor::*;
    use crate::xrandr::*;
    use crate::tui::*;

    #[test]
    // test menu navigation
    fn test_update_menu_left() {
        let mut app = App::new(State::MonitorEdit, true);
        let mut monitors = get_monitor_info(true).unwrap();
        monitor_proximity(&mut monitors);

        assert_eq!(app.current_idx, 0);
        assert_eq!(app.state, State::MonitorEdit);

        handle_key_press(KeyCode::Char('l'), &mut monitors, &mut app);

        assert_eq!(app.selected_idx, 1);

    }

    #[test]
    fn test_update_menu_right() {
        let mut app = App::new(State::MonitorEdit, true);
        let mut monitors = get_monitor_info(true).unwrap();
        monitor_proximity(&mut monitors);

        handle_key_press(KeyCode::Char('l'), &mut monitors, &mut app);
        assert_eq!(app.selected_idx, 1);
        handle_key_press(KeyCode::Char('h'), &mut monitors, &mut app);
        assert_eq!(app.selected_idx, 0);
    }

    #[test]
    fn test_states() {
        let mut app = App::new(State::MonitorEdit, true);
        let mut monitors = get_monitor_info(true).unwrap();
        monitor_proximity(&mut monitors);

        assert_eq!(app.state, State::MonitorEdit);
        handle_key_press(KeyCode::Char('m'), &mut monitors, &mut app);
        assert_eq!(app.state, State::MonitorSwap);
        handle_key_press(KeyCode::Enter, &mut monitors, &mut app);
        assert_eq!(app.state, State::MonitorEdit);
        handle_key_press(KeyCode::Enter, &mut monitors, &mut app);
        assert_eq!(app.state, State::MenuSelect);

        // navigate to resolutions
        handle_key_press(KeyCode::Char('j'), &mut monitors, &mut app);
        assert_eq!(app.menu_entry, MenuEntry::Resolution);

        handle_key_press(KeyCode::Enter, &mut monitors, &mut app);
        assert_eq!(app.state, State::InfoEdit);
        handle_key_press(KeyCode::Char('d'), &mut monitors, &mut app);
        assert_eq!(app.state, State::DebugPopup);

        // test escape is working
        handle_key_press(KeyCode::Esc, &mut monitors, &mut app);
        assert_eq!(app.state, State::InfoEdit);
        handle_key_press(KeyCode::Esc, &mut monitors, &mut app);
        assert_eq!(app.state, State::MenuSelect);
        handle_key_press(KeyCode::Char('?'), &mut monitors, &mut app);
        assert_eq!(app.state, State::HelpPopup);
        handle_key_press(KeyCode::Esc, &mut monitors, &mut app);
        assert_eq!(app.state, State::MenuSelect);
        handle_key_press(KeyCode::Esc, &mut monitors, &mut app);
        assert_eq!(app.state, State::MonitorEdit);

        // test quit
        handle_key_press(KeyCode::Char('q'), &mut monitors, &mut app);
        assert_eq!(app.state, State::Quit);
    }

    #[test]
    fn test_menu_navigation() {
        let mut app = App::new(State::MenuSelect, true);
        let mut monitors = get_monitor_info(true).unwrap();
        monitor_proximity(&mut monitors);

        assert_eq!(app.menu_entry, MenuEntry::Name);

        // make sure we don't underflow
        handle_key_press(KeyCode::Char('k'), &mut monitors, &mut app);
        assert_eq!(app.menu_entry, MenuEntry::Name);
        handle_key_press(KeyCode::Char('j'), &mut monitors, &mut app);
        assert_eq!(app.menu_entry, MenuEntry::Resolution);
        handle_key_press(KeyCode::Char('j'), &mut monitors, &mut app);
        assert_eq!(app.menu_entry, MenuEntry::Scale);
        handle_key_press(KeyCode::Char('j'), &mut monitors, &mut app);
        assert_eq!(app.menu_entry, MenuEntry::Position);
        handle_key_press(KeyCode::Char('j'), &mut monitors, &mut app);
        assert_eq!(app.menu_entry, MenuEntry::Primary);
        handle_key_press(KeyCode::Char('j'), &mut monitors, &mut app);
        assert_eq!(app.menu_entry, MenuEntry::Framerate);
        handle_key_press(KeyCode::Char('j'), &mut monitors, &mut app);
        assert_eq!(app.menu_entry, MenuEntry::Left);
        handle_key_press(KeyCode::Char('j'), &mut monitors, &mut app);
        assert_eq!(app.menu_entry, MenuEntry::Down);
        handle_key_press(KeyCode::Char('j'), &mut monitors, &mut app);
        assert_eq!(app.menu_entry, MenuEntry::Up);
        handle_key_press(KeyCode::Char('j'), &mut monitors, &mut app);
        assert_eq!(app.menu_entry, MenuEntry::Right);
        handle_key_press(KeyCode::Char('j'), &mut monitors, &mut app);
        assert_eq!(app.menu_entry, MenuEntry::Resolutions);

        // make sure we don't overflow
        handle_key_press(KeyCode::Char('j'), &mut monitors, &mut app);
        assert_eq!(app.menu_entry, MenuEntry::Resolutions);

        // test up
        handle_key_press(KeyCode::Char('k'), &mut monitors, &mut app);
        assert_eq!(app.menu_entry, MenuEntry::Right);
        handle_key_press(KeyCode::Char('k'), &mut monitors, &mut app);
        assert_eq!(app.menu_entry, MenuEntry::Up);

        // that's enough probably
    }
}
