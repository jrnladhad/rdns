use thiserror::Error;
use crate::packet::seder::{serializer::Serialize, ToBytes};

// For serialization
const SET_QUESTION: u16 = 0 << 15;
const SET_RESPONSE: u16 = 1 << 15;

const SET_QUERY: u16 = 0 << 11;
const SET_IQUERY: u16 = 1 << 11;
const SET_STATUS: u16 = 2 << 11;

const SET_AA: u16 = 1 << 10;

const SET_TC: u16 = 1 << 9;

const SET_RD: u16 = 1 << 8;

const SET_RA: u16 = 1 << 7;

const SET_NO_ERROR: u16 = 0;
const SET_FORMAT_ERROR: u16 = 1;
const SET_SERVER_FAILURE: u16 = 2;
const SET_NAME_ERROR: u16 = 3;
const SET_NOT_IMPLEMENTED: u16 = 4;
const SET_REFUSED: u16 = 5;

// For deserialization
const QR_MASK: u16 = 1 << 15;
const OPCODE_MASK: u16 = 1 << 11;
const AA_MASK: u16 = 1 << 10;
const TC_MASK: u16 = 1 << 9;
const RD_MASK: u16 = 1 << 8;
const RA_MASK: u16 = 1 << 7;
const ZERO_MASK: u16 = 7 << 4;
const RC_MASK: u16 = 15;

type HeaderFlagsBuilderUnset =
    HeaderFlagsBuilder<QrUnset, OpcodeUnset, AaUnset, TcUnset, RdUnset, RaUnset, RcodeUnset>;
type HeaderFlagsBuilderQrSet =
    HeaderFlagsBuilder<QrSet, OpcodeUnset, AaUnset, TcUnset, RdUnset, RaUnset, RcodeUnset>;
type HeaderFlagsBuilderOpcodeSet =
    HeaderFlagsBuilder<QrSet, OpcodeSet, AaUnset, TcUnset, RdUnset, RaUnset, RcodeUnset>;
type HeaderFlagsBuilderAaSet =
    HeaderFlagsBuilder<QrSet, OpcodeSet, AaSet, TcUnset, RdUnset, RaUnset, RcodeUnset>;
type HeaderFlagsBuilderTcSet =
    HeaderFlagsBuilder<QrSet, OpcodeSet, AaSet, TcSet, RdUnset, RaUnset, RcodeUnset>;
type HeaderFlagsBuilderRdSet =
    HeaderFlagsBuilder<QrSet, OpcodeSet, AaSet, TcSet, RdSet, RaUnset, RcodeUnset>;
type HeaderFlagsBuilderRaSet =
    HeaderFlagsBuilder<QrSet, OpcodeSet, AaSet, TcSet, RdSet, RaSet, RcodeUnset>;
type HeaderFlagsBuilderSet =
    HeaderFlagsBuilder<QrSet, OpcodeSet, AaSet, TcSet, RdSet, RaSet, RcodeSet>;


// TODO: This is the type state I would ideally like to have to response as well
type HeaderFlagsBuildingQuery =
    HeaderFlagsBuilder<QrSet, OpcodeSet, AaSet, TcSet, RdUnset, RaSet, RcodeSet>;

#[derive(Error, Debug, PartialEq)]
pub enum HeaderFlagError {
    #[error("Invalid Opcode provided")]
    MalformedOpcode,
    #[error("Rcode is malformed")]
    MalformedRcode,
    #[error("Zero flag has data other than 0")]
    ZeroFlagUnset,
    #[error("A query cannot have RA bit set")]
    QueryWithRABitSet,
    #[error("The authoritative answer bit is set for a query")]
    QueryWithAABitSet,
    #[error("The truncation bit is set for a query")]
    QueryWithTCBitSet,
}

#[derive(PartialEq, Debug, Default)]
pub enum QR {
    #[default]
    Query,
    Response,
}

#[derive(PartialEq, Debug, Default)]
pub enum Opcode {
    #[default]
    Query,
    Iquery,
    Status,
}

#[derive(PartialEq, Copy, Clone, Debug, Default)]
pub enum Rcode {
    #[default]
    NoError,
    FormatError,
    ServerFailure,
    NameError,
    NotImplemented,
    Refused,
}

#[derive(Default)]
struct QrUnset;
#[derive(Default)]
pub struct QrSet(QR);

trait QrState {}
impl QrState for QrUnset {}
impl QrState for QrSet {}

