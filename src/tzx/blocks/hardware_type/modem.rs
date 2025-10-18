use binrw::{
    binrw,
};
use strum_macros::Display;

#[binrw]
#[brw(little, repr = u8)]
#[derive(Clone, Copy, Display, Debug)]
pub enum ModemType {
    PrismVTX5000 = 0x00,
    TS2050Westridge2050 = 0x01,
}
