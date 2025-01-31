use std::collections::HashMap;

use crate::{App, Dir, State};
use std::cmp;

#[derive(Clone, PartialEq)]
pub struct Monitor {
    pub name: String,
    pub resolution: (i32, i32),                                 // Selected resolution
    pub displayed_resolution: (i32, i32),                       // Resolution used may be different due to scale
    pub available_resolutions: HashMap<(i32, i32), Vec<f32>>,   // Resolutions with vector of framerates
    pub scale: f32,
    pub framerate: f32,
    pub position: (i32, i32),
    pub is_primary: bool,
    pub is_selected: bool,
    pub left: Option<usize>,
    pub right: Option<usize>,
    pub up: Option<usize>,
    pub down: Option<usize>
}

impl Monitor {
    fn get_framerate(&self, index: usize) -> f32 {
        return self.available_resolutions.get(&self.resolution).expect("No available framerates")[index];
    }

    pub fn set_framerate(&mut self, index: usize) {
        self.framerate = self.get_framerate(index);
    }

    pub fn get_res_difference(&self) -> (i32, i32) {
        let new_res = ((self.resolution.0 as f32 * self.scale) as i32, (self.resolution.1 as f32 * self.scale) as i32);
        let difference = (new_res.0 - self.displayed_resolution.0, new_res.1 - self.displayed_resolution.1);
        return difference;
    }

    pub fn update_scale(&mut self) {
        let new_res = ((self.resolution.0 as f32 * self.scale) as i32, (self.resolution.1 as f32 * self.scale) as i32);
        self.displayed_resolution = new_res;
    }

    pub fn sort_resolutions(&self) -> Vec<&(i32, i32)> {
        let mut sorted_resolutions: Vec<&(i32, i32)> = self.available_resolutions.keys().collect();
        sorted_resolutions.sort_by(|a, b| {
            // First sort by width, then by height if widths are the same
            (b.0, b.1).cmp(&(a.0, a.1))
        });
        return sorted_resolutions;
    }
}

//swap two monitors
pub fn swap_monitors(monitors: &mut Vec<Monitor>, current_monitor: usize, switching_monitor: usize, direction: Dir, app: App) {
    assert!(app.state == State::MonitorSwap, "Tried to swap monitors when not in monitor edit state, actual state: {:?}", app.state);
    let temp_monitor = monitors[switching_monitor].clone();
    if direction == Dir::Right {
        monitors[switching_monitor].position = monitors[current_monitor].position;
        monitors[current_monitor].position.0 += temp_monitor.displayed_resolution.0 as i32;
    } else if direction == Dir::Left {
        monitors[switching_monitor].position.0 += monitors[current_monitor].displayed_resolution.0 as i32;
        monitors[current_monitor].position = temp_monitor.position;
    } else if direction == Dir::Down {
        monitors[switching_monitor].position = monitors[current_monitor].position;
        monitors[current_monitor].position.1 += temp_monitor.displayed_resolution.1 as i32;
    } else if direction == Dir::Up {
        monitors[switching_monitor].position.1 += monitors[current_monitor].displayed_resolution.1 as i32;
        monitors[current_monitor].position = temp_monitor.position;
    }
    monitors[switching_monitor].left = monitors[current_monitor].left;
    monitors[switching_monitor].right = monitors[current_monitor].right;
    monitors[switching_monitor].up = monitors[current_monitor].up;
    monitors[switching_monitor].down = monitors[current_monitor].down;
    monitors[current_monitor].left = temp_monitor.left;
    monitors[current_monitor].right = temp_monitor.right;
    monitors[current_monitor].up = temp_monitor.up;
    monitors[current_monitor].down = temp_monitor.down;

    // update order
    monitors.swap(app.selected_monitor, app.current_monitor);

    for i in 0..monitors.len() {
        let right = monitors[i].right;
        let down = monitors[i].down;
        if let Some(right_index) = right {
        let pos_x = monitors[i].position.0 + monitors[i].displayed_resolution.0;
            monitors[right_index].position.0 = pos_x;
        }
        if let Some(down_index) = down {
        let pos_y = monitors[i].position.1 + monitors[i].displayed_resolution.1;
            monitors[down_index].position.1 = pos_y;
        }
    }
}

