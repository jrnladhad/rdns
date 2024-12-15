use crate::packet::seder::deserializer::Deserialize;
use crate::packet::seder::serializer::Serialize;
use crate::records::record_data::RecordDataError;
use std::net::Ipv4Addr;

#[derive(Debug, PartialEq)]
pub struct A {
    address: Ipv4Addr,
}

impl A {
    const RECORD_DATA_LENGTH: u16 = 4;
    pub fn new(address: Ipv4Addr) -> Self {
        Self { address }
    }

    pub fn from_bytes(decoder: &mut Deserialize) -> Result<Self, RecordDataError> {
        let data = decoder
            .read_u32()
            .map_err(|_| RecordDataError::UnableToReadIpv4Address)?;
        let address = Ipv4Addr::from(data);

        Ok(A { address })
    }

    pub fn to_bytes(&self, encoder: &mut Serialize) {
        encoder.write_u16(A::RECORD_DATA_LENGTH);
        encoder.write_n_bytes(self.address.octets().to_vec());
    }
}