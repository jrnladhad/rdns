use thiserror::Error;
use crate::records::record_type::RecordTypeError::UnknownRecordType;

#[derive(Error, Debug)]
pub enum RecordTypeError {
    #[error("Message contains unknown record type. Record type provided {0}")]
    UnknownRecordType(u16)
}

#[derive(Debug, PartialEq, Clone, Copy)]
pub enum RecordType {
    A,
    AAAA,
    // NS,
    // CNAME,
    // SOA,
    // WKS,
    // PTR,
    // INFO,
    // MINFO,
    // MX,
    // TXT,
    // AXFR,
    // ALL,
}

impl TryFrom<u16> for RecordType {
    type Error = RecordTypeError;

    fn try_from(value: u16) -> Result<Self, Self::Error> {
        match value {
            1 => Ok(RecordType::A),
            28 => Ok(RecordType::AAAA),
            _ => Err(UnknownRecordType(value))
        }
    }
}

impl From<RecordType> for u16 {
    fn from(val: RecordType) -> Self {
        match val {
            RecordType::A => 1,
            RecordType::AAAA => 28,
        }
    }
}