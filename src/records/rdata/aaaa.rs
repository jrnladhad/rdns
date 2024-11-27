use std::net::Ipv6Addr;
use crate::packet::bin_reader::BinReader;
use crate::records::record_data::RecordDataError;

#[derive(Debug, PartialEq)]
pub struct AAAA {
    address: Ipv6Addr,
}

impl AAAA {
    pub fn new(address: Ipv6Addr) -> Self {
        Self{
            address
        }
    }

    pub fn read_record_data(decoder: &mut BinReader) -> Result<Self, RecordDataError> {
        let data = decoder.read_u128().map_err(|_| RecordDataError::UnableToReadIpv6Address)?;
        let address = Ipv6Addr::from(data);

        Ok(AAAA { address })
    }
}