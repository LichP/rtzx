use rodio::{
    ChannelCount,
    SampleRate,
    Source,
};
use std::fmt;
use std::sync::Arc;
use std::time::Duration;

use crate::tzx::{
    Config,
    waveforms::Waveform,
};

#[derive(Clone)]
pub struct EmptyWaveform {
    config: Arc<Config>,
}

impl EmptyWaveform {
    pub fn new(config: Arc<Config>) -> Self {
        return Self {
            config,
        }
    }
}

impl Iterator for EmptyWaveform {
    type Item = f32;

    fn next(&mut self) -> Option<Self::Item> {
        return None;
    }
}

impl Source for EmptyWaveform {
    fn channels(&self) -> ChannelCount { 1 }
    fn sample_rate(&self) -> SampleRate { self.config.sample_rate }
    fn current_span_len(&self) -> Option<usize> { None }
    fn total_duration(&self) -> Option<Duration> { None }
}

impl Waveform for EmptyWaveform {
    fn clone_box(&self) -> Box<dyn Waveform + Send> {
        Box::new(self.clone())
    }
}

impl fmt::Display for EmptyWaveform {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "EmptyWaveform")
    }
}
