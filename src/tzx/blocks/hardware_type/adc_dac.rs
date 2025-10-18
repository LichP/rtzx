use binrw::{
    binrw,
};
use strum_macros::Display;

#[binrw]
#[brw(little, repr = u8)]
#[derive(Clone, Copy, Display, Debug)]
pub enum AdcDacType {
    HarleySystemsADC82 = 0x00,
    BlackboardElectronics = 0x01,
}
