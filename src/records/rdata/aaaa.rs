use crate::packet::seder::deserializer::Deserialize;
use crate::packet::seder::serializer::Serialize;
use crate::records::record_data::RecordDataError;
use std::net::Ipv6Addr;

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

    pub fn from_bytes(decoder: &mut Deserialize) -> Result<Self, RecordDataError> {
        let data = decoder.read_u128().map_err(|_| RecordDataError::UnableToReadIpv6Address)?;
        let address = Ipv6Addr::from(data);

        Ok(AAAA { address })
    }

    pub fn to_bytes(&self, encoder: &mut Serialize) {
        let aaaa_record = self.address.octets();
        encoder.write_u16(aaaa_record.len() as u16);
        encoder.write_n_bytes(aaaa_record.to_vec());
    }
}