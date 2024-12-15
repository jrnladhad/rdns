use crate::packet::seder::deserializer::Deserialize;
use crate::packet::seder::serializer::Serialize;
use crate::records::rdata::a::A;
use crate::records::rdata::aaaa::AAAA;
use crate::records::record_type::RecordType;
use std::fmt::Debug;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum RecordDataError {
    #[error("Unable to read IPv4 address from response")]
    UnableToReadIpv4Address,
    #[error("Unable to read IPv6 address from response")]
    UnableToReadIpv6Address,
}

#[derive(Debug, PartialEq)]
pub enum RecordData {
    A(A),
    AAAA(AAAA),
    UNKNOWN,
}

impl RecordData {
    pub fn from_bytes(
        decoder: &mut Deserialize,
        record_type: &RecordType,
    ) -> Result<Self, RecordDataError> {
        match record_type {
            RecordType::A => {
                let data = A::from_bytes(decoder)?;
                Ok(RecordData::A(data))
            }
            RecordType::AAAA => {
                let data = AAAA::from_bytes(decoder)?;
                Ok(RecordData::AAAA(data))
            }
            _ => Err(RecordDataError::UnableToReadIpv4Address),
        }
    }

    pub fn to_bytes(&self, encoder: &mut Serialize) {
        match self {
            RecordData::A(a_rdata) => a_rdata.to_bytes(encoder),
            RecordData::AAAA(aaaa_rdata) => aaaa_rdata.to_bytes(encoder),
            _ => {}
        }
    }
}
