#[cfg(test)]
mod tests {
    use crate::*;
    use crate::monitor::*;
    use crate::xrandr::*;
    use crate::tui::*;
    use crossterm::event::KeyCode;

    #[test]
    fn swap_right() {
        let mut app = App::new(State::MonitorSwap, true);
        let mut monitors = get_monitor_info(true).unwrap();
        let mut app_states: Vec<Monitors> = Vec::new();
        monitor_proximity(&mut monitors);

        handle_key_press(KeyCode::Char('l'), &mut monitors, &mut app, &mut app_states);
        assert_eq!(monitors[0].resolution, (1920, 1080));
        assert_eq!(monitors[0].position, (0, 0));
        assert_eq!(monitors[1].resolution, (2560, 1440));
        assert_eq!(monitors[1].position, (1920, 0));
        assert_eq!(monitors[2].resolution, (1920, 1080));
        assert_eq!(monitors[2].position, (4480, 0));

    }

    #[test]
    fn swap_left() {
        let mut app = App::new(State::MonitorSwap, true);
        let mut monitors = get_monitor_info(true).unwrap();
        let mut app_states: Vec<Monitors> = Vec::new();
        monitor_proximity(&mut monitors);

        app.current_idx = 1;
        app.selected_idx = 1;
        handle_key_press(KeyCode::Char('h'), &mut monitors, &mut app, &mut app_states);
        assert_eq!(monitors[0].resolution, (1920, 1080));
        assert_eq!(monitors[0].position, (0, 0));
        assert_eq!(monitors[1].resolution, (2560, 1440));
        assert_eq!(monitors[1].position, (1920, 0));
        assert_eq!(monitors[2].resolution, (1920, 1080));
        assert_eq!(monitors[2].position, (4480, 0));
    }

    #[test]
    fn swap_down() {
        let mut app = App::new(State::MonitorSwap, true);
        let mut monitors = get_monitor_info(true).unwrap();
        let mut app_states: Vec<Monitors> = Vec::new();

        //veritcal stack monitors
        monitors[1].position = (0,1440);
        monitors[2].position = (0,1440+1080);
        monitor_proximity(&mut monitors);

        handle_key_press(KeyCode::Char('j'), &mut monitors, &mut app, &mut app_states);
        assert_eq!(monitors[0].resolution, (1920, 1080));
        assert_eq!(monitors[0].position, (0, 0));
        assert_eq!(monitors[1].resolution, (2560, 1440));
        assert_eq!(monitors[1].position, (0, 1080));
        assert_eq!(monitors[2].resolution, (1920, 1080));
        assert_eq!(monitors[2].position, (0, 1080+1440));
    }

    #[test]
    fn swap_up() {
        let mut app = App::new(State::MonitorSwap, true);
        let mut monitors = get_monitor_info(true).unwrap();
        let mut app_states: Vec<Monitors> = Vec::new();

        //veritcal stack monitors
        monitors[1].position = (0,1440);
        monitors[2].position = (0,1440+1080);
        monitor_proximity(&mut monitors);

        app.current_idx = 1;
        app.selected_idx = 1;
        handle_key_press(KeyCode::Char('k'), &mut monitors, &mut app, &mut app_states);
        assert_eq!(monitors[0].resolution, (1920, 1080));
        assert_eq!(monitors[0].position, (0, 0));
        assert_eq!(monitors[1].resolution, (2560, 1440));
        assert_eq!(monitors[1].position, (0, 1080));
        assert_eq!(monitors[2].resolution, (1920, 1080));
        assert_eq!(monitors[2].position, (0, 1080+1440));
    }

    #[test]
    fn vert_push_up() {
        let mut app = App::new(State::MonitorSwap, true);
        let mut monitors = get_monitor_info(true).unwrap();
        let mut app_states: Vec<Monitors> = Vec::new();
        monitor_proximity(&mut monitors);

        handle_key_press(KeyCode::Char('k'), &mut monitors, &mut app, &mut app_states);
        assert_eq!(monitors[0].resolution, (2560, 1440));
        assert_eq!(monitors[0].position, (0, 0));
        assert_eq!(monitors[1].resolution, (1920, 1080));
        assert_eq!(monitors[1].position, (0, 1440));
        assert_eq!(monitors[2].resolution, (1920, 1080));
        assert_eq!(monitors[2].position, (1920, 1440));
    }

    #[test]
    fn vert_push_down() {
        let mut app = App::new(State::MonitorSwap, true);
        let mut monitors = get_monitor_info(true).unwrap();
        let mut app_states: Vec<Monitors> = Vec::new();
        monitor_proximity(&mut monitors);

        handle_key_press(KeyCode::Char('j'), &mut monitors, &mut app, &mut app_states);
        assert_eq!(monitors[0].resolution, (2560, 1440));
        assert_eq!(monitors[0].position, (0, 1080));
        assert_eq!(monitors[1].resolution, (1920, 1080));
        assert_eq!(monitors[1].position, (0, 0));
        assert_eq!(monitors[2].resolution, (1920, 1080));
        assert_eq!(monitors[2].position, (1920, 0));
    }

