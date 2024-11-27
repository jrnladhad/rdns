use crate::packet::{bin_reader::BinReader, fqdn::Fqdn};
use crate::records::{record_class::RecordClass, record_data::RecordData, record_type::RecordType};
use thiserror::Error;

#[derive(Error, Debug)]
pub enum RecordError {
    #[error("Invalid record name")]
    InvalidName,
    #[error("Invalid record name")]
    InvalidType,
    #[error("Invalid record name")]
    InvalidClass,
    #[error("Invalid record name")]
    InvalidTtl,
    #[error("Invalid record name")]
    InvalidDataLength,
    #[error("Invalid record name")]
    InvalidData,
}

type RecordResult = Result<Record, RecordError>;
type RecordBuilderUnset = RecordBuilder<RecordOwnerUnset, RecordTypeUnset, RecordDataUnset>;
type RecordBuilderOwnerSet = RecordBuilder<RecordOwnerSet, RecordTypeUnset, RecordDataUnset>;
type RecordBuilderTypeSet = RecordBuilder<RecordOwnerSet, RecordTypeSet, RecordDataUnset>;
type RecordBuilderSet = RecordBuilder<RecordOwnerSet, RecordTypeSet, RecordDataSet>;

pub struct RecordDataUnset;

pub struct RecordDataSet(RecordData);

trait RecordDataState {}
impl RecordDataState for RecordDataUnset {}
impl RecordDataState for RecordDataSet {}

pub struct RecordOwnerUnset;
pub struct RecordOwnerSet(Fqdn);

trait RecordOwnerState {}
impl RecordOwnerState for RecordOwnerUnset {}
impl RecordOwnerState for RecordOwnerSet {}

pub struct RecordTypeUnset;
pub struct RecordTypeSet(RecordType);

trait RecordTypeState {}
impl RecordTypeState for RecordTypeUnset {}
impl RecordTypeState for RecordTypeSet {}

#[derive(Debug, PartialEq)]
pub struct Record {
    owner_name: Fqdn,
    record_type: RecordType,
    class: RecordClass,
    ttl: u32,
    data: RecordData,
}

pub struct RecordBuilder<O, T, D>
where
    O: RecordOwnerState,
    T: RecordTypeState,
    D: RecordDataState,
{
    owner_name: O,
    record_type: T,
    class: RecordClass,
    ttl: u32,
    data: D,
}

impl Default for RecordBuilderUnset {
    fn default() -> Self {
        RecordBuilder {
            owner_name: RecordOwnerUnset,
            record_type: RecordTypeUnset,
            class: RecordClass::IN,
            ttl: 0,
            data: RecordDataUnset,
        }
    }
}

impl Record {
    pub fn from_bytes(decoder: &mut BinReader) -> RecordResult {
        let owner_name = Fqdn::from_bytes(decoder).map_err(|_| RecordError::InvalidName)?;

        let record_type = decoder.read_u16().map_err(|_| RecordError::InvalidType)?;
        let record_type = RecordType::from(record_type);

        let class = decoder.read_u16().map_err(|_| RecordError::InvalidClass)?;
        let class = RecordClass::from(class);

        let ttl = decoder.read_u32().map_err(|_| RecordError::InvalidTtl)?;

        let _data_length = decoder
            .read_u16()
            .map_err(|_| RecordError::InvalidDataLength)?;

        // some checks on data length based on record type.

        let data = RecordData::generate_record_data(decoder, &record_type)
            .map_err(|_| RecordError::InvalidData)?;

        let record = RecordBuilder::new()
            .owner_name(owner_name)
            .record_type(record_type)
            .data(data)
            .class(class)
            .ttl(ttl)
            .build();

        Ok(record)
    }
}

impl RecordBuilderUnset {
    pub fn new() -> Self {
        RecordBuilder::default()
    }

    pub fn owner_name(self, name: Fqdn) -> RecordBuilderOwnerSet {
        RecordBuilder {
            owner_name: RecordOwnerSet(name),
            record_type: self.record_type,
            class: self.class,
            ttl: self.ttl,
            data: self.data,
        }
    }
}

impl RecordBuilderOwnerSet {
    pub fn record_type(self, r_type: RecordType) -> RecordBuilderTypeSet {
        RecordBuilder {
            owner_name: self.owner_name,
            record_type: RecordTypeSet(r_type),
            class: self.class,
            ttl: self.ttl,
            data: self.data,
        }
    }
}

impl RecordBuilderTypeSet {
    pub fn data(self, rdata: RecordData) -> RecordBuilderSet {
        RecordBuilder {
            owner_name: self.owner_name,
            record_type: self.record_type,
            class: self.class,
            ttl: self.ttl,
            data: RecordDataSet(rdata),
        }
    }
}

impl RecordBuilderSet {
    pub fn build(self) -> Record {
        Record {
            owner_name: self.owner_name.0,
            record_type: self.record_type.0,
            class: self.class,
            ttl: self.ttl,
            data: self.data.0,
        }
    }
}

