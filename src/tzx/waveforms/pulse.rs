use std::sync::Arc;
use std::time::Duration;

use crate::tzx::Config;

#[derive(Clone)]
pub struct Pulse {
    pub config: Arc<Config>,
    pub length: u16,
    pub high: bool,
}

impl Pulse {
    pub fn new(config: Arc<Config>, length: u16, high: bool) -> Self {
        return Self {
            config,
            length,
            high,
        }
    }

    pub fn len(&self) -> u32 {
        return (self.length as f64 * self.config.platform.t_cycle_secs_playback(self.config.playback_duration_percent) * self.config.sample_rate as f64).round() as u32
    }

    pub fn duration(&self) -> Duration {
        Duration::from_secs_f64(self.len() as f64 / self.config.sample_rate as f64)
    }

    pub fn get_sample(&self) -> f32 {
        return if self.high { 1.0f32 } else { -1.0f32 }
    }

    pub fn get_next_sample(&self, index: u32) -> Option<f32> {
        if index < self.len() {
            return Some(self.get_sample());
        }
        return None;
    }
}
