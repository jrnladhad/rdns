use crate::packet::seder::deserializer::Deserialize;
use crate::packet::seder::serializer::Serialize;
use crate::records::record_data::RecordDataError;
use std::net::Ipv4Addr;
use crate::packet::seder::{TryFrom, ToBytes};

type ARecordResult = Result<A, RecordDataError>;

#[derive(Debug, PartialEq)]
pub struct A {
    address: Ipv4Addr,
}

impl TryFrom for A {
    type Error = RecordDataError;

    fn try_from_bytes(decoder: &mut Deserialize) -> ARecordResult {
        let data = decoder
            .read_u32()
            .map_err(|_| RecordDataError::UnableToReadIpv4Address)?;
        let address = Ipv4Addr::from(data);

        Ok(A { address })
    }
}

impl ToBytes for A {
    fn to_bytes(&self, encoder: &mut Serialize) {
        encoder.write_u16(A::RECORD_DATA_LENGTH);
        encoder.write_n_bytes(self.address.octets().to_vec());
    }
}

impl A {
    const RECORD_DATA_LENGTH: u16 = 4;
    pub fn new(address: Ipv4Addr) -> Self {
        Self { address }
    }
}