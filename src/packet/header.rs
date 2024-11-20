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
    #[error("No XID provided for the message")]
    MissingId,
    #[error("Message has {0} questions, currently only supports 1")]
    IncorrectQuestionCount(u16),
}

type HeaderResult<T> = Result<T, HeaderError>;

// Different states for the Header builder
#[derive(Default, Clone)]
struct IdUnset;
#[derive(Default, Clone)]
struct IdSet(u16);

trait IdState {}
impl IdState for IdUnset {}
impl IdState for IdSet {}

#[derive(Default, Clone)]
struct FlagsUnset;
#[derive(Default, Clone)]
struct FlagsSet(HeaderFlags);

trait FlagState {}
impl FlagState for FlagsUnset {}
impl FlagState for FlagsSet {}

#[derive(Debug, PartialEq)]
pub struct Header {
    id: u16,
    flags: HeaderFlags,
    question_count: u16,
    answer_count: u16,
    authoritative_count: u16,
    additional_count: u16,
}

#[derive(Clone)]
pub struct HeaderBuilder<I, F>
where
    I: IdState,
    F: FlagState,
{
    id: I,
    flags: F,
    question_count: u16,
    answer_count: u16,
    authoritative_count: u16,
    additional_count: u16,
}

impl Default for HeaderBuilder<IdUnset, FlagsUnset> {
    fn default() -> Self {
        HeaderBuilder {
            id: IdUnset,
            flags: FlagsUnset,
            question_count: 0,
            answer_count: 0,
            authoritative_count: 0,
            additional_count: 0,
        }
    }
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

        match question_count {
            0 => return Err(HeaderError::IncorrectQuestionCount(0)),
            1 => question_count,
            _ => return Err(HeaderError::IncorrectQuestionCount(question_count)),
        };

        let answer_count = decoder
            .read_u16()
            .map_err(|_| HeaderError::InsufficientData(2))?;

        let authoritative_count = decoder
            .read_u16()
            .map_err(|_| HeaderError::InsufficientData(2))?;

        let additional_count = decoder
            .read_u16()
            .map_err(|_| HeaderError::InsufficientData(2))?;

        let header = HeaderBuilder::new()
            .id(id)
            .flags(flags)
            .question_count(question_count)
            .answer_count(answer_count)
            .authoritative_count(authoritative_count)
            .additional_count(additional_count)
            .build();

        Ok(header)
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

impl HeaderBuilder<IdUnset, FlagsUnset> {
    pub fn new() -> Self {
        HeaderBuilder::default()
    }

    pub fn id(self, id: u16) -> HeaderBuilder<IdSet, FlagsUnset> {
        HeaderBuilder {
            id: IdSet(id),
            flags: self.flags,
            question_count: self.question_count,
            answer_count: self.answer_count,
            authoritative_count: self.authoritative_count,
            additional_count: self.additional_count,
        }
    }
}

impl HeaderBuilder<IdSet, FlagsUnset> {
    pub fn flags(self, flags: HeaderFlags) -> HeaderBuilder<IdSet, FlagsSet> {
        HeaderBuilder {
            id: self.id,
            flags: FlagsSet(flags),
            question_count: self.question_count,
            answer_count: self.answer_count,
            authoritative_count: self.authoritative_count,
            additional_count: self.additional_count,
        }
    }
}

impl HeaderBuilder<IdSet, FlagsSet> {
    pub fn question_count(mut self, question_count: u16) -> Self {
        self.question_count = question_count;
        self
    }

    pub fn answer_count(mut self, answer_count: u16) -> Self {
        self.answer_count = answer_count;
        self
    }

    pub fn authoritative_count(mut self, authoritative_count: u16) -> Self {
        self.authoritative_count = authoritative_count;
        self
    }

    pub fn additional_count(mut self, additional_count: u16) -> Self {
        self.additional_count = additional_count;
        self
    }

    pub fn build(self) -> Header {
        Header {
            id: self.id.0,
            flags: self.flags.0,
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
    use crate::packet::header::{Header, HeaderBuilder, HeaderError};
    use crate::packet::header_flags::{HeaderFlagsBuilder, Opcode, Rcode, QR};

    #[test]
    fn read_query_header_success() {
        let packet_bytes: [u8; 12] = [
            0xf2, 0xe8, 0x01, 0x00, 0x00, 0x01, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
        ];

        let expected_header_flags = HeaderFlagsBuilder::new()
            .query_or_response(QR::Query)
            .opcode(Opcode::Query)
            .authoritative_answer(false)
            .truncation(false)
            .recursion_desired(true)
            .recursion_available(false)
            .response_code(Rcode::NoError)
            .build();

        let expected_header = HeaderBuilder::new()
            .id(62184)
            .flags(expected_header_flags)
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

        let expected_header_flags = HeaderFlagsBuilder::new()
            .query_or_response(QR::Response)
            .opcode(Opcode::Query)
            .authoritative_answer(false)
            .truncation(false)
            .recursion_desired(true)
            .recursion_available(true)
            .response_code(Rcode::NoError)
            .build();

        let expected_header = HeaderBuilder::new()
            .id(62184)
            .flags(expected_header_flags)
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
