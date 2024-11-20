#[derive(Debug, PartialEq, Copy, Clone)]
pub enum RecordClass {
    IN,
    CH,
    HS,
    ANY,
    UNKNOWN,
}

impl From<u16> for RecordClass {
    fn from(value: u16) -> Self {
        match value {
            1 => RecordClass::IN,
            3 => RecordClass::CH,
            4 => RecordClass::HS,
            255 => RecordClass::ANY,
            _ => RecordClass::UNKNOWN,
        }
    }
}