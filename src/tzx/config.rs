//! rtzx configuration.

use bon::Builder;
use rodio::SampleRate;
use std::time::Duration;

use crate::tzx::Platform;

/// Configuration information for rtzx.
#[derive(Clone, Debug, Builder, Default)]
pub struct Config {
    /// The [Platform]. This makes no difference for conversion / playback.
    #[builder(default = Platform::ZXSpectrum)]
    pub platform: Platform,
    /// The [SampleRate] for conversion / playback. Defaults to 44100.
    #[builder(default = 44100 as SampleRate)]
    pub sample_rate: SampleRate,
    /// The size of the playback buffer in milliseconds.
    #[builder(default = 200)]
    pub buffer_length_ms: u32,
    /// Modifies the lengths of [Pulse](crate::tzx::waveforms::Pulse)s to increase or decrease playback duration.
    /// Positive integers increase pulse lengths by that percentage, negative integers reduce them.
    #[builder(default = 0)]
    pub playback_duration_percent: i32
}

impl Config {
    /// Returns the desired size of the playback buffer in samples.
    pub fn buffer_size(&self) -> u32 { self.sample_rate * self.buffer_length_ms / 1000 }

    /// Returns the playback buffer delay as a [Duration].
    pub fn buffer_delay(&self) -> Duration {
        Duration::from_secs_f64(self.buffer_size() as f64 / self.sample_rate as f64)
    }
}
