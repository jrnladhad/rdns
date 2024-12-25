use crate::packet::seder::deserializer::Deserialize;
use crate::packet::seder::serializer::Serialize;
use crate::packet::seder::{TryFromBytes, ToBytes};
use crate::records::record_data::RecordDataError;
use std::net::Ipv6Addr;

type AAAARecordResult = Result<AAAA, RecordDataError>;

#[derive(Debug, PartialEq)]
pub struct AAAA {
    address: Ipv6Addr,
}

impl TryFromBytes for AAAA {
    type Error = RecordDataError;

    fn try_from_bytes(decoder: &mut Deserialize) -> AAAARecordResult {
        let data = decoder
            .read_u128()
            .map_err(|_| RecordDataError::UnableToReadIpv6Address)?;
        let address = Ipv6Addr::from(data);

        Ok(AAAA { address })
    }
}

impl ToBytes for AAAA {
    fn to_bytes(&self, encoder: &mut Serialize) {
        let aaaa_record = self.address.octets();
        encoder.write_u16(aaaa_record.len() as u16);
        encoder.write_n_bytes(aaaa_record.to_vec());
    }
}

impl AAAA {
    pub fn new(address: Ipv6Addr) -> Self {
        Self { address }
    }
}
