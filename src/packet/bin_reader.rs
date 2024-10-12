use thiserror::Error;

#[derive(Error, Debug)]
pub enum BinReaderError {
    #[error("{0} data requested, not enough data in buffer")]
    TooMuchDataRequested(u16),
    #[error("Cursor is past the buffer size and no data can be read")]
    ReaderIsPastTheDataBuffer,
}

pub type BinReaderResult<T> = Result<T, BinReaderError>;

pub struct BinReader<'a> {
    bin_data: &'a[u8],
    cursor: u16
}

impl<'a> BinReader<'a> {
    pub fn new(bin_data: &'a [u8]) -> BinReader<'a> {
        BinReader {
            bin_data,
            cursor: 0
        }
    }

    fn buf_len(&self) -> usize {
        self.bin_data.len()
    }

    pub fn read_n_bytes(&mut self, n: u16) -> BinReaderResult<&[u8]>
    {
        let buf_len = self.buf_len() as u16;

        if self.cursor >= buf_len || self.cursor + n > buf_len {
            return Err(BinReaderError::TooMuchDataRequested(n));
        }

        let data = &self.bin_data[(self.cursor as usize).. (self.cursor + n) as usize];
        self.cursor += n;

        Ok(data)
    }

    pub fn read_u8(&mut self) -> BinReaderResult<u8> {
        let bytes = self.read_n_bytes(1)?;

        Ok(bytes[0])
    }

    pub fn read_u16(&mut self) -> BinReaderResult<u16> {
        let bytes = self.read_n_bytes(2)?;

        Ok(u16::from_be_bytes([bytes[0], bytes[1]]))
    }

    pub fn read_u32(&mut self) -> BinReaderResult<u32> {
        let bytes = self.read_n_bytes(4)?;

        Ok(u32::from_be_bytes([bytes[0], bytes[1], bytes[2], bytes[3]]))
    }

    pub fn peek(&self) -> BinReaderResult<u8> {
        if self.cursor as usize >= self.bin_data.len() {
            return Err(BinReaderError::ReaderIsPastTheDataBuffer)
        }

        Ok(self.bin_data[self.cursor as usize])
    }

    pub fn cheap_clone(&self, cursor: u16) -> Self {
        BinReader {
            bin_data: self.bin_data,
            cursor
        }
    }
}
