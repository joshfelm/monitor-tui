
extern crate monitor_tui;

#[cfg(test)]
mod tests {
    use crossterm::event::KeyCode;
    use monitor_tui::*;
    use monitor_tui::monitor::*;
    use monitor_tui::tui::*;

    #[test]
    fn test_simple_move() {
        let mut app = App::new(State::MonitorSwap, true);
        let mut monitors = monitor_tui::xrandr::get_monitor_info(true).unwrap();
        monitor_proximity(&mut monitors);

        handle_key_press(KeyCode::Char('l'), &mut monitors, &mut app);
        assert_eq!(monitors[0].resolution, (1920, 1080));
        assert_eq!(monitors[0].position, (0, 0));
        assert_eq!(monitors[1].resolution, (2560, 1440));
        assert_eq!(monitors[1].position, (1920, 0));
        assert_eq!(monitors[2].resolution, (1920, 1080));
        assert_eq!(monitors[2].position, (4480, 0));

        handle_key_press(KeyCode::Char('l'), &mut monitors, &mut app);
        assert_eq!(monitors[0].resolution, (1920, 1080));
        assert_eq!(monitors[0].position, (0, 0));
        assert_eq!(monitors[1].resolution, (1920, 1080));
        assert_eq!(monitors[1].position, (1920, 0));
        assert_eq!(monitors[2].resolution, (2560, 1440));
        assert_eq!(monitors[2].position, (3840, 0));

        handle_key_press(KeyCode::Char('h'), &mut monitors, &mut app);
        assert_eq!(monitors[0].resolution, (1920, 1080));
        assert_eq!(monitors[0].position, (0, 0));
        assert_eq!(monitors[1].resolution, (2560, 1440));
        assert_eq!(monitors[1].position, (1920, 0));
        assert_eq!(monitors[2].resolution, (1920, 1080));
        assert_eq!(monitors[2].position, (4480, 0));
    }

    #[test]
    fn test_vert_push_up() {
        let mut app = App::new(State::MonitorSwap, true);
        let mut monitors = monitor_tui::xrandr::get_monitor_info(true).unwrap();
        monitor_proximity(&mut monitors);

        handle_key_press(KeyCode::Char('k'), &mut monitors, &mut app);
        assert_eq!(monitors[0].resolution, (2560, 1440));
        assert_eq!(monitors[0].position, (0, 0));
        assert_eq!(monitors[1].resolution, (1920, 1080));
        assert_eq!(monitors[1].position, (0, 1440));
        assert_eq!(monitors[2].resolution, (1920, 1080));
        assert_eq!(monitors[2].position, (1920, 1440));
    }

    #[test]
    fn test_vert_push_down() {
        let mut app = App::new(State::MonitorSwap, true);
        let mut monitors = monitor_tui::xrandr::get_monitor_info(true).unwrap();
        monitor_proximity(&mut monitors);

        handle_key_press(KeyCode::Char('j'), &mut monitors, &mut app);
        assert_eq!(monitors[0].resolution, (2560, 1440));
        assert_eq!(monitors[0].position, (0, 1080));
        assert_eq!(monitors[1].resolution, (1920, 1080));
        assert_eq!(monitors[1].position, (0, 0));
        assert_eq!(monitors[2].resolution, (1920, 1080));
        assert_eq!(monitors[2].position, (1920, 0));
    }

    #[test]
    fn test_vert_triangle_down() {
        let mut app = App::new(State::MonitorSwap, true);
        let mut monitors = monitor_tui::xrandr::get_monitor_info(true).unwrap();
        monitor_proximity(&mut monitors);

        // we expect this to lok the same with the list in a different order.
        // The order is irrelevant though, so we test for names to make sure
        handle_key_press(KeyCode::Char('j'), &mut monitors, &mut app);
        handle_key_press(KeyCode::Char('k'), &mut monitors, &mut app);
        handle_key_press(KeyCode::Char('h'), &mut monitors, &mut app);
        assert_eq!(monitors[1].resolution, (2560, 1440));
        assert_eq!(monitors[1].position, (0, 0));
        assert_eq!(monitors[1].name, "HDMI-1");
        assert_eq!(monitors[0].resolution, (1920, 1080));
        assert_eq!(monitors[0].position, (2560, 0));
        assert_eq!(monitors[0].name, "DP-1");
        assert_eq!(monitors[2].resolution, (1920, 1080));
        assert_eq!(monitors[2].position, (4480, 0));
        assert_eq!(monitors[2].name, "DP-2");
    }

    #[test]
    fn test_vert_triangle_up() {
        let mut app = App::new(State::MonitorSwap, true);
        let mut monitors = monitor_tui::xrandr::get_monitor_info(true).unwrap();
        monitor_proximity(&mut monitors);

        // we expect this to lok the same with the list in a different order.
        // The order is irrelevant though, so we test for names to make sure
        handle_key_press(KeyCode::Char('k'), &mut monitors, &mut app);
        assert_eq!(monitors[0].resolution, (2560, 1440));
        assert_eq!(monitors[0].position, (0, 0));
        assert_eq!(monitors[1].resolution, (1920, 1080));
        assert_eq!(monitors[1].position, (0, 1440));
        assert_eq!(monitors[2].resolution, (1920, 1080));
        assert_eq!(monitors[2].position, (1920, 1440));

        handle_key_press(KeyCode::Char('j'), &mut monitors, &mut app);
        assert_eq!(monitors[0].resolution, (1920, 1080));
        assert_eq!(monitors[0].position, (0, 0));
        assert_eq!(monitors[1].resolution, (2560, 1440));
        assert_eq!(monitors[1].position, (0, 1080));
        assert_eq!(monitors[2].resolution, (1920, 1080));
        assert_eq!(monitors[2].position, (2560, 1080));

        assert_eq!(monitors[0].down, Some(1));
        assert_eq!(monitors[0].left, None);
        assert_eq!(monitors[0].up, None);
        assert_eq!(monitors[0].right, None);

        assert_eq!(monitors[1].down, None);
        assert_eq!(monitors[1].left, None);
        assert_eq!(monitors[1].up, Some(0));
        assert_eq!(monitors[1].right, Some(2));

        assert_eq!(monitors[2].down, None);
        assert_eq!(monitors[2].left, Some(1));
        assert_eq!(monitors[2].up, None);
        assert_eq!(monitors[2].right, None);

        handle_key_press(KeyCode::Char('h'), &mut monitors, &mut app);
        assert_eq!(monitors[1].resolution, (2560, 1440));
        assert_eq!(monitors[1].position, (0, 0));
        assert_eq!(monitors[1].name, "HDMI-1");
        assert_eq!(monitors[0].resolution, (1920, 1080));
        assert_eq!(monitors[0].position, (2560, 0));
        assert_eq!(monitors[0].name, "DP-1");
        assert_eq!(monitors[2].resolution, (1920, 1080));
        assert_eq!(monitors[2].position, (4480, 0));
        assert_eq!(monitors[2].name, "DP-2");
    }
}
