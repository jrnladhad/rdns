use thiserror::Error;

#[derive(Error, Debug)]
pub enum HeaderFlagError {
    #[error("Invalid Opcode provided")]
    MalformedOpcode,
    #[error("Rcode is malformed")]
    MalformedRcode,
    #[error("Zero flag has data other than 0")]
    ZeroFlagUnset,
    #[error("A query cannot have RA bit set")]
    QueryWithRABitSet
}

#[derive(PartialEq, Copy, Clone, Debug)]
pub enum QR {
    Query,
    Response
}

impl Default for QR {
    fn default() -> Self {
        QR::Query
    }
}

impl From<u16> for QR {
    fn from(value: u16) -> Self {
        match value {
            0 => QR::Query,
            1 => QR::Response,
            _ => QR::Query
        }
    }
}

#[derive(PartialEq, Copy, Clone, Debug)]
pub enum Opcode {
    Query,
    Iquery,
    Status
}

impl Default for Opcode {
    fn default() -> Self {
        Opcode::Query
    }
}

impl TryFrom<u16> for Opcode {
    type Error = HeaderFlagError;
    fn try_from(value: u16) -> Result<Self, Self::Error> {
        match value {
            0 => Ok(Opcode::Query),
            1 => Ok(Opcode::Iquery),
            2 => Ok(Opcode::Status),
            _ => Err(HeaderFlagError::MalformedOpcode)
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
    Refused
}

impl Default for Rcode {
    fn default() -> Self {
        Rcode::NoError
    }
}

impl TryFrom<u16> for Rcode {
    type Error = HeaderFlagError;
    fn try_from(value: u16) -> Result<Self, Self::Error> {
        match value {
            0 => Ok(Rcode::NoError),
            1 => Ok(Rcode::FormatError),
            2 => Ok(Rcode::ServerFailure),
            3 => Ok(Rcode::NameError),
            4 => Ok(Rcode::NotImplemented),
            5 => Ok(Rcode::Refused),
            _ => Err(HeaderFlagError::MalformedRcode)
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
            query_or_response: QR::default(),
            opcode: Opcode::default(),
            authoritative_answer: bool::default(),
            truncation: bool::default(),
            recursion_desired: bool::default(),
            recursion_available: bool::default(),
            zero: u8::default(),
            response_code: Rcode::default()
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
    type Error = HeaderFlagError;
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

        let opcode = Opcode::try_from((value & OPCODE_MASK) >> 11)?;

        let authoritative_answer = (value & AA_MASK) >> 10 == 1;
        let truncation = (value & TC_MASK) >> 9 == 1;
        let recursion_desired = (value & RD_MASK) >> 8 == 1;
        let recursion_available = (value & RA_MASK) >> 7 == 1;

        let zero = ((value & ZERO_MASK) >> 4) as u8;

        let response_code = Rcode::try_from(value & RC_MASK)?;

        if zero != 0 {
            return Err(HeaderFlagError::ZeroFlagUnset)
        }

        if query_or_response == QR::Query && recursion_available {
            return Err(HeaderFlagError::QueryWithRABitSet)
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