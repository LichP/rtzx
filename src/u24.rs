use binrw::{
    BinRead, BinWrite, BinResult, Endian
};
use std::io::{Read, Seek, Write};

pub struct U24(pub u32);

impl From<u32> for U24 {
    fn from(value: u32) -> Self { U24(value) }
}

impl From<usize> for U24 {
    fn from(value: usize) -> Self { U24(value as u32) }
}

impl From<U24> for u32 {
    fn from(value: U24) -> Self { value.0 }
}

impl From<U24> for usize {
    fn from(value: U24) -> Self { value.0 as usize }
}

impl BinRead for U24 {
    type Args<'a> = ();

    fn read_options<R: Read + Seek>(
        reader: &mut R,
        endian: binrw::Endian,
        _args: Self::Args<'_>,
    ) -> BinResult<Self> {
    type ConvFn = fn([u8; 4]) -> u32;
        let mut buf = [0u8; 4];
        let (conv, out): (ConvFn, &mut [u8]) = match endian {
            Endian::Little => (u32::from_le_bytes, &mut buf[..3]),
            Endian::Big => (u32::from_be_bytes, &mut buf[1..]),
        };
        reader.read_exact(out)?;
        Ok(U24(conv(buf)))
    }
}

impl BinWrite for U24 {
    type Args<'a> = ();

    fn write_options<W: Write + Seek>(
        &self,
        writer: &mut W,
        endian: Endian,
        _args: Self::Args<'_>,
    ) -> BinResult<()> {
        let (buf, range) = match endian {
            Endian::Little => (self.0.to_le_bytes(), 0..3),
            Endian::Big => (self.0.to_be_bytes(), 1..4),
        };
        writer.write_all(&buf[range]).map_err(Into::into)
    }
}
