use binrw::{
    binrw,
};
use strum_macros::Display;

#[binrw]
#[brw(little, repr = u8)]
#[derive(Clone, Copy, Display, Debug, Eq, PartialEq, Hash)]
pub enum KeyboardType {
    KeypadForZXSpectrum128k = 0x00,
}
