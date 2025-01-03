use crate::packet::headers::header_flags::HeaderFlags;
use crate::packet::seder::{deserializer::Deserialize, serializer::Serialize, TryFromBytes, ToBytes};
use std::cmp::PartialEq;
use thiserror::Error;

type HeaderResult = Result<Header, HeaderError>;

#[derive(Error, Debug, PartialEq)]
pub enum HeaderError {
    #[error("Could not get {0} bytes from wire")]
    InsufficientData(usize),
    #[error("Unable to build flags, wrong combination of flags received")]
    FlagError,
    #[error("No XID provided for the message")]
    MissingId,
    #[error("Message has {0} questions, currently only supports 1")]
    IncorrectQuestionCount(u16),
}

// Different states for the Header builder
#[derive(Default)]
struct IdUnset;
#[derive(Default)]
struct IdSet(u16);

trait IdState {}
impl IdState for IdUnset {}
impl IdState for IdSet {}

#[derive(Default)]
struct FlagsUnset;
#[derive(Default)]
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

struct HeaderBuilder<I, F>
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

impl Header {
    // pub fn id(&self) -> u16 {
    //     self.id
    // }
    //
    // pub fn flags(&self) -> &HeaderFlags {
    //     &self.flags
    // }
    //
    // pub fn question_count(&self) -> u16 {
    //     self.question_count
    // }

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

impl TryFromBytes for Header {
    type Error = HeaderError;

    fn try_from_bytes(decoder: &mut Deserialize) -> HeaderResult {
        let id = decoder.read_u16().map_err(|_| HeaderError::MissingId)?;

        let flags = decoder
            .read_u16()
            .map_err(|_| HeaderError::InsufficientData(2))?;

        let flags =  HeaderFlags::try_from(flags).map_err(|_| HeaderError::FlagError)?;

        let question_count = decoder
            .read_u16()
            .map_err(|_| HeaderError::InsufficientData(2))?;

        match question_count {
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
}

impl ToBytes for Header {
    fn to_bytes(&self, encoder: &mut Serialize) {
        encoder.write_u16(self.id);
        self.flags.to_bytes(encoder);
        encoder.write_u16(self.question_count);
        encoder.write_u16(self.answer_count);
        encoder.write_u16(self.authoritative_count);
        encoder.write_u16(self.additional_count);
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
pub mod header_unittest {
    use crate::packet::headers::header::{Header, HeaderBuilder, HeaderError};
    use crate::packet::headers::header_flags::{
        header_flags_unittest::{generate_query_header_flags, generate_response_header_flag},
        Rcode,
    };
    use crate::packet::seder::{deserializer::Deserialize, serializer::Serialize, TryFromBytes, ToBytes};

    pub fn get_response_header(id: u16) -> Header {
        let expected_header_flags =
            generate_response_header_flag(false, false, true, true, Rcode::NoError);

        HeaderBuilder::new()
            .id(id)
            .flags(expected_header_flags)
            .question_count(1)
            .answer_count(1)
            .authoritative_count(0)
            .additional_count(0)
            .build()
    }

    #[test]
    fn read_query_header() {
        let packet_bytes: [u8; 12] = [
            0xf2, 0xe8, 0x01, 0x00, 0x00, 0x01, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
        ];

        let expected_header_flags = generate_query_header_flags(true);

        let expected_header = HeaderBuilder::new()
            .id(62184)
            .flags(expected_header_flags)
            .question_count(1)
            .answer_count(0)
            .authoritative_count(0)
            .additional_count(0)
            .build();

        let mut decoder = Deserialize::new(&packet_bytes);
        let header = Header::try_from_bytes(&mut decoder).unwrap();

        assert_eq!(header, expected_header);
    }

    #[test]
    fn header_with_max_id() {
        let packet_bytes: [u8; 12] = [
            0xff, 0xff, 0x01, 0x00, 0x00, 0x01, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
        ];

        let expected_header_flags = generate_query_header_flags(true);

        let expected_header = HeaderBuilder::new()
            .id(65535)
            .flags(expected_header_flags)
            .question_count(1)
            .answer_count(0)
            .authoritative_count(0)
            .additional_count(0)
            .build();

        let mut decoder = Deserialize::new(&packet_bytes);
        let header = Header::try_from_bytes(&mut decoder).unwrap();

        assert_eq!(header, expected_header);
    }

    #[test]
    fn read_response_header() {
        let packet_bytes: [u8; 12] = [
            0xf2, 0xe8, 0x81, 0x80, 0x00, 0x01, 0x00, 0x01, 0x00, 0x00, 0x00, 0x00,
        ];

        let expected_header_flags =
            generate_response_header_flag(false, false, true, true, Rcode::NoError);

        let expected_header = HeaderBuilder::new()
            .id(62184)
            .flags(expected_header_flags)
            .question_count(1)
            .answer_count(1)
            .authoritative_count(0)
            .additional_count(0)
            .build();

        let mut decoder = Deserialize::new(&packet_bytes);
        let header = Header::try_from_bytes(&mut decoder).unwrap();

        assert_eq!(header, expected_header);
    }

    #[test]
    fn error_header_insufficient_data() {
        let packet_bytes: [u8; 2] = [0xf2, 0xe8];

        let mut decoder = Deserialize::new(&packet_bytes);

        assert_eq!(Header::try_from_bytes(&mut decoder), Err(HeaderError::InsufficientData(2)));
    }

    #[test]
    fn error_multi_question() {
        let wire_data: [u8; 12] = [
            0xf2, 0xe8, 0x01, 0x00, 0x00, 0x04, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
        ];

        let mut decoder = Deserialize::new(&wire_data);

        assert_eq!(Header::try_from_bytes(&mut decoder), Err(HeaderError::IncorrectQuestionCount(4)));
    }

    #[test]
    fn serialize_query_header() {
        let expected_packet_bytes: [u8; 12] = [
            0xf2, 0xe8, 0x01, 0x00, 0x00, 0x01, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
        ];

        let header_flags = generate_query_header_flags(true);

        let header = HeaderBuilder::new()
            .id(62184)
            .flags(header_flags)
            .question_count(1)
            .answer_count(0)
            .authoritative_count(0)
            .additional_count(0)
            .build();

        let mut encoder = Serialize::new();
        header.to_bytes(&mut encoder);

        assert_eq!(encoder.bin_data(), expected_packet_bytes)
    }
}
