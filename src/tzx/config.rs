use bon::Builder;
use rodio::SampleRate;
use std::time::Duration;

use crate::tzx::Platform;

#[derive(Clone, Debug, Builder)]
pub struct Config {
    #[builder(default = Platform::ZXSpectrum)]
    pub platform: Platform,
    #[builder(default = 44100 as SampleRate)]
    pub sample_rate: SampleRate,
    #[builder(default = 1024)]
    pub buffer_size: u32,
}

impl Config {
    pub fn buffer_delay(&self) -> Duration {
        Duration::from_secs_f64(self.buffer_size as f64 / self.sample_rate as f64)
    }
}