    #[test]
    fn vert_push_up_from_middle() {
        let mut app = App::new(State::MonitorSwap, true);
        let mut monitors = get_monitor_info(true).unwrap();
        let mut app_states: Vec<Monitors> = Vec::new();
        monitor_proximity(&mut monitors);

        app.current_idx = 1;
        app.selected_idx = 1;

        handle_key_press(KeyCode::Char('k'), &mut monitors, &mut app, &mut app_states);
        assert_eq!(monitors[0].name, "HDMI-1");
        assert_eq!(monitors[0].resolution, (2560, 1440));
        assert_eq!(monitors[0].position, (0, 1080));
        assert_eq!(monitors[1].name, "DP-1");
        assert_eq!(monitors[1].resolution, (1920, 1080));
        assert_eq!(monitors[1].position, (0, 0));
        assert_eq!(monitors[2].name, "DP-2");
        assert_eq!(monitors[2].resolution, (1920, 1080));
        assert_eq!(monitors[2].position, (2560, 1080));
    }

    #[test]
    fn vert_push_down_from_middle() {
        let mut app = App::new(State::MonitorSwap, true);
        let mut monitors = get_monitor_info(true).unwrap();
        let mut app_states: Vec<Monitors> = Vec::new();
        monitor_proximity(&mut monitors);

        app.current_idx = 1;
        app.selected_idx = 1;

        handle_key_press(KeyCode::Char('j'), &mut monitors, &mut app, &mut app_states);
        assert_eq!(monitors[0].name, "HDMI-1");
        assert_eq!(monitors[0].resolution, (2560, 1440));
        assert_eq!(monitors[0].position, (0, 0));
        assert_eq!(monitors[1].name, "DP-1");
        assert_eq!(monitors[1].resolution, (1920, 1080));
        assert_eq!(monitors[1].position, (0, 1440));
        assert_eq!(monitors[2].name, "DP-2");
        assert_eq!(monitors[2].resolution, (1920, 1080));
        assert_eq!(monitors[2].position, (2560, 0));
    }

    #[test]
    fn vert_push_with_below() {
        let mut app = App::new(State::MonitorSwap, true);
        let mut monitors = get_monitor_info(true).unwrap();
        let mut app_states: Vec<Monitors> = Vec::new();

        monitors[0].position = (1920,0);
        monitors[1].position = (0,0);
        monitors[2].position = (1920,1440);
        monitor_proximity(&mut monitors);

        app.selected_idx = 0;
        app.current_idx = 2;
        handle_key_press(KeyCode::Char('k'), &mut monitors, &mut app, &mut app_states);

        assert_eq!(monitors[0].name, "HDMI-1");
        assert_eq!(monitors[1].name, "DP-1");
        assert_eq!(monitors[2].name, "DP-2");
        assert_eq!(monitors[0].position, (0,0));
        assert_eq!(monitors[1].position, (0,1440));
        assert_eq!(monitors[2].position, (1920,1440));
    }

    #[test]
    fn vert_triangle_down_position() {
        let mut app = App::new(State::MonitorSwap, true);
        let mut monitors = get_monitor_info(true).unwrap();
        let mut app_states: Vec<Monitors> = Vec::new();
        monitor_proximity(&mut monitors);

        // we expect this to look the same with the list in a different order.
        // The order is irrelevant though, so we test for names to make sure
        handle_key_press(KeyCode::Char('j'), &mut monitors, &mut app, &mut app_states);
        handle_key_press(KeyCode::Char('k'), &mut monitors, &mut app, &mut app_states);
        handle_key_press(KeyCode::Char('h'), &mut monitors, &mut app, &mut app_states);
        assert_eq!(monitors[1].name, "HDMI-1");
        assert_eq!(monitors[1].resolution, (2560, 1440));
        assert_eq!(monitors[1].position, (0, 0));
        assert_eq!(monitors[0].name, "DP-1");
        assert_eq!(monitors[0].resolution, (1920, 1080));
        assert_eq!(monitors[0].position, (2560, 0));
        assert_eq!(monitors[2].name, "DP-2");
        assert_eq!(monitors[2].resolution, (1920, 1080));
        assert_eq!(monitors[2].position, (4480, 0));
    }

