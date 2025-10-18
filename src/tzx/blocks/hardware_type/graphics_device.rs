use binrw::{
    binrw,
};
use strum_macros::Display;

#[binrw]
#[brw(little, repr = u8)]
#[derive(Clone, Copy, Display, Debug)]
pub enum GraphicsDeviceType {
    WRXHiRes = 0x00,
    G007 = 0x01,
    Memotech = 0x02,
    LambdaColour = 0x03,
}
