use binrw::{
    binrw,
};
use strum_macros::Display;

#[binrw]
#[brw(little, repr = u8)]
#[derive(Clone, Copy, Display, Debug)]
pub enum ExternalStorageType {
    ZXMicrodrive = 0x00,
    OpusDiscovery = 0x01,
    MGTDisciple = 0x02,
    MGTPlusD = 0x03,
    RotronicsWafadrive = 0x04,
    TRDOSBetaDisk = 0x05,
    ByteDrive = 0x06,
    Watsford = 0x07,
    FIZ = 0x08,
    Radofin = 0x09,
    DidaktikDiskDrives = 0x0a,
    BSDOSMB02 = 0x0b,
    ZXSpectrumPlus3DiskDrive = 0x0c,
    JLOOligerDiskInterface = 0x0d,
    TimexFDD3000 = 0x0e,
    ZebraDiskDrive = 0x0f,
    RamexMillenia = 0x10,
    Larken = 0x11,
    KempstonDiskInterface = 0x12,
    Sandy = 0x13,
    ZXSpectrumPlus3eHardDisk = 0x14,
    ZXATASP = 0x15,
    DivIDE = 0x16,
    ZXCF= 0x17,
}
