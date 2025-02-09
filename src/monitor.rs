use std::collections::HashMap;

use crate::{App, Dir};
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

pub type Monitors = Vec<Monitor>;

impl Monitor {
    fn get_framerate(&self, index: usize) -> f32 {
        return self.available_resolutions.get(&self.resolution).expect("No available framerates")[index];
    }

    pub fn set_framerate(&mut self, index: usize) {
        self.framerate = self.get_framerate(index);
    }

    pub fn get_res_difference(&self) -> (i32, i32) {
        let new_res = ((self.resolution.0 as f32 * (1.0/self.scale)) as i32, (self.resolution.1 as f32 * (1.0/self.scale)) as i32);
        let difference = (new_res.0 - self.displayed_resolution.0, new_res.1 - self.displayed_resolution.1);
        return difference;
    }

    pub fn update_scale(&mut self) {
        let new_res = ((self.resolution.0 as f32 * (1.0/self.scale)) as i32, (self.resolution.1 as f32 * (1.0/self.scale)) as i32);
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
pub fn swap_monitors(
    monitors: &mut Monitors,
    current_idx: usize,
    switch_idx: usize,
    direction: Dir,
) {
    let temp_monitor = monitors[switch_idx].clone();

    match direction {
        Dir::Right => {
            let difference = monitors[switch_idx].position.0 - (monitors[current_idx].position.0 + temp_monitor.displayed_resolution.0);
            monitors[switch_idx].position = monitors[current_idx].position;
            monitors[current_idx].position.0 += temp_monitor.displayed_resolution.0 as i32;
            shift_mons(monitors, switch_idx, difference, false, vec![switch_idx]);
        }
        Dir::Left => {
            let difference = monitors[current_idx].position.0 - (monitors[switch_idx].position.0 + monitors[current_idx].displayed_resolution.0);
            monitors[switch_idx].position.0 += monitors[current_idx].displayed_resolution.0 as i32;
            monitors[current_idx].position = temp_monitor.position;
            shift_mons(monitors, current_idx, difference, false, vec![current_idx]);
        }
        Dir::Down => {
            let difference = monitors[current_idx].resolution.1 - monitors[switch_idx].resolution.1;
            monitors[switch_idx].position = monitors[current_idx].position;
            monitors[current_idx].position.1 += temp_monitor.displayed_resolution.1 as i32;
            if difference != 0 && (monitors[current_idx].position.1 == 0 || monitors[switch_idx].position.1 == 0) {
                shift_mons(monitors, switch_idx, difference, true, vec![switch_idx]);
            }
        }
        Dir::Up => {
            let difference = monitors[switch_idx].resolution.1 - monitors[current_idx].resolution.1;
            monitors[switch_idx].position.1 += monitors[current_idx].displayed_resolution.1 as i32;
            monitors[current_idx].position = temp_monitor.position;
            if difference != 0 && (monitors[current_idx].position.1 == 0 || monitors[switch_idx].position.1 == 0) {
                shift_mons(monitors, current_idx, difference, true, vec![current_idx]);
            }
        }
    }

    monitors[switch_idx].left = monitors[current_idx].left;
    monitors[switch_idx].right = monitors[current_idx].right;
    monitors[switch_idx].up = monitors[current_idx].up;
    monitors[switch_idx].down = monitors[current_idx].down;

    monitors[current_idx].left = temp_monitor.left;
    monitors[current_idx].right = temp_monitor.right;
    monitors[current_idx].up = temp_monitor.up;
    monitors[current_idx].down = temp_monitor.down;

    // update order
    monitors.swap(switch_idx, current_idx);

    update_neighbor_positions(monitors);
}

pub fn update_neighbor_positions(monitors: &mut Monitors) {
    for i in 0..monitors.len() {
        let right = monitors[i].right;
        let down = monitors[i].down;
        if let Some(right_index) = right {
            let pos_x = monitors[i].position.0 + monitors[i].displayed_resolution.0;
            monitors[right_index].position.0 = pos_x;
            monitors[right_index].position.1 = monitors[i].position.1;
        }
        if let Some(down_index) = down {
            let pos_y = monitors[i].position.1 + monitors[i].displayed_resolution.1;
            monitors[down_index].position.1 = pos_y;
            monitors[down_index].position.0 = monitors[i].position.0;
        }
    }
    monitor_proximity(monitors);
}

// shift monitor specifically when resolution changes
pub fn shift_res(monitors: &mut Monitors, mon_index: usize, difference: (i32, i32)) {
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
// - If shifting horizontally, work right (ignore any connected to the left, since they won't need
//      to shift)
// - If shifting vertically, work downwards (ignore any connected above for the same reason)
pub fn shift_mons(monitors: &mut Monitors, current_idx: usize, difference: i32, vertical: bool, mut searched_mons: Vec<usize>) -> Vec<usize> {
    if !searched_mons.contains(&current_idx) {
        if vertical {
            monitors[current_idx].position.1 -= difference;
        } else {
            monitors[current_idx].position.0 -= difference;
        }
    }

    searched_mons.push(current_idx);
    if monitors[current_idx].right.is_some() && !searched_mons.contains(&monitors[current_idx].right.unwrap()) {
        searched_mons = shift_mons(monitors, monitors[current_idx].right.unwrap(), difference, vertical, searched_mons)
    }
    if monitors[current_idx].left.is_some() && !searched_mons.contains(&monitors[current_idx].left.unwrap()) && vertical {
        searched_mons = shift_mons(monitors, monitors[current_idx].left.unwrap(), difference, vertical, searched_mons)
    }
    if monitors[current_idx].up.is_some() && !searched_mons.contains(&monitors[current_idx].up.unwrap()) && !vertical {
        searched_mons = shift_mons(monitors, monitors[current_idx].up.unwrap(), difference, vertical, searched_mons)
    }
    if monitors[current_idx].down.is_some() && !searched_mons.contains(&monitors[current_idx].down.unwrap()) {
        searched_mons = shift_mons(monitors, monitors[current_idx].down.unwrap(), difference, vertical, searched_mons)
    }
    return searched_mons;
}

// when moving up or down, and need to turn a horizontal stack into a vertical one
pub fn vert_push(monitors: &mut Monitors, pivot_idx: usize, dir: Dir, vert_dir: Dir, app:App) {
    if dir == Dir::Left {
        monitors[app.selected_idx].left = None;
        if monitors[app.selected_idx].right.is_some() {
            let difference = monitors[monitors[app.selected_idx].right.unwrap()].position.0 - monitors[app.selected_idx].displayed_resolution.0;
            shift_mons(monitors, monitors[app.selected_idx].right.unwrap(), difference, false, Vec::new());
        }
        monitors[pivot_idx].right = monitors[app.selected_idx].right;
        monitors[app.selected_idx].right = None;
    } else if dir == Dir::Right {
        monitors[app.selected_idx].right = None;
        if monitors[app.selected_idx].left.is_some() {
            let difference = monitors[monitors[app.selected_idx].left.unwrap()].position.0 - monitors[app.selected_idx].displayed_resolution.0;
            shift_mons(monitors, monitors[app.selected_idx].left.unwrap(), difference, false, Vec::new());
        }
        monitors[pivot_idx].left = monitors[app.selected_idx].left;
        monitors[app.selected_idx].left = None;
    }
    if monitors[pivot_idx].position.0 > monitors[app.selected_idx].position.0 {
        let difference = monitors[pivot_idx].position.0 - monitors[app.selected_idx].position.0;
        shift_mons(monitors, pivot_idx, difference, false, Vec::new());
    }
    if vert_dir == Dir::Down {
        monitors[app.selected_idx].position = (monitors[pivot_idx].position.0, monitors[pivot_idx].position.1 + monitors[pivot_idx].displayed_resolution.1);
        monitors[pivot_idx].down = Some(app.selected_idx);
        monitors[app.selected_idx].up = Some(pivot_idx);
    } else if vert_dir == Dir::Up {
        let new_pos_1 = monitors[pivot_idx].position.1 - monitors[pivot_idx].displayed_resolution.1;
        if new_pos_1 < 0 {
            let difference = monitors[pivot_idx].position.1 - monitors[app.selected_idx].displayed_resolution.1;
            shift_mons(monitors, pivot_idx, difference, true, Vec::new());
        }
        monitors[app.selected_idx].position = (monitors[pivot_idx].position.0, monitors[pivot_idx].position.1 - monitors[app.selected_idx].displayed_resolution.1);
    }
    monitor_proximity(monitors);

    update_neighbor_positions(monitors);
}

// when moving left or right, and need to turn a vertical stack into a horizontal one
pub fn horizontal_push(monitors: &mut Monitors, pivot_idx: usize, dir: Dir, vert_dir: Dir, app:App) {
    if dir == Dir::Up {
        monitors[pivot_idx].down = monitors[app.selected_idx].down;
        monitors[app.selected_idx].up = None;
    } else if dir == Dir::Down {
        monitors[pivot_idx].up = monitors[app.selected_idx].up;
        monitors[app.selected_idx].down = None;
    }
    if monitors[pivot_idx].position.1 > monitors[app.selected_idx].position.1 {
        let difference = monitors[pivot_idx].position.1 - cmp::min(monitors[app.selected_idx].position.1, monitors[pivot_idx].position.1);
        shift_mons(monitors, pivot_idx, difference, true, Vec::new());
    }
    if vert_dir == Dir::Right {
        monitors[app.selected_idx].position = (monitors[pivot_idx].position.0 + monitors[pivot_idx].displayed_resolution.0, monitors[pivot_idx].position.1);
        if let Some(left) = monitors[app.selected_idx].left {
            monitors[pivot_idx].left = Some(left);
            monitors[left].right = Some(pivot_idx);
        }
        monitors[pivot_idx].right = Some(app.selected_idx);
        monitors[app.selected_idx].left = Some(pivot_idx);
    } else if vert_dir == Dir::Left {
        let new_pos_1 = monitors[pivot_idx].position.0 - monitors[pivot_idx].displayed_resolution.0;
        if new_pos_1 < 0 {
            let difference = monitors[pivot_idx].position.0 - monitors[app.selected_idx].displayed_resolution.0;
            shift_mons(monitors, pivot_idx, difference, false, Vec::new());
        }
        monitors[app.selected_idx].position = (monitors[pivot_idx].position.0 - monitors[app.selected_idx].displayed_resolution.0, monitors[pivot_idx].position.1);
        let right = monitors[app.selected_idx].right;
        if right.is_some() {
            monitors[pivot_idx].right = monitors[app.selected_idx].right;
            monitors[right.unwrap()].left = Some(pivot_idx);
        }
        monitors[pivot_idx].left = Some(app.selected_idx);
        monitors[app.selected_idx].right = Some(pivot_idx);
    }

    update_neighbor_positions(monitors);
}

// recalculate cardinal proximity
pub fn monitor_proximity(monitors: &mut Monitors) {
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

pub fn traverse_monitors(monitors: &mut Monitors, selected_idx: usize, direction: Dir) -> bool {
    let mut traverse: bool = false;
    match direction {
        Dir::Right | Dir::Left => {
            let neighbour: Option<usize>;
            if let Some(traverse_idx) = monitors[selected_idx].down {
                if matches!(direction, Dir::Right) {
                    neighbour = monitors[traverse_idx].right;
                } else {
                    neighbour = monitors[traverse_idx].left;
                }
                if let Some(neighbour_idx) = neighbour {
                    monitors[neighbour_idx].up = Some(selected_idx);
                    monitors[selected_idx].down = Some(neighbour_idx);
                    monitors[selected_idx].position.0 = monitors[neighbour_idx].position.0;
                    monitors[selected_idx].position.1 = monitors[neighbour_idx].position.1 - monitors[selected_idx].displayed_resolution.1;
                    monitors[traverse_idx].up = None;
                    traverse = true;
                }
            } else if let Some(traverse_idx) = monitors[selected_idx].up {
                if matches!(direction, Dir::Right) {
                    neighbour = monitors[traverse_idx].right;
                } else {
                    neighbour = monitors[traverse_idx].left;
                }
                if let Some(neighbour_idx) = neighbour {
                    monitors[neighbour_idx].down = Some(selected_idx);
                    monitors[selected_idx].up = Some(neighbour_idx);
                    monitors[selected_idx].position.0 = monitors[neighbour_idx].position.0;
                    monitors[selected_idx].position.1 = monitors[neighbour_idx].position.1 + monitors[neighbour_idx].displayed_resolution.1;
                    monitors[traverse_idx].down = None;
                    traverse = true;
                }
            }
        }
        Dir::Up | Dir::Down => {
            let neighbour: Option<usize>;
            if let Some(traverse_idx) = monitors[selected_idx].right {
                if matches!(direction, Dir::Up) {
                    neighbour = monitors[traverse_idx].up;
                } else {
                    neighbour = monitors[traverse_idx].down;
                }
                if let Some(neighbour_idx) = neighbour {
                    monitors[neighbour_idx].left = Some(selected_idx);
                    monitors[selected_idx].right = Some(neighbour_idx);
                    monitors[selected_idx].position.0 = monitors[neighbour_idx].position.0 - monitors[selected_idx].displayed_resolution.0;
                    monitors[selected_idx].position.1 = monitors[neighbour_idx].position.1;
                    monitors[traverse_idx].left = None;
                    traverse = true;
                }
            } else if let Some(traverse_idx) = monitors[selected_idx].left {
                if matches!(direction, Dir::Up) {
                    neighbour = monitors[traverse_idx].up;
                } else {
                    neighbour = monitors[traverse_idx].down;
                }
                if let Some(neighbour_idx) = neighbour {
                    monitors[neighbour_idx].right = Some(selected_idx);
                    monitors[selected_idx].left = Some(neighbour_idx);
                    monitors[selected_idx].position.0 = monitors[neighbour_idx].position.0 + monitors[neighbour_idx].displayed_resolution.0;
                    monitors[selected_idx].position.1 = monitors[neighbour_idx].position.1;
                    monitors[traverse_idx].right = None;
                    traverse = true;
                }
            }
        }
    }
    return traverse;
}
