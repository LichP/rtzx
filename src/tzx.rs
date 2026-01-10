pub mod blocks;
pub mod config;
pub mod data;
pub mod header;
pub mod tap;
pub mod tzx_data;
pub mod platform;
pub mod player;
pub mod recovery_enum;
pub mod waveforms;

pub use config::Config;
pub use header::Header;
pub use tzx_data::TzxData;
pub use platform::Platform;
pub use player::Player;
pub use recovery_enum::RecoveryEnum;

/// Facilitates gathering additional pieces of information for display in contexts
/// where more detail is desired.
pub trait ExtendedDisplayCollector {
    /// Push a piece of displayable data to the collector.
    ///
    /// Callers are free to push as many pieces of information as they like, for example
    /// for each entry in a collection.
    ///
    /// Trait implementations are responsible for displaying the information received from
    /// the sender in whatever way they see fit, e.g. by directly printing or collecting to a
    /// vec for later rendering in a UI.
    fn push(&mut self, piece: &dyn std::fmt::Display);
}
