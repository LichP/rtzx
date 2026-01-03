use binrw::{
    binrw,
};
use strum_macros::Display;

#[binrw]
#[brw(little, repr = u8)]
#[derive(Clone, Copy, Display, Debug, Eq, PartialEq, Hash)]
pub enum PrinterType {
    ZXPrinterAlphacom32Compatibles = 0x00,
    GenericPrinter = 0x01,
    EPSONCompatible = 0x02,
}
