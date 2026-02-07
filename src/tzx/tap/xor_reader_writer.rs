use std::io::{self, Read, Write, Seek, SeekFrom, Result, Error, ErrorKind};

/// A reader for handling XOR checksums.
///
/// This is used when parsing [SpectrumHeader](crate::tzx::tap::SpectrumHeader) and [SpectrumData](crate::tzx::tap::SpectrumData)
/// payloads to facilitate validation of the checksum byte.
#[derive(Clone, Debug)]
pub struct XorReader<R> {
    inner: R,
    initial: u8,
    xor: u8,
}

impl<R: Read> XorReader<R> {
    pub fn new(inner: R, initial: u8) -> Self {
        Self {
            inner,
            initial,
            xor: initial,
        }
    }

    pub fn xor(&self) -> u8 { self.xor }
}

impl<R: Read> Read for XorReader<R> {
    fn read(&mut self, out: &mut [u8]) -> io::Result<usize> {
        let total_read = self.inner.read(out)?;

        for b in out {
            self.xor ^= *b;
        }
        Ok(total_read)
    }
}

impl<R: Read + Seek> Seek for XorReader<R> {
    fn seek(&mut self, pos: SeekFrom) -> Result<u64> {
        let mut buf = match pos {
            SeekFrom::Current(offset) => {
                // Read bytes between current pos and offset.
                match offset {
                    o if o > 0 => vec![0 as u8; o as usize],
                    o if o < 0 => { self.inner.seek(pos)?; vec![0; o.abs() as usize] }
                    _ => vec![]
                }
            }
            SeekFrom::Start(start_pos) => {
                // Read bytes between start and target position.
                self.inner.seek(SeekFrom::Start(0))?;
                self.xor = self.initial;
                vec![0; start_pos as usize]
            }
            SeekFrom::End(_) => { return Err(Error::new(ErrorKind::Unsupported, "XorRW cannot seek from end")) }
        };

        // Read bytes between current and new position.
        self.inner.read_exact(&mut buf)?;

        // Update XOR
        for b in buf {
            self.xor ^= b;
        }

        return self.inner.seek(pos);
    }
}

/// A reader for handling XOR checksums.
///
/// This is used when parsing [SpectrumHeader](crate::tzx::tap::SpectrumHeader) and [SpectrumData](crate::tzx::tap::SpectrumData)
/// payloads to facilitate validation of the checksum byte.
#[derive(Clone, Debug)]
pub struct XorWriter<W> {
    inner: W,
    xor: u8,
}

impl<W: Write> XorWriter<W> {
    pub fn new(inner: W, initial: u8) -> Self {
        Self {
            inner,
            xor: initial,
        }
    }

    pub fn xor(&self) -> u8 { self.xor }
}

impl<W: Write> Write for XorWriter<W> {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        let n = self.inner.write(buf)?;
        for &b in &buf[..n] {
            self.xor ^= b;
        }
        Ok(n)
    }

    fn flush(&mut self) -> io::Result<()> { self.inner.flush() }
}

impl<W: Write + Seek> Seek for XorWriter<W> {
    fn seek(&mut self, pos: SeekFrom) -> Result<u64> { self.inner.seek(pos) }
}