#[derive(Default)]
pub struct OpcodeUnset;
#[derive(Default)]
pub struct OpcodeSet(Opcode);

trait OpcodeState {}
impl OpcodeState for OpcodeUnset {}
impl OpcodeState for OpcodeSet {}

#[derive(Default)]
pub struct AaUnset;
#[derive(Default)]
pub struct AaSet(bool);

trait AaState {}
impl AaState for AaUnset {}
impl AaState for AaSet {}

#[derive(Default)]
pub struct TcUnset;
#[derive(Default)]
pub struct TcSet(bool);

trait TcState {}
impl TcState for TcUnset {}
impl TcState for TcSet {}

#[derive(Default)]
pub struct RdUnset;
#[derive(Default)]
pub struct RdSet(bool);

trait RdState {}
impl RdState for RdUnset {}
impl RdState for RdSet {}

#[derive(Default)]
pub struct RaUnset;
#[derive(Default)]
pub struct RaSet(bool);

trait RaState {}
impl RaState for RaUnset {}
impl RaState for RaSet {}

#[derive(Default)]
pub struct RcodeUnset;
#[derive(Default)]
pub struct RcodeSet(Rcode);

trait RcodeState {}
impl RcodeState for RcodeUnset {}
impl RcodeState for RcodeSet {}

#[derive(Debug, PartialEq, Default)]
pub struct HeaderFlags {
    query_or_response: QR,
    opcode: Opcode,
    authoritative_answer: bool,
    truncation: bool,
    recursion_desired: bool,
    recursion_available: bool,
    response_code: Rcode,
}

#[derive(Default)]
struct HeaderFlagsBuilder<Q, O, A, T, RD, RA, RC>
where
    Q: QrState,
    O: OpcodeState,
    A: AaState,
    T: TcState,
    RD: RdState,
    RA: RaState,
    RC: RcodeState,
{
    query_or_response: Q,
    opcode: O,
    authoritative_answer: A,
    truncation: T,
    recursion_desired: RD,
    recursion_available: RA,
    response_code: RC,
}

