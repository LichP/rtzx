use binrw::{
    BinRead,
    BinWrite,
};
use std::fmt;
use std::io::{
    Read,
    Seek,
};

#[derive(Clone, Copy, Debug)]
pub enum RecoveryEnum<TKnown, TUnknown> {
    Known(TKnown),
    Unknown(TUnknown),
}

impl<TKnown, TUnknown> BinRead for RecoveryEnum<TKnown, TUnknown>
where
    TKnown: BinRead<Args<'static> = ()> + 'static,
    TUnknown: BinRead<Args<'static> = ()> + 'static,
{
    type Args<'a> = ();

    fn read_options<R: Read + Seek>(
        reader: &mut R,
        endian: binrw::Endian,
        _args: Self::Args<'_>,
    ) -> binrw::BinResult<Self> {
        let start_pos = reader.stream_position()?;
        let raw_val: TUnknown = BinRead::read_options(reader, endian, ())?;
        reader.seek(std::io::SeekFrom::Start(start_pos))?;
        match TKnown::read_options(reader, endian, ()) {
            Ok(val) => Ok(RecoveryEnum::Known(val)),
            Err(_) => {
                reader.seek(std::io::SeekFrom::Start(start_pos + 1))?;
                Ok(RecoveryEnum::Unknown(raw_val))
            }
        }
    }
}

impl<TKnown, TUnknown> BinWrite for RecoveryEnum<TKnown, TUnknown>
where
    TKnown: BinWrite<Args<'static> = ()> + 'static,
    TUnknown: BinWrite<Args<'static> = ()> + 'static,
{
    type Args<'a> = ();

    fn write_options<W: std::io::Write + std::io::Seek>(
        &self,
        writer: &mut W,
        endian: binrw::Endian,
        _args: Self::Args<'_>,
    ) -> binrw::BinResult<()> {
        match self {
            RecoveryEnum::Known(val_known) => val_known.write_options(writer, endian, ()),
            RecoveryEnum::Unknown(val_unknown) => val_unknown.write_options(writer, endian, ()),
        }
    }
}

impl<TKnown, TUnknown> fmt::Display for RecoveryEnum<TKnown, TUnknown>
where
    TKnown: fmt::Display,
    TUnknown: fmt::Display,
{
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            RecoveryEnum::Known(val_known) => val_known.fmt(f),
            RecoveryEnum::Unknown(val_unknown) => val_unknown.fmt(f),
        }
    }
}
