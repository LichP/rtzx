use std::sync::Arc;
use std::time::Duration;

use crate::tzx::Machine;

#[derive(Clone)]
pub struct Pulse {
    pub machine: Arc<Machine>,
    pub length: u16,
    pub high: bool,
    index: u32,
}

impl Pulse {
    pub fn new(machine: Arc<Machine>, length: u16, high: bool) -> Self {
        return Self {
            machine,
            length,
            high,
            index: 0,
        }
    }

    pub fn len(&self) -> u32 {
        return (self.length as f64 * self.machine.t_cycle_secs() * 48000.0).round() as u32
    }

    pub fn duration(&self) -> Duration {
        Duration::from_secs_f64(self.len() as f64 / 48000.0)
    }

    pub fn get_sample(&self) -> f32 {
        return if self.high { 1.0f32 } else { -1.0f32 }
    }
}

impl Iterator for Pulse {
    type Item = f32;

    fn next(&mut self) -> Option<Self::Item> {
        if self.index < self.len() {
            self.index += 1;
            return Some(self.get_sample());
        }
        return None;
    }
}
