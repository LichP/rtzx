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

/// A waveform which provides samples for conversion or playback.
///
/// This trait extends [Source] to allow for direct playback with Rodio, and therefore in turn
/// extends [Iterator] to iterate samples directly.
pub trait Waveform: Source + fmt::Display {
    /// Returns a boxed dyn clone of the waveform.
    fn clone_box(&self) -> Box<dyn Waveform + Send>;

    /// Returns whether or not the waveform has started.
    fn started(&self) -> bool;

    /// Returns the current baud where applicable. Default implementation is to return `None`,
    /// for waveforms that do not correspond to actual data such as pauses, tones, etc.
    fn current_baud(&self) -> Option<usize> { None }

    /// Returns a visualisation string of the given length.
    ///
    /// For waveforms consisting of pulses, a [unicode full block](https://unicodeplus.com/U+2588)
    /// character is used to represent high pulses, and a space for low pulses. The number of
    /// characters per pulse should approximately correspond to the length of the pulse relative to
    /// the shortest length of any pulse in the waveform. For example, in [DataWaveform] the pulses
    /// for zeros are usually around half the length of the pulses for ones, so a single character
    /// per pulse is used for zeros, and two characters per pulse are used for ones.
    fn visualise(&self, _pulse_string_length: usize) -> String { "".to_string() }

    /// Returns the waveform's payload with current playback position, if applicable.
    fn payload_with_position(&self) -> Option<DataPayloadWithPosition> { None }
}

impl Clone for Box<dyn Waveform + Send> {
    fn clone(&self) -> Box<dyn Waveform + Send> {
        self.clone_box()
    }
}
