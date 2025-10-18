use binrw::{
    binrw,
};
use strum_macros::Display;

#[binrw]
#[brw(little, repr = u8)]
#[derive(Clone, Copy, Display, Debug)]
pub enum MouseType {
    AMXMouse = 0x00,
    KempstonMouse = 0x01,
}
