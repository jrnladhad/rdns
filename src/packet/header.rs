// standard crates
use std::cmp::PartialEq;

// external crates
use thiserror::Error;

// custom crates
use super::bin_reader::BinReader;

#[derive(Error, Debug)]
pub enum HeaderError {
    #[error("Could not get {0} bytes from wire")]
    InsufficientData(usize),
    #[error("Zero flag has data other than 0")]
    ZeroFlagUnset,
    #[error("A query cannot have RA bit set")]
    QueryWithRABitSet,
    #[error("QR bit is malformed")]
    MalformedQRBit,
    #[error("Opcode is malformed")]
    MalformedOpcode,
    #[error("Rcode is malformed")]
    MalformedRcode
}

pub(crate) type HeaderResult<T> = Result<T, HeaderError>;

#[derive(PartialEq, Copy, Clone, Debug)]
pub enum QR {
    Query,
    Response,
    Unknown
}

impl Default for QR {
    fn default() -> Self {
        QR::Unknown
    }
}

impl From<u16> for QR {
    fn from(value: u16) -> Self {
        match value {
            0 => QR::Query,
            1 => QR::Response,
            _ => QR::Unknown
        }
    }
}

#[derive(PartialEq, Copy, Clone, Debug)]
pub enum Opcode {
    Query,
    Iquery,
    Status,
    Unknown
}

impl Default for Opcode {
    fn default() -> Self {
        Opcode::Unknown
    }
}

impl From<u16> for Opcode {
    fn from(value: u16) -> Self {
        match value {
            0 => Opcode::Query,
            1 => Opcode::Iquery,
            2 => Opcode::Status,
            _ => Opcode::Unknown
        }
    }
}

#[derive(PartialEq, Copy, Clone, Debug)]
pub enum Rcode {
    NoError,
    FormatError,
    ServerFailure,
    NameError,
    NotImplemented,
    Refused,
    Unknown
}

impl Default for Rcode {
    fn default() -> Self {
        Rcode::Unknown
    }
}

impl From<u16> for Rcode {
    fn from(value: u16) -> Self {
        match value {
            0 => Rcode::NoError,
            1 => Rcode::FormatError,
            2 => Rcode::ServerFailure,
            3 => Rcode::NameError,
            4 => Rcode::NotImplemented,
            5 => Rcode::Refused,
            _ => Rcode::Unknown
        }
    }
}

#[derive(Copy, Clone, Debug, PartialEq)]
pub struct HeaderFlags {
    query_or_response: QR,
    opcode: Opcode,
    authoritative_answer: bool,
    truncation: bool,
    recursion_desired: bool,
    recursion_available: bool,
    zero: u8,
    response_code: Rcode
}

pub struct HeaderFlagsBuilder {
    query_or_response: QR,
    opcode: Opcode,
    authoritative_answer: bool,
    truncation: bool,
    recursion_desired: bool,
    recursion_available: bool,
    zero: u8,
    response_code: Rcode
}

impl HeaderFlagsBuilder {
    pub fn new() -> Self {
        Self {
            query_or_response: QR::Unknown,
            opcode: Opcode::Unknown,
            authoritative_answer: false,
            truncation: false,
            recursion_desired: false,
            recursion_available: false,
            zero: 0,
            response_code: Rcode::Unknown
        }
    }

    pub fn query_or_response(&mut self, qr: QR) -> &mut Self{
        self.query_or_response = qr;
        self
    }

    pub fn opcode(&mut self, opcode: Opcode) -> &mut Self{
        self.opcode= opcode;
        self
    }

    pub fn authoritative_answer(&mut self, authoritative_answer: bool) -> &mut Self{
        self.authoritative_answer = authoritative_answer;
        self
    }

    pub fn truncation(&mut self, truncation: bool) -> &mut Self{
        self.truncation = truncation;
        self
    }

    pub fn recursion_desired(&mut self, recursion_desired: bool) -> &mut Self {
        self.recursion_desired = recursion_desired;
        self
    }

    pub fn recursion_available(&mut self, recursion_available: bool) -> &mut Self{
        self.recursion_available = recursion_available;
        self
    }

    pub fn response_code(&mut self, response_code: Rcode) -> &mut Self{
        self.response_code = response_code;
        self
    }

    pub fn build(&mut self) -> HeaderFlags {
        HeaderFlags {
            query_or_response: self.query_or_response,
            opcode: self.opcode,
            authoritative_answer: self.authoritative_answer,
            truncation: self.truncation,
            recursion_desired: self.recursion_desired,
            recursion_available: self.recursion_available,
            zero: self.zero,
            response_code: self.response_code
        }
    }
}

