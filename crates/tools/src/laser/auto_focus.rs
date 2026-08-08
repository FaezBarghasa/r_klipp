use heapless::Vec;
use nalgebra::Vector2;

const GRID_SIZE: usize = 5;

pub struct AutoFocus {
    height_map: [[f32; GRID_SIZE]; GRID_SIZE],
}

impl AutoFocus {
    pub fn new() -> Self {
        Self {
            height_map: [[0.0; GRID_SIZE]; GRID_SIZE],
        }
    }

    pub fn probe_grid(&mut self /* ... HAL peripherals ... */) {
        // In a real implementation, this would move the Z-probe to each grid point
        // and record the height. For now, we'll just fill it with dummy data.
        for i in 0..GRID_SIZE {
            for j in 0..GRID_SIZE {
                self.height_map[i][j] = (i + j) as f32 * 0.1;
            }
        }
    }

    pub fn get_z_offset(&self, x: f32, y: f32) -> f32 {
        // This is a simplified bilinear interpolation.
        // A real implementation would need to handle edge cases and grid scaling.
        let x_index = (x * (GRID_SIZE - 1) as f32) as usize;
        let y_index = (y * (GRID_SIZE - 1) as f32) as usize;

        if x_index >= GRID_SIZE - 1 || y_index >= GRID_SIZE - 1 {
            return self.height_map[x_index.min(GRID_SIZE - 1)][y_index.min(GRID_SIZE - 1)];
        }

        let x_frac = x * (GRID_SIZE - 1) as f32 - x_index as f32;
        let y_frac = y * (GRID_SIZE - 1) as f32 - y_index as f32;

        let h00 = self.height_map[x_index][y_index];
        let h10 = self.height_map[x_index + 1][y_index];
        let h01 = self.height_map[x_index][y_index + 1];
        let h11 = self.height_map[x_index + 1][y_index + 1];

        let h_x1 = h00 * (1.0 - x_frac) + h10 * x_frac;
        let h_x2 = h01 * (1.0 - x_frac) + h11 * x_frac;

        h_x1 * (1.0 - y_frac) + h_x2 * y_frac
    }
}
