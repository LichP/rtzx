use binrw::{
    binrw,
};
use strum_macros::Display;

#[binrw]
#[brw(little, repr = u8)]
#[derive(Clone, Copy, Display, Debug, Eq, PartialEq, Hash)]
pub enum SerialPortType {
    ZXInterface1 = 0x00,
    ZXSpectrum128k = 0x01,
}
