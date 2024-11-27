use std::net::Ipv4Addr;
use crate::packet::bin_reader::BinReader;
use crate::records::record_data::RecordDataError;

#[derive(Debug, PartialEq)]
pub struct A {
    address: Ipv4Addr,
}

impl A {
    pub fn new(address: Ipv4Addr) -> Self {
        Self { address }
    }

    pub fn read_record_data(decoder: &mut BinReader) -> Result<Self, RecordDataError> {
        let data = decoder
            .read_u32()
            .map_err(|_| RecordDataError::UnableToReadIpv4Address)?;
        let address = Ipv4Addr::from(data);

        Ok(A { address })
    }
}