use std::collections::HashMap;

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

