pub mod pulse;
pub mod data_waveform;
pub mod direct_waveform;
pub mod empty_waveform;
pub mod pause_waveform;
pub mod pilot_waveform;
pub mod pulse_sequence_waveform;
pub mod sync_waveform;

pub use pulse::Pulse;
pub use data_waveform::DataWaveform;
pub use direct_waveform::DirectWaveform;
pub use empty_waveform::EmptyWaveform;
pub use pause_waveform::PauseWaveform;
pub use pilot_waveform::PilotWaveform;
pub use pulse_sequence_waveform::PulseSequenceWaveform;
pub use sync_waveform::SyncWaveform;

use rodio::{
    Source,
};
use std::fmt;

pub trait Waveform: Source + fmt::Display {
    fn clone_box(&self) -> Box<dyn Waveform + Send>;

    fn started(&self) -> bool;

    fn visualise(&self) -> String { "".to_string() }
}

impl Clone for Box<dyn Waveform + Send> {
    fn clone(&self) -> Box<dyn Waveform + Send> {
        self.clone_box()
    }
}
