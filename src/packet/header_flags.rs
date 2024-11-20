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
    QueryWithRABitSet,
}

type HeaderFlagsBuilderUnset = HeaderFlagsBuilder<QrUnset, OpcodeUnset, AaUnset, TcUnset, RdUnset, RaUnset, RcodeUnset>;
type HeaderFlagsBuilderQrSet = HeaderFlagsBuilder<QrSet, OpcodeUnset, AaUnset, TcUnset, RdUnset, RaUnset, RcodeUnset>;
type HeaderFlagsBuilderOpcodeSet = HeaderFlagsBuilder<QrSet, OpcodeSet, AaUnset, TcUnset, RdUnset, RaUnset, RcodeUnset>;
type HeaderFlagsBuilderAaSet = HeaderFlagsBuilder<QrSet, OpcodeSet, AaSet, TcUnset, RdUnset, RaUnset, RcodeUnset>;
type HeaderFlagsBuilderTcSet = HeaderFlagsBuilder<QrSet, OpcodeSet, AaSet, TcSet, RdUnset, RaUnset, RcodeUnset>;
type HeaderFlagsBuilderRdSet = HeaderFlagsBuilder<QrSet, OpcodeSet, AaSet, TcSet, RdSet, RaUnset, RcodeUnset>;
type HeaderFlagsBuilderRaSet = HeaderFlagsBuilder<QrSet, OpcodeSet, AaSet, TcSet, RdSet, RaSet, RcodeUnset>;
type HeaderFlagsBuilderSet = HeaderFlagsBuilder<QrSet, OpcodeSet, AaSet, TcSet, RdSet, RaSet, RcodeSet>;

#[derive(PartialEq, Copy, Clone, Debug)]
pub enum QR {
    Query,
    Response,
}

impl Default for QR {
    fn default() -> Self {
        QR::Query
    }
}

impl From<bool> for QR {
    fn from(value: bool) -> Self {
        match value {
            true => QR::Response,
            false => QR::Query
        }
    }
}

#[derive(PartialEq, Copy, Clone, Debug)]
pub enum Opcode {
    Query,
    Iquery,
    Status,
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
            _ => Err(HeaderFlagError::MalformedOpcode),
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
            _ => Err(HeaderFlagError::MalformedRcode),
        }
    }
}

#[derive(Clone, Debug, PartialEq, Default)]
pub struct HeaderFlags {
    query_or_response: QR,
    opcode: Opcode,
    authoritative_answer: bool,
    truncation: bool,
    recursion_desired: bool,
    recursion_available: bool,
    zero: u8,
    response_code: Rcode,
}

#[derive(Default, Clone)]
pub struct QrUnset;
#[derive(Default, Clone)]
pub struct QrSet(QR);

trait QrState {}
impl QrState for QrUnset {}
impl QrState for QrSet {}

#[derive(Default, Clone)]
pub struct OpcodeUnset;
#[derive(Default, Clone)]
pub struct OpcodeSet(Opcode);

trait OpcodeState {}
impl OpcodeState for OpcodeUnset {}
impl OpcodeState for OpcodeSet {}

#[derive(Default, Clone)]
pub struct AaUnset;
#[derive(Default, Clone)]
pub struct AaSet(bool);

trait AaState {}
impl AaState for AaUnset {}
impl AaState for AaSet {}

#[derive(Default, Clone)]
pub struct TcUnset;
#[derive(Default, Clone)]
pub struct TcSet(bool);

trait TcState {}
impl TcState for TcUnset {}
impl TcState for TcSet {}

#[derive(Default, Clone)]
pub struct RdUnset;
#[derive(Default, Clone)]
pub struct RdSet(bool);

trait RdState {}
impl RdState for RdUnset {}
impl RdState for RdSet {}

#[derive(Default, Clone)]
pub struct RaUnset;
#[derive(Default, Clone)]
pub struct RaSet(bool);

trait RaState {}
impl RaState for RaUnset {}
impl RaState for RaSet {}

#[derive(Default, Clone)]
pub struct RcodeUnset;
#[derive(Default, Clone)]
pub struct RcodeSet(Rcode);

trait RcodeState {}
impl RcodeState for RcodeUnset {}
impl RcodeState for RcodeSet {}

#[derive(Default, Clone)]
pub struct HeaderFlagsBuilder<Q, O, A, T, RD, RA, RC>
where
    Q: QrState,
    O: OpcodeState,
    A: AaState,
    T: TcState,
    RD: RdState,
    RA: RaState,
    RC: RcodeState
{
    query_or_response: Q,
    opcode: O,
    authoritative_answer: A,
    truncation: T,
    recursion_desired: RD,
    recursion_available: RA,
    zero: u8,
    response_code: RC,
}

impl HeaderFlagsBuilderUnset {
    pub fn new () -> Self {
        HeaderFlagsBuilder::default()
    }

