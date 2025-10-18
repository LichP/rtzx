use binrw::{
    binrw,
};
use strum_macros::Display;

#[binrw]
#[brw(little, repr = u8)]
#[derive(Clone, Copy, Display, Debug)]
pub enum ParallelPortType {
    KempstonS = 0x00,
    KempstonE = 0x01,
    ZXSpectrumPlus3 = 0x02,
    Tasman = 0x03,
    DKTronics = 0x04,
    Hilderbay = 0x05,
    INESPrinterface = 0x06,
    ZXLPrintInterface3 = 0x07,
    MultiPrint = 0x08,
    OpusDiscovery = 0x09,
    Standard8255 = 0x0a,
}
