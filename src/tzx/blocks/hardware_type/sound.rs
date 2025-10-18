use binrw::{
    binrw,
};
use strum_macros::Display;

#[binrw]
#[brw(little, repr = u8)]
#[derive(Clone, Copy, Display, Debug)]
pub enum SoundType {
    ClassicAY = 0x00,
    FullerBoxAY = 0x01,
    CurrahMicroSpeech = 0x02,
    SpecDrum = 0x03,
    AYACBStereoMelodik = 0x04,
    AYABCStereo = 0x05,
    RAMMusicMachine = 0x06,
    Covox = 0x07,
    GeneralSound = 0x08,
    IntecElectronicsDigitalInterfaceB8001 = 0x09,
    ZonXAY = 0x0a,
    QuickSilvaAY = 0x0b,
    JupiterACE = 0x0c,
}