    pub fn query_or_response(self, qr: QR) -> HeaderFlagsBuilderQrSet {
        HeaderFlagsBuilder {
            query_or_response: QrSet(qr),
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

impl HeaderFlagsBuilderQrSet {
    pub fn opcode(self, opcode: Opcode) -> HeaderFlagsBuilderOpcodeSet {
        HeaderFlagsBuilder {
            query_or_response: self.query_or_response,
            opcode: OpcodeSet(opcode),
            authoritative_answer: self.authoritative_answer,
            truncation: self.truncation,
            recursion_desired: self.recursion_desired,
            recursion_available: self.recursion_available,
            zero: self.zero,
            response_code: self.response_code
        }
    }
}

impl HeaderFlagsBuilderOpcodeSet {
    pub fn authoritative_answer(self, authoritative_answer: bool) -> HeaderFlagsBuilderAaSet {
        HeaderFlagsBuilder {
            query_or_response: self.query_or_response,
            opcode: self.opcode,
            authoritative_answer: AaSet(authoritative_answer),
            truncation: self.truncation,
            recursion_desired: self.recursion_desired,
            recursion_available: self.recursion_available,
            zero: self.zero,
            response_code: self.response_code
        }
    }
}

impl HeaderFlagsBuilderAaSet {
    pub fn truncation(self, truncation: bool) -> HeaderFlagsBuilderTcSet {
        HeaderFlagsBuilder {
            query_or_response: self.query_or_response,
            opcode: self.opcode,
            authoritative_answer: self.authoritative_answer,
            truncation: TcSet(truncation),
            recursion_desired: self.recursion_desired,
            recursion_available: self.recursion_available,
            zero: self.zero,
            response_code: self.response_code
        }
    }
}

impl HeaderFlagsBuilderTcSet {
    pub fn recursion_desired(self, recursion_desired: bool) -> HeaderFlagsBuilderRdSet {
        HeaderFlagsBuilder {
            query_or_response: self.query_or_response,
            opcode: self.opcode,
            authoritative_answer: self.authoritative_answer,
            truncation: self.truncation,
            recursion_desired: RdSet(recursion_desired),
            recursion_available: self.recursion_available,
            zero: self.zero,
            response_code: self.response_code
        }
    }
}

impl HeaderFlagsBuilderRdSet {
    pub fn recursion_available(self, recursion_available: bool) -> HeaderFlagsBuilderRaSet {
        HeaderFlagsBuilder {
            query_or_response: self.query_or_response,
            opcode: self.opcode,
            authoritative_answer: self.authoritative_answer,
            truncation: self.truncation,
            recursion_desired: self.recursion_desired,
            recursion_available: RaSet(recursion_available),
            zero: self.zero,
            response_code: self.response_code
        }
    }
}

impl HeaderFlagsBuilderRaSet {
    pub fn response_code(self, response_code: Rcode) -> HeaderFlagsBuilderSet {
        HeaderFlagsBuilder {
            query_or_response: self.query_or_response,
            opcode: self.opcode,
            authoritative_answer: self.authoritative_answer,
            truncation: self.truncation,
            recursion_desired: self.recursion_desired,
            recursion_available: self.recursion_available,
            zero: self.zero,
            response_code: RcodeSet(response_code)
        }
    }
}

impl HeaderFlagsBuilderSet {
    pub fn build(self) -> HeaderFlags {
        HeaderFlags {
            query_or_response: self.query_or_response.0,
            opcode: self.opcode.0,
            authoritative_answer: self.authoritative_answer.0,
            truncation: self.truncation.0,
            recursion_desired: self.recursion_desired.0,
            recursion_available: self.recursion_available.0,
            zero: self.zero,
            response_code: self.response_code.0
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

        let query_or_response = QR::from((value & QR_MASK) != 0);

        let opcode = Opcode::try_from((value & OPCODE_MASK) >> 11)?;

        let authoritative_answer = (value & AA_MASK) >> 10 == 1;
        let truncation = (value & TC_MASK) >> 9 == 1;
        let recursion_desired = (value & RD_MASK) >> 8 == 1;
        let recursion_available = (value & RA_MASK) >> 7 == 1;

        let zero = ((value & ZERO_MASK) >> 4) as u8;

        let response_code = Rcode::try_from(value & RC_MASK)?;

        if zero != 0 {
            return Err(HeaderFlagError::ZeroFlagUnset);
        }

        if query_or_response == QR::Query && recursion_available {
            return Err(HeaderFlagError::QueryWithRABitSet);
        }

        let header_flags = HeaderFlagsBuilder::new()
            .query_or_response(query_or_response)
            .opcode(opcode)
            .authoritative_answer(authoritative_answer)
            .truncation(truncation)
            .recursion_desired(recursion_desired)
            .recursion_available(recursion_available)
            .response_code(response_code)
            .build();

        Ok(header_flags)
    }
}

impl HeaderFlags {
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
