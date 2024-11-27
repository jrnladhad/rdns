use crate::packet::bin_reader::BinReader;
use crate::records::record_type::RecordType;
use std::fmt::Debug;
use thiserror::Error;
use crate::records::rdata::a::A;
use crate::records::rdata::aaaa::AAAA;

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
    pub fn generate_record_data(
        decoder: &mut BinReader,
        record_type: &RecordType,
    ) -> Result<Self, RecordDataError> {
        match record_type {
            RecordType::A => {
                let data = A::read_record_data(decoder)?;
                Ok(RecordData::A(data))
            },
            RecordType::AAAA => {
                let data = AAAA::read_record_data(decoder)?;
                Ok(RecordData::AAAA(data))
            }
            _ => Err(RecordDataError::UnableToReadIpv4Address),
        }
    }
}