impl HeaderFlags {
    pub fn truncation(&mut self, tc: bool) {
        self.truncation = tc
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

impl From<bool> for QR {
    fn from(value: bool) -> Self {
        match value {
            true => QR::Response,
            false => QR::Query,
        }
    }
}

impl TryFrom<u16> for HeaderFlags {
    type Error = HeaderFlagError;
    fn try_from(value: u16) -> Result<Self, Self::Error> {
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

        if query_or_response == QR::Query && authoritative_answer {
            return Err(HeaderFlagError::QueryWithAABitSet);
        }

        if query_or_response == QR::Query && truncation {
            return Err(HeaderFlagError::QueryWithTCBitSet);
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

impl ToBytes for HeaderFlags {
    fn to_bytes(&self, encoder: &mut Serialize) {
        let mut flags: u16 = 0;

        flags = match self.query_or_response {
            QR::Query => flags | SET_QUESTION,
            QR::Response => flags | SET_RESPONSE,
        };

        flags = match self.opcode {
            Opcode::Query => flags | SET_QUERY,
            Opcode::Iquery => flags | SET_IQUERY,
            Opcode::Status => flags | SET_STATUS,
        };

        flags = match self.authoritative_answer {
            true => flags | SET_AA,
            false => flags,
        };

        flags = match self.truncation {
            true => flags | SET_TC,
            false => flags,
        };

        flags = match self.recursion_desired {
            true => flags | SET_RD,
            false => flags,
        };

        flags = match self.recursion_available {
            true => flags | SET_RA,
            false => flags,
        };

        flags = match self.response_code {
            Rcode::NoError => flags | SET_NO_ERROR,
            Rcode::FormatError => flags | SET_FORMAT_ERROR,
            Rcode::ServerFailure => flags | SET_SERVER_FAILURE,
            Rcode::NameError => flags | SET_NAME_ERROR,
            Rcode::NotImplemented => flags | SET_NOT_IMPLEMENTED,
            Rcode::Refused => flags | SET_REFUSED,
        };

        encoder.write_u16(flags);
    }
}

impl HeaderFlagsBuilderUnset {
    pub fn new() -> Self {
        HeaderFlagsBuilder::default()
    }

    pub fn query(
        self,
    ) -> HeaderFlagsBuildingQuery {
        HeaderFlagsBuilder {
            query_or_response: QrSet(QR::Query),
            opcode: OpcodeSet(Opcode::Query),
            authoritative_answer: AaSet(false),
            truncation: TcSet(false),
            recursion_desired: self.recursion_desired,
            recursion_available: RaSet(false),
            response_code: RcodeSet(Rcode::NoError),
        }
    }

    // TODO: Think about this if we want specific response builders such as No_Error, SERVFAIL, ...
    // pub fn response(
    //     self,
    //     _query_header_flags: &HeaderFlags,
    // ) -> HeaderFlagsBuilder<QrSet, OpcodeSet, AaUnset, TcSet, RdSet, RaUnset, RcodeUnset> {
    //     todo!();
    // }

    pub fn query_or_response(self, qr: QR) -> HeaderFlagsBuilderQrSet {
        HeaderFlagsBuilder {
            query_or_response: QrSet(qr),
            opcode: self.opcode,
            authoritative_answer: self.authoritative_answer,
            truncation: self.truncation,
            recursion_desired: self.recursion_desired,
            recursion_available: self.recursion_available,
            response_code: self.response_code,
        }
    }
}

impl HeaderFlagsBuildingQuery {
    pub fn recursion_desired(self, rd: bool) -> HeaderFlagsBuilderSet {
        HeaderFlagsBuilder {
            query_or_response: self.query_or_response,
            opcode: self.opcode,
            authoritative_answer: self.authoritative_answer,
            truncation: self.truncation,
            recursion_desired: RdSet(rd),
            recursion_available: self.recursion_available,
            response_code: self.response_code,
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
            response_code: self.response_code,
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
            response_code: self.response_code,
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
            response_code: self.response_code,
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
            response_code: self.response_code,
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
            response_code: self.response_code,
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
            response_code: RcodeSet(response_code),
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
            response_code: self.response_code.0,
        }
    }
}

#[cfg(test)]
pub mod header_flags_unittest {
    use crate::packet::headers::header_flags::{
        HeaderFlagError, HeaderFlags, HeaderFlagsBuilder, Opcode, Rcode, QR,
    };
    use crate::packet::seder::{serializer::Serialize, ToBytes};

    pub fn generate_query_header_flags(rd: bool) -> HeaderFlags {
        HeaderFlagsBuilder::new()
            .query()
            .recursion_desired(rd)
            .build()
    }

    pub fn generate_response_header_flag(
        aa: bool,
        tc: bool,
        rd: bool,
        ra: bool,
        rcode: Rcode,
    ) -> HeaderFlags {
        HeaderFlagsBuilder::new()
            .query_or_response(QR::Response)
            .opcode(Opcode::Query)
            .authoritative_answer(aa)
            .truncation(tc)
            .recursion_desired(rd)
            .recursion_available(ra)
            .response_code(rcode)
            .build()
    }

    #[test]
    fn serialize_response_header_flag() {
        let expected_bin_data: Vec<u8> = vec![0x81, 0x80];

        let header_flags = generate_response_header_flag(false, false, true, true, Rcode::NoError);

        let mut encoder = Serialize::new();
        header_flags.to_bytes(&mut encoder);

        assert_eq!(encoder.bin_data(), expected_bin_data);
    }

    #[test]
    fn error_zero_flag_unset() {
        let header_flags: u16 = 0b0000_0000_0001_0000;

        assert_eq!(
            HeaderFlags::try_from(header_flags),
            Err(HeaderFlagError::ZeroFlagUnset)
        );
    }

    #[test]
    fn error_query_with_ra() {
        let header_flags: u16 = 0b0000_0000_1000_0000;

        assert_eq!(
            HeaderFlags::try_from(header_flags),
            Err(HeaderFlagError::QueryWithRABitSet)
        );
    }

    #[test]
    fn error_query_with_aa() {
        let header_flags: u16 = 0b0000_0100_0000_0000;

        assert_eq!(
            HeaderFlags::try_from(header_flags),
            Err(HeaderFlagError::QueryWithAABitSet)
        );
    }

    #[test]
    fn error_query_with_tc() {
        let header_flags: u16 = 0b0000_0010_0000_0000;

        assert_eq!(
            HeaderFlags::try_from(header_flags),
            Err(HeaderFlagError::QueryWithTCBitSet)
        );
    }

    #[test]
    fn mutate_tc_bit() {
        let mut header_flag = generate_response_header_flag(false, false, true, true, Rcode::NoError);

        header_flag.truncation(true);

        assert!(header_flag.truncation);
    }
}