// shift monitor specifically when resolution changes
pub fn shift_res(monitors: &mut Vec<Monitor>, mon_index: usize, difference: (i32, i32)) {
    let current_monitor = monitors[mon_index].clone();
    for m in monitors {
        if m.position.0 >= (current_monitor.position.0 + current_monitor.displayed_resolution.0) {
            m.position.0 += difference.0;
        }
        if m.position.1 >= (current_monitor.position.1 + current_monitor.displayed_resolution.1) {
            m.position.1 += difference.1;
        }
    }
}

// shift monitor and recursively shift connected monitors by a given amount
pub fn shift_mons(monitors: &mut Vec<Monitor>, current_monitor: usize, difference: i32, vertical: bool, mut searched_mons: Vec<usize>) -> Vec<usize> {
    if !searched_mons.contains(&current_monitor) {
        if vertical {
            monitors[current_monitor].position.1 -= difference;
        } else {
            monitors[current_monitor].position.0 -= difference;
        }
    }
    searched_mons.push(current_monitor);
    if monitors[current_monitor].right.is_some() && !searched_mons.contains(&monitors[current_monitor].right.unwrap()) { searched_mons = shift_mons(monitors, monitors[current_monitor].right.unwrap(), difference, vertical, searched_mons) }
    if monitors[current_monitor].left.is_some() && !searched_mons.contains(&monitors[current_monitor].left.unwrap()) { searched_mons = shift_mons(monitors, monitors[current_monitor].left.unwrap(), difference, vertical, searched_mons) }
    if monitors[current_monitor].up.is_some() && !searched_mons.contains(&monitors[current_monitor].up.unwrap()) { searched_mons = shift_mons(monitors, monitors[current_monitor].up.unwrap(), difference, vertical, searched_mons) }
    if monitors[current_monitor].down.is_some() && !searched_mons.contains(&monitors[current_monitor].down.unwrap()) { searched_mons = shift_mons(monitors, monitors[current_monitor].down.unwrap(), difference, vertical, searched_mons) }
    return searched_mons;
}

// when moving up or down, and need to turn a horizontal stack into a vertical one
pub fn vert_push(monitors: &mut Vec<Monitor>, pivot_monitor: usize, dir: Dir, vert_dir: Dir, app:App) {
    if dir == Dir::Left {
        monitors[app.selected_monitor].left = None;
        if monitors[app.selected_monitor].right.is_some() {
            let difference = monitors[monitors[app.selected_monitor].right.unwrap()].position.0 - monitors[app.selected_monitor].displayed_resolution.0;
            shift_mons(monitors, monitors[app.selected_monitor].right.unwrap(), difference, false, Vec::new());
        }
        monitors[pivot_monitor].right = monitors[app.selected_monitor].right;
        monitors[app.selected_monitor].right = None;
    } else if dir == Dir::Right {
        monitors[app.selected_monitor].right = None;
        if monitors[app.selected_monitor].left.is_some() {
            let difference = monitors[monitors[app.selected_monitor].left.unwrap()].position.0 - monitors[app.selected_monitor].displayed_resolution.0;
            shift_mons(monitors, monitors[app.selected_monitor].left.unwrap(), difference, false, Vec::new());
        }
        monitors[pivot_monitor].left = monitors[app.selected_monitor].left;
        monitors[app.selected_monitor].left = None;
    }
    if monitors[pivot_monitor].position.0 > monitors[app.selected_monitor].position.0 {
        let difference = monitors[pivot_monitor].position.0 - monitors[app.selected_monitor].position.0;
        shift_mons(monitors, pivot_monitor, difference, false, Vec::new());
    }
    if vert_dir == Dir::Down {
        monitors[app.selected_monitor].position = (monitors[pivot_monitor].position.0, monitors[pivot_monitor].position.1 + monitors[pivot_monitor].displayed_resolution.1);
        monitors[pivot_monitor].down = Some(app.selected_monitor);
        monitors[app.selected_monitor].up = Some(pivot_monitor);
    } else if vert_dir == Dir::Up {
        let new_pos_1 = monitors[pivot_monitor].position.1 - monitors[pivot_monitor].displayed_resolution.1;
        if new_pos_1 < 0 {
            let difference = monitors[pivot_monitor].position.1 - monitors[app.selected_monitor].displayed_resolution.1;
            shift_mons(monitors, pivot_monitor, difference, true, Vec::new());
        }
        monitors[app.selected_monitor].position = (monitors[pivot_monitor].position.0, monitors[pivot_monitor].position.1 - monitors[app.selected_monitor].displayed_resolution.1);
    }
    monitor_proximity(monitors);

    for i in 0..monitors.len() {
        let right = monitors[i].right;
        let down = monitors[i].down;
        if let Some(right_index) = right {
            let pos_x = monitors[i].position.0 + monitors[i].displayed_resolution.0;
            monitors[right_index].position.0 = pos_x;
        }
        if let Some(down_index) = down {
            let pos_y = monitors[i].position.1 + monitors[i].displayed_resolution.1;
            monitors[down_index].position.1 = pos_y;
        }
    }
}