    #[test]
    fn vert_triangle_down_proximity() {
        let mut app = App::new(State::MonitorSwap, true);
        let mut monitors = get_monitor_info(true).unwrap();
        let mut app_states: Vec<Monitors> = Vec::new();
        monitor_proximity(&mut monitors);

        // we expect this to look the same with the list in a different order.
        // The order is irrelevant though, so we test for names to make sure
        handle_key_press(KeyCode::Char('j'), &mut monitors, &mut app, &mut app_states);
        handle_key_press(KeyCode::Char('k'), &mut monitors, &mut app, &mut app_states);
        handle_key_press(KeyCode::Char('h'), &mut monitors, &mut app, &mut app_states);
        assert_eq!(monitors[0].left, Some(1));
        assert_eq!(monitors[0].right, Some(2));
        assert_eq!(monitors[0].up, None);
        assert_eq!(monitors[0].down, None);

        assert_eq!(monitors[1].left, None);
        assert_eq!(monitors[1].right, Some(0));
        assert_eq!(monitors[1].up, None);
        assert_eq!(monitors[1].down, None);

        assert_eq!(monitors[2].left, Some(0));
        assert_eq!(monitors[2].right, None);
        assert_eq!(monitors[2].up, None);
        assert_eq!(monitors[2].down, None);
    }

    #[test]
    fn vert_triangle_up_position() {
        let mut app = App::new(State::MonitorSwap, true);
        let mut monitors = get_monitor_info(true).unwrap();
        let mut app_states: Vec<Monitors> = Vec::new();
        monitor_proximity(&mut monitors);

        // we expect this to look the same with the list in a different order.
        // The order is irrelevant though, so we test for names to make sure
        handle_key_press(KeyCode::Char('k'), &mut monitors, &mut app, &mut app_states);
        handle_key_press(KeyCode::Char('j'), &mut monitors, &mut app, &mut app_states);
        handle_key_press(KeyCode::Char('h'), &mut monitors, &mut app, &mut app_states);

        assert_eq!(monitors[1].name, "HDMI-1");
        assert_eq!(monitors[1].resolution, (2560, 1440));
        assert_eq!(monitors[1].position, (0, 0));
        assert_eq!(monitors[0].name, "DP-1");
        assert_eq!(monitors[0].resolution, (1920, 1080));
        assert_eq!(monitors[0].position, (2560, 0));
        assert_eq!(monitors[2].name, "DP-2");
        assert_eq!(monitors[2].resolution, (1920, 1080));
        assert_eq!(monitors[2].position, (4480, 0));

    }

    #[test]
    fn vert_triangle_up_proximity() {
        let mut app = App::new(State::MonitorSwap, true);
        let mut monitors = get_monitor_info(true).unwrap();
        let mut app_states: Vec<Monitors> = Vec::new();
        monitor_proximity(&mut monitors);

        // we expect this to look the same with the list in a different order.
        // The order is irrelevant though, so we test for names to make sure
        handle_key_press(KeyCode::Char('k'), &mut monitors, &mut app, &mut app_states);
        handle_key_press(KeyCode::Char('j'), &mut monitors, &mut app, &mut app_states);
        handle_key_press(KeyCode::Char('h'), &mut monitors, &mut app, &mut app_states);

        assert_eq!(monitors[0].left, Some(1));
        assert_eq!(monitors[0].right, Some(2));
        assert_eq!(monitors[0].up, None);
        assert_eq!(monitors[0].down, None);

        assert_eq!(monitors[1].left, None);
        assert_eq!(monitors[1].right, Some(0));
        assert_eq!(monitors[1].up, None);
        assert_eq!(monitors[1].down, None);

        assert_eq!(monitors[2].left, Some(0));
        assert_eq!(monitors[2].right, None);
        assert_eq!(monitors[2].up, None);
        assert_eq!(monitors[2].down, None);
    }

    #[test]
    fn swap_different_resolutions_vertically_left() {
        let mut monitors = get_monitor_info(true).unwrap();

        monitors[0].position = (0,0);
        monitors[1].position = (0,1440);
        monitors[2].position = (1920,1440);
        monitor_proximity(&mut monitors);

        swap_monitors(&mut monitors, 0, 1, Dir::Down);

        assert_eq!(monitors[0].name, "DP-1");
        assert_eq!(monitors[0].position, (0,0));
        assert_eq!(monitors[1].name, "HDMI-1");
        assert_eq!(monitors[1].position, (0,1080));
        assert_eq!(monitors[2].name, "DP-2");
        assert_eq!(monitors[2].position, (2560,1080));
    }

