use binrw::{
    binrw,
};
use strum_macros::Display;

#[binrw]
#[brw(little, repr = u8)]
#[derive(Clone, Copy, Display, Debug)]
pub enum OtherControllerType {
    Trickstick = 0x00,
    ZXLightGun = 0x01,
    ZebraGraphicsTablet = 0x02,
    DefenderLightGun = 0x03,
}