impl TryFrom<u16> for HeaderFlags {
    type Error = HeaderError;
    fn try_from(value: u16) -> Result<Self, Self::Error> {
        const QR_MASK: u16 = 1 << 15;
        const OPCODE_MASK: u16 = 1 << 11;
        const AA_MASK: u16 = 1 << 10;
        const TC_MASK: u16 = 1 << 9;
        const RD_MASK: u16 = 1 << 8;
        const RA_MASK: u16 = 1 << 7;
        const ZERO_MASK: u16 = 7 << 4;
        const RC_MASK: u16 = 15;

        let query_or_response = QR::from((value & QR_MASK) >> 15);

        let opcode = Opcode::from((value & OPCODE_MASK) >> 11);

        let authoritative_answer = (value & AA_MASK) >> 10 == 1;
        let truncation = (value & TC_MASK) >> 9 == 1;
        let recursion_desired = (value & RD_MASK) >> 8 == 1;
        let recursion_available = (value & RA_MASK) >> 7 == 1;

        let zero = ((value & ZERO_MASK) >> 4) as u8;

        let response_code = Rcode::from(value & RC_MASK);

        if zero != 0 {
            return Err(HeaderError::ZeroFlagUnset)
        }

        if query_or_response == QR::Query && recursion_available {
            return Err(HeaderError::QueryWithRABitSet)
        }

        if query_or_response == QR::Unknown {
            return Err(HeaderError::MalformedQRBit)
        }

        if opcode == Opcode::Unknown {
            return Err(HeaderError::MalformedOpcode)
        }

        if response_code == Rcode::Unknown {
            return Err(HeaderError::MalformedRcode)
        }

        Ok(HeaderFlags {
            query_or_response,
            opcode,
            authoritative_answer,
            truncation,
            recursion_desired,
            recursion_available,
            zero,
            response_code
        })
    }
}

impl HeaderFlags {
    pub fn builder() -> HeaderFlagsBuilder {
        HeaderFlagsBuilder::new()
    }

    pub fn query_or_response(&self) -> &QR {
        &self.query_or_response
    }

    pub fn opcode(&self) -> &Opcode {
        &self.opcode
    }

    pub fn authoritative_answer(&self) -> bool {
        self.authoritative_answer
    }

    pub fn truncation(&self) -> bool {
        self.truncation
    }

    pub fn recursion_desired(&self) -> bool {
        self.recursion_desired
    }

    pub fn recursion_available(&self) -> bool {
        self.recursion_available
    }

    pub fn response_code(&self) -> &Rcode {
        &self.response_code
    }
}

#[derive(Debug, PartialEq)]
pub struct Header {
    id: u16,
    flags: HeaderFlags,
    question_count: u16,
    answer_count: u16,
    authoritative_count: u16,
    additional_count: u16
}

pub struct HeaderBuilder {
    id: u16,
    flags: HeaderFlags,
    question_count: u16,
    answer_count: u16,
    authoritative_count: u16,
    additional_count: u16
}

impl HeaderBuilder {
    pub fn new(flags: HeaderFlags) -> Self {
        Self {
            id: 0,
            flags,
            question_count: 0,
            answer_count: 0,
            additional_count: 0,
            authoritative_count: 0
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

    pub fn flags(&mut self, flags: HeaderFlags) -> &mut Self {
        self.flags = flags;
        self
    }

    pub fn build(&mut self) -> Header {
        Header {
            id: self.id,
            flags: self.flags,
            question_count: self.question_count,
            answer_count: self.answer_count,
            authoritative_count: self.authoritative_count,
            additional_count: self.additional_count
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

        let flags = HeaderFlags::try_from(flags)?;
        
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

        Ok(Header{
            id,
            flags,
            question_count,
            answer_count,
            authoritative_count,
            additional_count
        })
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

#[cfg(test)]
mod test
{
    use crate::packet::bin_reader::BinReader;
    use crate::packet::header::{Header, HeaderBuilder, HeaderFlags, HeaderFlagsBuilder, Opcode, Rcode, QR};

    #[test]
    fn read_header_success()
    {
        let packet_bytes: [u8; 12] = [
            0xf2, 0xe8, 0x01, 0x00,
            0x00, 0x01, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00
        ];

        let expected_header_flags = HeaderFlagsBuilder::new()
            .query_or_response(QR::Query)
            .opcode(Opcode::Query)
            .authoritative_answer(false)
            .truncation(false)
            .recursion_desired(false)
            .recursion_available(false)
            .response_code(Rcode::NoError)
            .build();

        let expected_header = HeaderBuilder::new(expected_header_flags)
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
}