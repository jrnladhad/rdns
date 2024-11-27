#[derive(Debug, PartialEq, Clone, Copy)]
pub enum RecordType {
    A,
    AAAA,
    // NS,
    // CNAME,
    // SOA,
    // WKS,
    // PTR,
    // INFO,
    // MINFO,
    // MX,
    // TXT,
    // AXFR,
    // ALL,
    UNKNOWN,
}

impl From<u16> for RecordType {
    fn from(value: u16) -> Self {
        match value {
            1 => RecordType::A,
            28 => RecordType::AAAA,
            // 2 => RecordType::NS,
            // 5 => RecordType::CNAME,
            // 6 => RecordType::SOA,
            // 255 => RecordType::ALL,
            _ => RecordType::UNKNOWN,
        }
    }
}