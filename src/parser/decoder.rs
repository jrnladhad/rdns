use thiserror::Error;

pub(crate) type ParserResult<T> = Result<T, ParserError>;

#[derive(Debug, Error, PartialEq)]
#[non_exhaustive]
pub(crate) enum ParserError {
    #[error("{data_requested} data requested, but only {remaining_data} is left")]
    TooMuchDataRequested {
        data_requested: usize,
        remaining_data: usize
    }
}

pub struct Decoder {
    binary_data: Vec<u8>,
    cursor: usize
}

impl Decoder {
    pub fn new(binary_data: Vec<u8>) -> Decoder {
        Decoder {
            binary_data,
            cursor: 0
        }
    }

    fn buf_len(&self) -> usize {
        self.binary_data.len()
    }

    fn read_n_bytes(&mut self, n: usize) -> ParserResult<&[u8]>
    {
        let buf_len = self.buf_len();

        if self.cursor >= buf_len || self.cursor + n > buf_len {
            return Err(ParserError::TooMuchDataRequested {
                data_requested: n,
                remaining_data: buf_len - self.cursor
            });
        }

        let data = &self.binary_data[self.cursor .. self.cursor + n];
        self.cursor += n;

        Ok(data)
    }

    pub fn read_u8(&mut self) -> ParserResult<u8> {
        let bytes = self.read_n_bytes(1)?;

        Ok(bytes[0])
    }

    pub fn read_u16(&mut self) -> ParserResult<u16> {
        let bytes = self.read_n_bytes(2)?;

        Ok(u16::from_be_bytes([bytes[0], bytes[1]]))
    }

    pub fn read_u32(&mut self) -> ParserResult<u32> {
        let bytes = self.read_n_bytes(4)?;

        Ok(u32::from_be_bytes([bytes[0], bytes[1], bytes[2], bytes[3]]))
    }
}

#[cfg(test)]
mod parser_test {
    use crate::parser::decoder::{Decoder, ParserError};

    #[test]
    fn read_bytes()
    {
        let packet_bytes = vec![0xf2, 0xe8, 0x01, 0x00, 0x00, 0x01, 0x00];
        let mut decoder = Decoder::new(packet_bytes);

        let u8_bytes = decoder.read_u8().expect("Failed to read 1 byte");
        assert_eq!(u8_bytes, 0xf2);

        let u16_bytes = decoder.read_u16().expect("Failed to read 2 bytes");
        assert_eq!(u16_bytes, 0xe801);

        let u32_bytes = decoder.read_u32().expect("Failed to read 4 bytes");
        assert_eq!(u32_bytes, 0x00000100);
    }

    #[test]
    fn read_bytes_fail()
    {
        let packet_bytes = vec![0x00];
        let mut decoder = Decoder::new(packet_bytes);

        let bytes = decoder.read_n_bytes(10).expect_err("Failed to read 10 bytes");
        assert_eq!(bytes, ParserError::TooMuchDataRequested {
            data_requested: 10,
            remaining_data: 1
        });
    }
}