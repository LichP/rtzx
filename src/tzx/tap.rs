//! Data payloads corresponding to known platform encodings, such as defined in TAP files.

pub mod cpc;
pub mod crc_reader;
pub mod xor_reader_writer;

pub use cpc::{CPCData, CPCHeader, CPCSync};
pub use crc_reader::CrcPagedRW;
pub use xor_reader_writer::{XorReader, XorWriter};

/// A payload corresponding to a known platform encoding.
pub trait Payload: std::fmt::Display {
    /// Returns the bytes representing this payload, excluding any padding and checksums.
    fn bytes(&self) -> Vec<u8>;

    fn clone_box(&self) -> Box<dyn Payload>;
}

impl Clone for Box<dyn Payload> {
    fn clone(&self) -> Box<dyn Payload> {
        self.clone_box()
    }
}
