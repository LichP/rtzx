use std::io::{self, Read, Write, Seek, SeekFrom, Result, Error, ErrorKind};
use crc::{Crc, Algorithm};

const CRC16_CCITT_CPC: Algorithm<u16> = Algorithm {
    width: 16,
    poly: 0x1021,
    init: 0xffff,
    refin: false,
    refout: false,
    xorout: 0xffff,
    check: 0x0000,
    residue: 0x0000,
};

const CRC16: Crc<u16> = Crc::<u16>::new(&CRC16_CCITT_CPC);

/// A reader / writer for handling paged checksums.
///
/// This is used when parsing [DataPayload](crate::tzx::data::DataPayload)s to [CPCHeader](crate::tzx::tap::CPCHeader) and
/// [CPCData](crate::tzx::tap::CPCData) payloads, and will read / write one page of data at a time followed by the relevant
/// checksum bytes, validating and stripping them when reading, and calculating and inserting them when writing.
#[derive(Clone, Debug)]
pub struct CrcPagedRW<RW> {
    inner: RW,
    inner_offset: u64,
    page_size: usize,
    buffer: Vec<u8>,
    buf_pos: usize,
    page_filled: bool,
    page_number: u64,
}

impl<RW> CrcPagedRW<RW> {
    pub fn new(inner: RW, inner_offset: u64, page_size: usize) -> Self {
        Self {
            inner,
            inner_offset,
            page_size,
            buffer: vec![0; page_size],
            buf_pos: 0,
            page_filled: false,
            page_number: 0,
        }
    }

    pub fn into_inner(self) -> RW { self.inner }
}

impl<R: Read> CrcPagedRW<R> {
    fn read_page(&mut self) -> io::Result<()> {
        // eprintln!("read_next_page: {:?}", self.page_number);
        // Read exactly one page.
        self.inner.read_exact(&mut self.buffer)?;
        // eprintln!("read_next_page buffer: {:?}", self.buffer);

        // Read CRC.
        let mut crc_bytes = [0u8; 2];
        self.inner.read_exact(&mut crc_bytes)?;
        let expected = u16::from_be_bytes(crc_bytes);

        // Compute actual CRC over the full page.
        let actual = CRC16.checksum(&self.buffer);
        if actual != expected {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                format!("CRC mismatch: expected {:04X}, got {:04X}", expected, actual),
            ));
        }

        self.page_filled = true;
        Ok(())
    }
}

impl<R: Read> Read for CrcPagedRW<R> {
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        let mut total_read = 0;

        while total_read < buf.len() {
            if !self.page_filled {
                match self.read_page() {
                    Ok(_) => {}
                    Err(ref e) if e.kind() == io::ErrorKind::UnexpectedEof => break,
                    Err(e) => return Err(e),
                }
            }

            let remaining = self.page_size - self.buf_pos as usize;
            let to_copy = std::cmp::min(remaining, buf.len() - total_read);

            buf[total_read..total_read + to_copy]
                .copy_from_slice(&self.buffer[self.buf_pos..self.buf_pos + to_copy]);

            self.buf_pos += to_copy;
            total_read += to_copy;

            if self.buf_pos == self.page_size {
                // finished current block
                self.page_filled = false;
                self.buf_pos = 0;
                self.page_number += 1;
            }
        }

        Ok(total_read)
    }
}

impl<W: Write> Write for CrcPagedRW<W> {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        let mut total_written = 0;

        while total_written < buf.len() {
            let remaining_block = self.page_size - self.buf_pos;
            let to_write = std::cmp::min(remaining_block, buf.len() - total_written);

            // Copy into buffer.
            self.buffer[self.buf_pos..self.buf_pos + to_write]
                .copy_from_slice(&buf[total_written..total_written + to_write]);

            self.buf_pos += to_write;
            total_written += to_write;

            if self.buf_pos == self.page_size {
                // Block full → write block + CRC.
                self.inner.write_all(&self.buffer)?;
                let crc = CRC16.checksum(&self.buffer);
                self.inner.write_all(&crc.to_be_bytes())?;

                // Reset buffer position.
                self.buf_pos = 0;
            }
        }

        Ok(total_written)
    }

    fn flush(&mut self) -> io::Result<()> {
        // Write final partial page if needed.
        if self.buf_pos > 0 {
            // Pad with zeros
            for b in self.buf_pos..self.page_size {
                self.buffer[b] = 0;
            }
            self.inner.write_all(&self.buffer)?;
            let crc = CRC16.checksum(&self.buffer);
            self.inner.write_all(&crc.to_be_bytes())?;
            self.buf_pos = 0;
        }

        self.inner.flush()
    }
}

impl<RW: Seek> Seek for CrcPagedRW<RW> {
    fn seek(&mut self, pos: SeekFrom) -> Result<u64> {
        // eprintln!("{:?}", pos);
        let mut new_buf_pos: u64 = 0;
        let mut new_page_number: u64 = 0;
        let mut seek_before_crc_start = false;
        match pos {
            SeekFrom::Current(offset) if self.inner_offset as i64 + offset < 0 => {
                seek_before_crc_start = true;
            },
            SeekFrom::Current(mut offset) => {
                new_page_number = self.page_number;
                while offset < 0 {
                    if (offset.abs() as u64) <= self.buf_pos as u64 {
                        offset = self.buf_pos as i64 + offset;
                        break;
                    }
                    offset += self.page_size as i64;
                    new_page_number -= 1;
                }
                while offset >= self.page_size as i64 {
                    offset -= self.page_size as i64;
                    new_page_number += 1;
                }
                new_buf_pos = self.buf_pos as u64 + offset as u64;
            }
            SeekFrom::Start(start_pos) if start_pos < self.inner_offset => {
                seek_before_crc_start = true;
            },
            SeekFrom::Start(start_pos) => {
                let mut crc_start_pos = start_pos - self.inner_offset;
                while crc_start_pos >= self.page_size as u64 {
                    crc_start_pos -= self.page_size as u64;
                    new_page_number += 1;
                }
                new_buf_pos = crc_start_pos;
                // eprintln!("new_buf_pos: {}; new_page_number: {}", new_buf_pos, new_page_number);
            }
            SeekFrom::End(_) => { return Err(Error::new(ErrorKind::Unsupported, "CrcPagedRW cannot seek from end")) }
        }

        self.page_number = new_page_number;
        self.buf_pos = new_buf_pos as usize;

        if seek_before_crc_start {
            self.page_filled = false;
            return self.inner.seek(pos);
        }

        if self.page_number != new_page_number {
            self.page_filled = false;
            self.inner.seek(SeekFrom::Current((new_page_number as i64 - self.page_number as i64) * (self.page_size as i64 + 2)))?;
        }

        return Ok(self.page_number * self.page_size as u64 + self.buf_pos as u64);
    }
}
