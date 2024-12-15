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

impl Into<u16> for RecordClass {
    fn into(self) -> u16 {
        match self {
            RecordClass::IN => 1,
            RecordClass::CH => 3,
            RecordClass::HS => 4,
            RecordClass::ANY => 255,
            RecordClass::UNKNOWN => 256
        }
    }
}