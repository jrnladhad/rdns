use thiserror::Error;

#[derive(Error, Debug)]
pub enum DeserializeError {
    #[error("{0} data requested, not enough data in buffer")]
    TooMuchDataRequested(u16),
    #[error("Cursor is past the buffer size and no data can be read")]
    ReaderIsPastTheDataBuffer,
}

pub type DeserializeResult<T> = Result<T, DeserializeError>;

pub struct Deserialize<'a> {
    bin_data: &'a [u8],
    cursor: u16,
}

impl<'a> Deserialize<'a> {
    pub fn new(bin_data: &'a [u8]) -> Deserialize<'a> {
        Deserialize {
            bin_data,
            cursor: 0,
        }
    }

    fn buf_len(&self) -> usize {
        self.bin_data.len()
    }

    pub fn read_n_bytes(&mut self, n: u16) -> DeserializeResult<&[u8]> {
        let buf_len = self.buf_len() as u16;

        if self.cursor >= buf_len || self.cursor + n > buf_len {
            return Err(DeserializeError::TooMuchDataRequested(n));
        }

        let data = &self.bin_data[(self.cursor as usize)..(self.cursor + n) as usize];
        self.cursor += n;

        Ok(data)
    }

    pub fn read_u8(&mut self) -> DeserializeResult<u8> {
        let bytes = self.read_n_bytes(1)?;

        Ok(bytes[0])
    }

    pub fn read_u16(&mut self) -> DeserializeResult<u16> {
        let bytes = self.read_n_bytes(2)?;

        Ok(u16::from_be_bytes([bytes[0], bytes[1]]))
    }

    pub fn read_u32(&mut self) -> DeserializeResult<u32> {
        let bytes = self.read_n_bytes(4)?;

        Ok(u32::from_be_bytes([bytes[0], bytes[1], bytes[2], bytes[3]]))
    }

    pub fn read_u128(&mut self) -> DeserializeResult<u128> {
        let bytes = self.read_n_bytes(16)?;

        Ok(u128::from_be_bytes([
            bytes[0], bytes[1], bytes[2], bytes[3], bytes[4], bytes[5], bytes[6], bytes[7],
            bytes[8], bytes[9], bytes[10], bytes[11], bytes[12], bytes[13], bytes[14], bytes[15],
        ]))
    }

    pub fn peek(&self) -> DeserializeResult<u8> {
        if self.cursor as usize >= self.bin_data.len() {
            return Err(DeserializeError::ReaderIsPastTheDataBuffer);
        }

        Ok(self.bin_data[self.cursor as usize])
    }

    pub fn cheap_clone(&self, cursor: u16) -> Self {
        Deserialize {
            bin_data: self.bin_data,
            cursor,
        }
    }

    pub fn cursor(&self) -> u16 {
        self.cursor
    }
}
