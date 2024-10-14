// standard crates
use std::cmp::PartialEq;

// external crates
use thiserror::Error;

// custom crates
use super::bin_reader::BinReader;
use crate::packet::header_flags::HeaderFlags;

#[derive(Error, Debug, PartialEq)]
pub enum HeaderError {
    #[error("Could not get {0} bytes from wire")]
    InsufficientData(usize),
    #[error("Flag errors")]
    FlagError,
}

type HeaderResult<T> = Result<T, HeaderError>;

#[derive(Debug, PartialEq)]
pub struct Header {
    id: u16,
    flags: HeaderFlags,
    question_count: u16,
    answer_count: u16,
    authoritative_count: u16,
    additional_count: u16,
}

impl Header {
    pub fn from_bytes(decoder: &mut BinReader) -> HeaderResult<Header> {
        let id = decoder
            .read_u16()
            .map_err(|_| HeaderError::InsufficientData(2))?;

        let flags = decoder
            .read_u16()
            .map_err(|_| HeaderError::InsufficientData(2))?;

        let flags = HeaderFlags::try_from(flags).map_err(|_| HeaderError::FlagError)?;

        let question_count = decoder
            .read_u16()
            .map_err(|_| HeaderError::InsufficientData(2))?;

        let answer_count = decoder
            .read_u16()
            .map_err(|_| HeaderError::InsufficientData(2))?;

        let authoritative_count = decoder
            .read_u16()
            .map_err(|_| HeaderError::InsufficientData(2))?;

        let additional_count = decoder
            .read_u16()
            .map_err(|_| HeaderError::InsufficientData(2))?;

        Ok(Header {
            id,
            flags,
            question_count,
            answer_count,
            authoritative_count,
            additional_count,
        })
    }

    pub fn builder(flags: HeaderFlags) -> HeaderBuilder {
        HeaderBuilder::new(flags)
    }

    pub fn id(&self) -> u16 {
        self.id
    }

    pub fn flags(&self) -> &HeaderFlags {
        &self.flags
    }

    pub fn question_count(&self) -> u16 {
        self.question_count
    }

    pub fn answer_count(&self) -> u16 {
        self.answer_count
    }

    pub fn authority_count(&self) -> u16 {
        self.authoritative_count
    }

    pub fn additional_count(&self) -> u16 {
        self.additional_count
    }
}

pub struct HeaderBuilder {
    id: u16,
    flags: HeaderFlags,
    question_count: u16,
    answer_count: u16,
    authoritative_count: u16,
    additional_count: u16,
}

impl HeaderBuilder {
    pub fn new(flags: HeaderFlags) -> Self {
        Self {
            id: 0,
            flags,
            question_count: 0,
            answer_count: 0,
            additional_count: 0,
            authoritative_count: 0,
        }
    }

    pub fn id(&mut self, id: u16) -> &mut Self {
        self.id = id;
        self
    }

    pub fn question_count(&mut self, question_count: u16) -> &mut Self {
        self.question_count = question_count;
        self
    }

    pub fn answer_count(&mut self, answer_count: u16) -> &mut Self {
        self.answer_count = answer_count;
        self
    }

    pub fn additional_count(&mut self, additional_count: u16) -> &mut Self {
        self.additional_count = additional_count;
        self
    }

    pub fn authoritative_count(&mut self, authoritative_count: u16) -> &mut Self {
        self.authoritative_count = authoritative_count;
        self
    }

    pub fn build(&mut self) -> Header {
        Header {
            id: self.id,
            flags: self.flags,
            question_count: self.question_count,
            answer_count: self.answer_count,
            authoritative_count: self.authoritative_count,
            additional_count: self.additional_count,
        }
    }
}

#[cfg(test)]
mod test {
    use crate::packet::bin_reader::BinReader;
    use crate::packet::header::{Header, HeaderError};
    use crate::packet::header_flags::{HeaderFlags, Opcode, Rcode, QR};

    #[test]
    fn read_query_header_success() {
        let packet_bytes: [u8; 12] = [
            0xf2, 0xe8, 0x01, 0x00, 0x00, 0x01, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
        ];

        let expected_header_flags = HeaderFlags::builder()
            .query_or_response(QR::Query)
            .opcode(Opcode::Query)
            .authoritative_answer(false)
            .truncation(false)
            .recursion_desired(true)
            .recursion_available(false)
            .response_code(Rcode::NoError)
            .build();

        let expected_header = Header::builder(expected_header_flags)
            .id(62184)
            .question_count(1)
            .answer_count(0)
            .authoritative_count(0)
            .additional_count(0)
            .build();

        let mut decoder = BinReader::new(&packet_bytes);
        let header = Header::from_bytes(&mut decoder).unwrap();

        assert_eq!(header, expected_header);
    }

    #[test]
    fn read_response_header_success() {
        let packet_bytes: [u8; 12] = [
            0xf2, 0xe8, 0x81, 0x80, 0x00, 0x01, 0x00, 0x01, 0x00, 0x00, 0x00, 0x00,
        ];

        let expected_header_flags = HeaderFlags::builder()
            .query_or_response(QR::Response)
            .opcode(Opcode::Query)
            .authoritative_answer(false)
            .truncation(false)
            .recursion_desired(true)
            .recursion_available(true)
            .response_code(Rcode::NoError)
            .build();

        let expected_header = Header::builder(expected_header_flags)
            .id(62184)
            .question_count(1)
            .answer_count(1)
            .authoritative_count(0)
            .additional_count(0)
            .build();

        let mut decoder = BinReader::new(&packet_bytes);
        let header = Header::from_bytes(&mut decoder).unwrap();

        assert_eq!(header, expected_header);
    }

    #[test]
    fn read_header_insufficient_data() {
        let packet_bytes: [u8; 2] = [0xf2, 0xe8];

        let mut decoder = BinReader::new(&packet_bytes);

        assert!(Header::from_bytes(&mut decoder).is_err_and(|e| {
            match e {
                HeaderError::InsufficientData(_) => true,
                _ => false,
            }
        }));
    }
}
