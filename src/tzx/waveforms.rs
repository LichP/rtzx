pub mod pulse;
pub mod data_waveform;
pub mod direct_waveform;
pub mod empty_waveform;
pub mod generalized_waveform;
pub mod pause_waveform;
pub mod pilot_waveform;
pub mod pulse_sequence_waveform;
pub mod sync_waveform;

pub use pulse::Pulse;
pub use data_waveform::DataWaveform;
pub use direct_waveform::DirectWaveform;
pub use empty_waveform::EmptyWaveform;
pub use generalized_waveform::GeneralizedWaveform;
pub use pause_waveform::{PauseType, PauseWaveform};
pub use pilot_waveform::PilotWaveform;
pub use pulse_sequence_waveform::PulseSequenceWaveform;
pub use sync_waveform::SyncWaveform;

use rodio::{
    Source,
};
use std::fmt;

use crate::tzx::data::DataPayloadWithPosition;

pub trait Waveform: Source + fmt::Display {
    fn clone_box(&self) -> Box<dyn Waveform + Send>;

    fn started(&self) -> bool;

    fn visualise(&self, _pulse_string_length: usize) -> String { "".to_string() }

    fn payload_with_position(&self) -> Option<DataPayloadWithPosition> { None }
}

impl Clone for Box<dyn Waveform + Send> {
    fn clone(&self) -> Box<dyn Waveform + Send> {
        self.clone_box()
    }
}
