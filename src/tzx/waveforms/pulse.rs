use std::fmt;
use std::hash::{Hash, Hasher};
use std::sync::Arc;
use std::time::Duration;

use crate::tzx::Config;

#[derive(Clone, Debug)]
pub struct Pulse {
    pub config: Arc<Config>,
    pub length: u16,
    pub high: bool,
}

impl PartialEq for Pulse {
    fn eq(&self, other: &Self) -> bool { self.length == other.length && self.high == other.high }
}

impl Eq for Pulse {}

impl Hash for Pulse {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.length.hash(state);
        self.high.hash(state);
    }
}

impl Default for Pulse {
    fn default() -> Self { Pulse::new(Arc::new(Config::default()), 0, true)}
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

    pub fn sample(&self) -> f32 {
        return if self.high { 1.0f32 } else { -1.0f32 }
    }

    pub fn next_sample(&self, index: u32) -> Option<f32> {
        if index < self.len() {
            return Some(self.sample());
        }
        return None;
    }
}

impl fmt::Display for Pulse {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}{}", self.length, if self.high { 'h' } else { 'l' })
    }
}