    #[test]
    fn swap_different_resolutions_vertically_right() {
        let mut monitors = get_monitor_info(true).unwrap();

        monitors[0].position = (1920,0);
        monitors[1].position = (0,1440);
        monitors[2].position = (1920,1440);
        monitor_proximity(&mut monitors);

        swap_monitors(&mut monitors, 0, 2, Dir::Down);

        assert_eq!(monitors[0].name, "DP-2");
        assert_eq!(monitors[0].position, (1920,0));
        assert_eq!(monitors[1].name, "DP-1");
        assert_eq!(monitors[1].position, (0,1080));
        assert_eq!(monitors[2].name, "HDMI-1");
        assert_eq!(monitors[2].position, (1920,1080));
    }

    #[test]
    fn swap_different_resolutions_horizontally_up() {
        let mut monitors = get_monitor_info(true).unwrap();

        monitors[0].position = (1920,0);
        monitors[1].position = (0,0);
        monitors[2].position = (0,1080);
        monitor_proximity(&mut monitors);

        swap_monitors(&mut monitors, 0, 1, Dir::Left);

        assert_eq!(monitors[0].name, "DP-1");
        assert_eq!(monitors[0].position, (2560,0));
        assert_eq!(monitors[1].name, "HDMI-1");
        assert_eq!(monitors[1].position, (0,0));
        assert_eq!(monitors[2].name, "DP-2");
        assert_eq!(monitors[2].position, (0,1440));
    }

    #[test]
    fn swap_different_resolutions_horizontally_down() {
        let mut monitors = get_monitor_info(true).unwrap();

        monitors[0].position = (1920,1080);
        monitors[1].position = (0,0);
        monitors[2].position = (0,1080);
        monitor_proximity(&mut monitors);

        swap_monitors(&mut monitors, 0, 2, Dir::Left);

        assert_eq!(monitors[0].name, "DP-2");
        assert_eq!(monitors[0].position, (2560,1080));
        assert_eq!(monitors[1].name, "DP-1");
        assert_eq!(monitors[1].position, (0,0));
        assert_eq!(monitors[2].name, "HDMI-1");
        assert_eq!(monitors[2].position, (0,1080));
    }

    #[test]
    fn swap_different_resolutions_down_adjust_left() {
        let mut monitors = get_monitor_info(true).unwrap();

        monitors[0].position = (1920,0);
        monitors[1].position = (0,1440);
        monitors[2].position = (1920,1440);
        monitor_proximity(&mut monitors);

        swap_monitors(&mut monitors, 0, 2, Dir::Down);

        assert_eq!(monitors[0].position, (1920,0));
        assert_eq!(monitors[0].name, "DP-2");
        assert_eq!(monitors[2].position, (1920,1080));
        assert_eq!(monitors[2].name, "HDMI-1");
        assert_eq!(monitors[1].position, (0,1080));
        assert_eq!(monitors[1].name, "DP-1");
    }

    #[test]
    fn swap_left_with_above() {
        let mut monitors = get_monitor_info(true).unwrap();

        monitors[0].position = (1920,1080);
        monitors[1].position = (0,1080);
        monitors[2].position = (1920,0);
        monitor_proximity(&mut monitors);

        swap_monitors(&mut monitors, 0, 1, Dir::Left);

        assert_eq!(monitors[0].name, "DP-1");
        assert_eq!(monitors[0].position, (2560,1080));
        assert_eq!(monitors[1].name, "HDMI-1");
        assert_eq!(monitors[1].position, (0,1080));
        assert_eq!(monitors[2].name, "DP-2");
        assert_eq!(monitors[2].position, (2560,0));
    }

    #[test]
    fn swap_right_with_above() {
        let mut monitors = get_monitor_info(true).unwrap();

        monitors[0].position = (1920,1080);
        monitors[1].position = (0,1080);
        monitors[2].position = (1920,0);
        monitor_proximity(&mut monitors);

        swap_monitors(&mut monitors, 1, 0, Dir::Right);

        assert_eq!(monitors[0].name, "DP-1");
        assert_eq!(monitors[0].position, (2560,1080));
        assert_eq!(monitors[1].name, "HDMI-1");
        assert_eq!(monitors[1].position, (0,1080));
        assert_eq!(monitors[2].name, "DP-2");
        assert_eq!(monitors[2].position, (2560,0));
    }

    #[test]
    fn swap_up_with_left() {
        let mut monitors = get_monitor_info(true).unwrap();

        monitors[0].position = (1920,1080);
        monitors[1].position = (0,0);
        monitors[2].position = (1920,0);
        monitor_proximity(&mut monitors);

        swap_monitors(&mut monitors, 0, 2, Dir::Up);

        assert_eq!(monitors[0].name, "DP-2");
        assert_eq!(monitors[0].position, (1920,1440));
        assert_eq!(monitors[1].name, "DP-1");
        assert_eq!(monitors[1].position, (0,0));
        assert_eq!(monitors[2].name, "HDMI-1");
        assert_eq!(monitors[2].position, (1920,0));
    }
}
