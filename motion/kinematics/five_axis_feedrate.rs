//! 5-axis feedrate scheduling.

pub struct FiveAxisFeedrate {
    max_rotary_velocity: f32,
}

impl FiveAxisFeedrate {
    pub fn new(max_rotary_velocity: f32) -> Self {
        Self { max_rotary_velocity }
    }

    pub fn schedule(&self, linear_feedrate: f32, rotary_delta: f32, segment_length: f32) -> f32 {
        let time_linear = segment_length / linear_feedrate;
        let required_rotary_velocity = rotary_delta / time_linear;

        if required_rotary_velocity > self.max_rotary_velocity {
            let new_time = rotary_delta / self.max_rotary_velocity;
            segment_length / new_time
        } else {
            linear_feedrate
        }
    }
}