// when moving left or right, and need to turn a vertical stack into a horizontal one
pub fn horizontal_push(monitors: &mut Vec<Monitor>, pivot_monitor: usize, dir: Dir, vert_dir: Dir, app:App) {
    if dir == Dir::Up {
        monitors[pivot_monitor].down = monitors[app.selected_monitor].down;
        monitors[app.selected_monitor].up = None;
    } else if dir == Dir::Down {
        monitors[pivot_monitor].up = monitors[app.selected_monitor].up;
        monitors[app.selected_monitor].down = None;
    }
    if monitors[pivot_monitor].position.1 > monitors[app.selected_monitor].position.1 {
        let difference = monitors[pivot_monitor].position.1 - cmp::min(monitors[app.selected_monitor].position.1, monitors[pivot_monitor].position.1);
        shift_mons(monitors, pivot_monitor, difference, true, Vec::new());
    }
    if vert_dir == Dir::Right {
        monitors[app.selected_monitor].position = (monitors[pivot_monitor].position.0 + monitors[pivot_monitor].displayed_resolution.0, monitors[pivot_monitor].position.1);
        let left = monitors[app.selected_monitor].left;
        if left.is_some() {
            monitors[pivot_monitor].left = left;
            monitors[left.unwrap()].right = Some(pivot_monitor);
        }
        monitors[pivot_monitor].right = Some(app.selected_monitor);
        monitors[app.selected_monitor].left = Some(pivot_monitor);
    } else if vert_dir == Dir::Left {
        let new_pos_1 = monitors[pivot_monitor].position.0 - monitors[pivot_monitor].displayed_resolution.0;
        if new_pos_1 < 0 {
            let difference = monitors[pivot_monitor].position.0 - monitors[app.selected_monitor].displayed_resolution.0;
            shift_mons(monitors, pivot_monitor, difference, false, Vec::new());
        }
        monitors[app.selected_monitor].position = (monitors[pivot_monitor].position.0 - monitors[app.selected_monitor].displayed_resolution.0, monitors[pivot_monitor].position.1);
        let right = monitors[app.selected_monitor].right;
        if right.is_some() {
            monitors[pivot_monitor].right = monitors[app.selected_monitor].right;
            monitors[right.unwrap()].left = Some(pivot_monitor);
        }
        monitors[pivot_monitor].left = Some(app.selected_monitor);
        monitors[app.selected_monitor].right = Some(pivot_monitor);
    }
    monitor_proximity(monitors);

    for i in 0..monitors.len() {
        let right = monitors[i].right;
        let down = monitors[i].down;
        if let Some(right_index) = right {
        let pos_x = monitors[i].position.0 + monitors[i].displayed_resolution.0;
            monitors[right_index].position.0 = pos_x;
        }
        if let Some(down_index) = down {
        let pos_y = monitors[i].position.1 + monitors[i].displayed_resolution.1;
            monitors[down_index].position.1 = pos_y;
        }
    }
}

