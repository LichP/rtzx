use std::fmt;
use std::hash::{Hash, Hasher};
use std::sync::Arc;
use std::time::Duration;

use crate::tzx::Config;

/// A pulse within a waveform.
///
/// This struct is used as a building block by waveforms, which will usually iterate over a series of pulses
/// and then use [.next_sample()](Pulse::next_sample) to iterate the samples for each pulse.
#[derive(Clone, Debug)]
pub struct Pulse {
    /// The configuration to use for timings etc.
    pub config: Arc<Config>,
    /// The length of the pulse in ZX Spectrum t cycles.
    pub length: u16,
    /// Indicates whether the pulse is high or low.
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
    /// Pulse constructor.
    pub fn new(config: Arc<Config>, length: u16, high: bool) -> Self {
        return Self {
            config,
            length,
            high,
        }
    }

    /// Returns the length of the pulse in samples for the configured playback duration adjustment and sample rate.
    ///
    /// Note that we round the length to an integer number of samples: this makes all pulses with the same timings
    /// consistent, but may cause issues for certain timings when a waveform is looking for a zero pulse with half
    /// the length of a one pulse due to this rounding. For example, if a none-rounded zero and one would be 13.6
    /// and 27.2 samples long, then when rounded these would be 14 samples for the zero and 27 for the one, making
    /// the one pulse shorter than intended. For longer blocks this drift can cause the block to fail.
    pub fn len(&self) -> u32 {
        return (self.length as f64 * self.config.platform.t_cycle_secs_playback(self.config.playback_duration_percent) * self.config.sample_rate as f64).round() as u32
    }

    /// Returns the duration of the pulse for the configured playback duration adjustment and sample rate.
    pub fn duration(&self) -> Duration {
        Duration::from_secs_f64(self.len() as f64 / self.config.sample_rate as f64)
    }

    // Returns a sample for this pulse.
    pub fn sample(&self) -> f32 {
        return if self.high { 1.0f32 } else { -1.0f32 }
    }

    // Returns the next sample corresponding to the supplied index.
    //
    // The caller is responsible for maintaining index state.
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