impl<O, T, D> RecordBuilder<O, T, D>
where
    O: RecordOwnerState,
    T: RecordTypeState,
    D: RecordDataState,
{
    pub fn class(self, class: RecordClass) -> Self {
        Self {
            owner_name: self.owner_name,
            record_type: self.record_type,
            class,
            ttl: self.ttl,
            data: self.data,
        }
    }

    pub fn ttl(self, ttl: u32) -> Self {
        Self {
            owner_name: self.owner_name,
            record_type: self.record_type,
            class: self.class,
            ttl,
            data: self.data,
        }
    }
}

#[cfg(test)]
mod test {
    use crate::packet::bin_reader::BinReader;
    use crate::packet::fqdn::FqdnBuilder;
    use crate::packet::record::{Record, RecordBuilder};
    use crate::records::rdata::a::A;
    use crate::records::rdata::aaaa::AAAA;
    use crate::records::record_class::RecordClass;
    use crate::records::record_data::RecordData;
    use crate::records::record_type::RecordType;
    use std::net::{Ipv4Addr, Ipv6Addr};
    use std::str::FromStr;

    fn get_sample_a_record() -> Record {
        let expected_name = FqdnBuilder::new()
            .generate_from_string(String::from("www.google.com"))
            .build();

        let data = A::new(Ipv4Addr::from_str("172.217.14.196").unwrap());
        let record_data = RecordData::A(data);

        let expected_record = RecordBuilder::new()
            .owner_name(expected_name)
            .record_type(RecordType::A)
            .class(RecordClass::IN)
            .ttl(104)
            .data(record_data)
            .build();

        expected_record
    }

    fn get_sample_aaaa_record() -> Record {
        let expected_name = FqdnBuilder::new()
            .generate_from_string(String::from("www.google.com"))
            .build();

        let data = AAAA::new(Ipv6Addr::from_str("2607:f8b0:400a:80a::2004").unwrap());
        let record_data = RecordData::AAAA(data);

        let expected_record = RecordBuilder::new()
            .owner_name(expected_name)
            .record_type(RecordType::AAAA)
            .class(RecordClass::IN)
            .ttl(201)
            .data(record_data)
            .build();

        expected_record
    }

    #[test]
    fn read_a_record_successfully() {
        let packet_bytes: [u8; 36] = [
            0x03, 0x77, 0x77, 0x77, 0x06, 0x67, 0x6f, 0x6f, 0x67, 0x6c, 0x65, 0x03, 0x63, 0x6f,
            0x6d, 0x00, 0x00, 0x01, 0x00, 0x01, 0xc0, 0x00, 0x00, 0x01, 0x00, 0x01, 0x00, 0x00,
            0x00, 0x68, 0x00, 0x04, 0xac, 0xd9, 0x0e, 0xc4,
        ];

        let expected_record = get_sample_a_record();

        let decoder = BinReader::new(&packet_bytes);
        let mut decoder = decoder.cheap_clone(20);

        let actual_record = Record::from_bytes(&mut decoder).unwrap();

        assert_eq!(actual_record, expected_record);
    }

    #[test]
    fn read_aaaa_record_successfully() {
        let packet_bytes: [u8; 48] = [
            0x03, 0x77, 0x77, 0x77, 0x06, 0x67, 0x6f, 0x6f, 0x67, 0x6c, 0x65, 0x03, 0x63, 0x6f,
            0x6d, 0x00, 0x00, 0x01, 0x00, 0x01, 0xc0, 0x00, 0x00, 0x1c, 0x00, 0x01, 0x00, 0x00,
            0x00, 0xc9, 0x00, 0x10, 0x26, 0x07, 0xf8, 0xb0, 0x40, 0x0a, 0x08, 0x0a, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x20, 0x04,
        ];

        let expected_record = get_sample_aaaa_record();

        let decoder = BinReader::new(&packet_bytes);
        let mut decoder = decoder.cheap_clone(20);

        let actual_record = Record::from_bytes(&mut decoder).unwrap();

        assert_eq!(actual_record, expected_record);
    }

    #[test]
    fn read_multi_records_successfully() {
        let packet_bytes: [u8; 64] = [
            0x03, 0x77, 0x77, 0x77, 0x06, 0x67, 0x6f, 0x6f, 0x67, 0x6c, 0x65, 0x03, 0x63, 0x6f,
            0x6d, 0x00, 0x00, 0x01, 0x00, 0x01, 0xc0, 0x00, 0x00, 0x01, 0x00, 0x01, 0x00, 0x00,
            0x00, 0x68, 0x00, 0x04, 0xac, 0xd9, 0x0e, 0xc4, 0xc0, 0x00, 0x00, 0x1c, 0x00, 0x01,
            0x00, 0x00, 0x00, 0xc9, 0x00, 0x10, 0x26, 0x07, 0xf8, 0xb0, 0x40, 0x0a, 0x08, 0x0a,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x20, 0x04,
        ];

        let expected_records: Vec<Record> = vec![get_sample_a_record(), get_sample_aaaa_record()];
        let mut actual_records: Vec<Record> = Vec::with_capacity(2);

        let decoder = BinReader::new(&packet_bytes);
        let mut decoder = decoder.cheap_clone(20);

        for _ in 0..2 {
            let actual_record = Record::from_bytes(&mut decoder).unwrap();
            actual_records.push(actual_record);
        }

        assert_eq!(actual_records, expected_records);
    }
}
