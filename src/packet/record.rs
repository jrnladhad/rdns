use crate::packet::fqdn::Fqdn;
use crate::packet::record_type::RecordType;
use crate::packet::record_class::RecordClass;

struct Record {
    owner_name: Fqdn,
    record_type: RecordType,
    class: RecordClass,
    ttl: u32,
    data_length: u16,
}