// recalculate cardinal proximity
pub fn monitor_proximity(monitors: &mut Vec<Monitor>) {
    for i in 0..monitors.len() {
        for j in 0..monitors.len() {
            if i == j {
                continue;
            }

            if monitors[j].position.0 == (monitors[i].position.0 + monitors[j].displayed_resolution.0) && monitors[j].position.1 == monitors[i].position.1  {
                monitors[i].right = Some(j);
                monitors[j].left = Some(i);
            } else if monitors[j].position.1  == (monitors[i].position.1 + monitors[j].displayed_resolution.1) && monitors[j].position.0 == monitors[i].position.0  {
                monitors[i].down = Some(j);
                monitors[j].up = Some(i);
            } else if monitors[j].position.0  == (monitors[i].position.0 - monitors[j].displayed_resolution.0) && monitors[j].position.1 == monitors[i].position.1  {
                monitors[i].left = Some(j);
                monitors[j].right = Some(i);
            } else if monitors[j].position.1  == (monitors[i].position.1 - monitors[j].displayed_resolution.1)  && monitors[j].position.0 == monitors[i].position.0  {
                monitors[i].up = Some(j);
                monitors[j].down = Some(i);
            }
        }
    }
}

pub fn traverse_monitors(monitors: &mut Vec<Monitor>, selected_monitor: usize, direction: Dir) -> bool {
    let traverse_monitor: usize;
    let mut traverse: bool = false;
    match direction {
        Dir::Right => {
            let right_traverse: Option<usize>;
            if monitors[selected_monitor].down.is_some() {
                traverse_monitor = monitors[selected_monitor].down.unwrap();
                right_traverse = monitors[traverse_monitor].right;
                if right_traverse.is_some() {
                    monitors[right_traverse.unwrap()].up = Some(selected_monitor);
                    monitors[selected_monitor].down = right_traverse;
                    monitors[selected_monitor].position.0 = monitors[right_traverse.unwrap()].position.0;
                    monitors[selected_monitor].position.1 = monitors[right_traverse.unwrap()].position.1 - monitors[selected_monitor].displayed_resolution.1;
                    monitors[traverse_monitor].up = None;
                    traverse = true;
                }
            } else if monitors[selected_monitor].up.is_some() {
                traverse_monitor = monitors[selected_monitor].up.unwrap();
                right_traverse = monitors[traverse_monitor].right;
                if right_traverse.is_some() {
                    monitors[right_traverse.unwrap()].down = Some(selected_monitor);
                    monitors[selected_monitor].up = right_traverse;
                    monitors[selected_monitor].position.0 = monitors[right_traverse.unwrap()].position.0;
                    monitors[selected_monitor].position.1 = monitors[right_traverse.unwrap()].position.1 + monitors[right_traverse.unwrap()].displayed_resolution.1;
                    monitors[traverse_monitor].down = None;
                    traverse = true;
                }
            }
        }
        Dir::Left => {
            let left_traverse: Option<usize>;
            if monitors[selected_monitor].down.is_some() {
                traverse_monitor = monitors[selected_monitor].down.unwrap();
                left_traverse = monitors[traverse_monitor].left;
                if left_traverse.is_some() {
                    monitors[left_traverse.unwrap()].up = Some(selected_monitor);
                    monitors[selected_monitor].down = left_traverse;
                    monitors[selected_monitor].position.0 = monitors[left_traverse.unwrap()].position.0;
                    monitors[selected_monitor].position.1 = monitors[left_traverse.unwrap()].position.1 - monitors[selected_monitor].displayed_resolution.1;
                    monitors[traverse_monitor].up = None;
                    traverse = true;
                }
            } else if monitors[selected_monitor].up.is_some() {
                traverse_monitor = monitors[selected_monitor].up.unwrap();
                left_traverse = monitors[traverse_monitor].left;
                if left_traverse.is_some() {
                    monitors[left_traverse.unwrap()].down = Some(selected_monitor);
                    monitors[selected_monitor].up = left_traverse;
                    monitors[selected_monitor].position.0 = monitors[left_traverse.unwrap()].position.0;
                    monitors[selected_monitor].position.1 = monitors[left_traverse.unwrap()].position.1 + monitors[left_traverse.unwrap()].displayed_resolution.1;
                    monitors[traverse_monitor].down = None;
                    traverse = true;
                }
            }
        }
        Dir::Up => {
            let up_traverse;
            if monitors[selected_monitor].right.is_some() {
                traverse_monitor = monitors[selected_monitor].right.unwrap();
                up_traverse = monitors[traverse_monitor].up;
                if up_traverse.is_some() {
                    monitors[up_traverse.unwrap()].left = Some(selected_monitor);
                    monitors[selected_monitor].right = up_traverse;
                    monitors[selected_monitor].position.1 = monitors[up_traverse.unwrap()].position.1;
                    monitors[selected_monitor].position.0 = monitors[up_traverse.unwrap()].position.0 - monitors[selected_monitor].displayed_resolution.0;
                    monitors[traverse_monitor].left = None;
                    traverse = true;
                }
            } else if monitors[selected_monitor].left.is_some() {
                traverse_monitor = monitors[selected_monitor].left.unwrap();
                up_traverse = monitors[traverse_monitor].up;
                if up_traverse.is_some() {
                    monitors[up_traverse.unwrap()].right = Some(selected_monitor);
                    monitors[selected_monitor].left = up_traverse;
                    monitors[selected_monitor].position.1 = monitors[up_traverse.unwrap()].position.1;
                    monitors[selected_monitor].position.0 = monitors[up_traverse.unwrap()].position.0 + monitors[up_traverse.unwrap()].displayed_resolution.0;
                    monitors[traverse_monitor].right = None;
                    traverse = true;
                }
            }
        }
        Dir::Down => {
            let down_traverse;
            if monitors[selected_monitor].right.is_some() {
                traverse_monitor = monitors[selected_monitor].right.unwrap();
                down_traverse = monitors[traverse_monitor].down;
                if down_traverse.is_some() {
                    monitors[down_traverse.unwrap()].left = Some(selected_monitor);
                    monitors[selected_monitor].right = down_traverse;
                    monitors[selected_monitor].position.1 = monitors[down_traverse.unwrap()].position.1;
                    monitors[selected_monitor].position.0 = monitors[down_traverse.unwrap()].position.0 - monitors[selected_monitor].displayed_resolution.0;
                    monitors[traverse_monitor].left = None;
                    traverse = true;
                }
            } else if monitors[selected_monitor].left.is_some() {
                traverse_monitor = monitors[selected_monitor].left.unwrap();
                down_traverse = monitors[traverse_monitor].down;
                if down_traverse.is_some() {
                    monitors[down_traverse.unwrap()].right = Some(selected_monitor);
                    monitors[selected_monitor].left = down_traverse;
                    monitors[selected_monitor].position.1 = monitors[down_traverse.unwrap()].position.1;
                    monitors[selected_monitor].position.0 = monitors[down_traverse.unwrap()].position.0 + monitors[down_traverse.unwrap()].displayed_resolution.0;
                    monitors[traverse_monitor].right = None;
                    traverse = true;
                }
            }
        }

    }
    return traverse;
}
