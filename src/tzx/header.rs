use binrw::{
    binrw,    // #[binrw] attribute
};
use std::fmt;

#[binrw]
#[brw(little, magic = b"ZXTape!\x1A")]
pub struct Header {
    major: u8,
    minor: u8
}

impl fmt::Display for Header {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "TZX version {}.{}", self.major, self.minor)
    }
}
