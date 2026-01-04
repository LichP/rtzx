use binrw::{
    binrw,    // #[binrw] attribute
};
use std::fmt;

#[binrw]
#[brw(little, magic = b"ZXTape!\x1A")]
#[derive(Clone, Debug)]
pub struct Header {
    major: u8,
    minor: u8
}

impl Header {
    pub fn new(major: u8, minor: u8) -> Self { Header { major, minor }}
}

impl Default for Header {
    fn default() -> Self { Header::new(1, 20) }
}

impl fmt::Display for Header {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "TZX version {}.{}", self.major, self.minor)
    }
}
