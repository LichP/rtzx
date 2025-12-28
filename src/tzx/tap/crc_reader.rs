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

pub struct CrcPagedRW<RW> {
    inner: RW,
    page_size: usize,
    buffer: Vec<u8>,
    buf_pos: usize,
    page_filled: bool,
    page_number: u64,
}

impl<RW> CrcPagedRW<RW> {
    pub fn new(inner: RW, page_size: usize) -> Self {
        Self {
            inner,
            page_size,
            buffer: vec![0; page_size],
            buf_pos: 0,
            page_filled: false,
            page_number: 0,
        }
    }
}

impl<R: Read> CrcPagedRW<R> {
    fn read_next_page(&mut self) -> io::Result<()> {
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

        self.buf_pos = 0;
        self.page_filled = true;
        Ok(())
    }
}

impl<R: Read> Read for CrcPagedRW<R> {
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        // eprintln!("read: {:?}", buf.len());
        let mut total_read = 0;

        while total_read < buf.len() {
            if !self.page_filled {
                // load next block
                match self.read_next_page() {
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

impl<RW: Read + Seek> Seek for CrcPagedRW<RW> {
    fn seek(&mut self, pos: SeekFrom) -> Result<u64> {
        // eprintln!("{:?}", pos);
        let new_buf_pos: u64;
        let mut new_page_number: u64 = 0;
        match pos {
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
            SeekFrom::Start(mut start_pos) => {
                while start_pos >= self.page_size as u64 {
                    start_pos -= self.page_size as u64;
                    new_page_number += 1;
                }
                new_buf_pos = start_pos;
                // eprintln!("new_buf_pos: {}; new_page_number: {}", new_buf_pos, new_page_number);
            }
            SeekFrom::End(_) => { return Err(Error::new(ErrorKind::Unsupported, "CrcPagedRW cannot seek from end")) }
        }

        self.page_number = new_page_number;
        if self.page_number != new_page_number {
            let seek_inner = self.inner.seek(SeekFrom::Current((new_page_number as i64 - self.page_number as i64) * (self.page_size as i64 + 2)));
            if seek_inner.is_err() { return seek_inner }
            self.read_next_page().unwrap();
        }

        self.buf_pos = new_buf_pos as usize;

        return Ok(self.page_number * self.page_size as u64 + self.buf_pos as u64);
    }
}
