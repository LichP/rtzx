use binrw::{
    binrw,
};
use strum_macros::Display;

#[binrw]
#[brw(little, repr = u8)]
#[derive(Clone, Copy, Display, Debug, Eq, PartialEq, Hash)]
pub enum JoystickType {
    Kempston = 0x00,
    CursorProtekAGF = 0x01,
    Sinclair2Left = 0x02,
    Sinclair1Right = 0x03,
    Fuller = 0x04,
}
