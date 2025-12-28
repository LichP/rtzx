pub mod cpc;
pub mod crc_reader;

pub use cpc::{CPCData, CPCHeader, CPCSync};
pub use crc_reader::CrcPagedRW;

pub trait Payload: std::fmt::Display {
    fn bytes(&self) -> Vec<u8>;

    fn clone_box(&self) -> Box<dyn Payload>;
}

impl Clone for Box<dyn Payload> {
    fn clone(&self) -> Box<dyn Payload> {
        self.clone_box()
    }
}
