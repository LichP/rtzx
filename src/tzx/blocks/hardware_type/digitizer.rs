use binrw::{
    binrw,
};
use strum_macros::Display;

#[binrw]
#[brw(little, repr = u8)]
#[derive(Clone, Copy, Display, Debug)]
pub enum DigitizerType {
    RDDigitalTracer = 0x00,
    DKTronicsLightPen = 0x01,
    BritishMicroGraphPad = 0x02,
    RomanticRobotVideoface = 0x03,
}
