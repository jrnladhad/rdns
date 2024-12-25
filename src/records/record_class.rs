use thiserror::Error;

#[derive(Error, Debug)]
pub enum RecordClassError
{
    #[error("Record class {0} in message is invalid according to RFC or is not supported")]
    UnknownRecordClass(u16)
}

#[derive(Debug, PartialEq, Copy, Clone)]
pub enum RecordClass {
    IN,
    CS,
    CH,
    HS,
}

impl TryFrom<u16> for RecordClass
{
    type Error = RecordClassError;

    fn try_from(value: u16) -> Result<Self, Self::Error> {
        match value {
            1 => Ok(RecordClass::IN),
            2 => Ok(RecordClass::CS),
            3 => Ok(RecordClass::CH),
            4 => Ok(RecordClass::HS),
            _ => Err(RecordClassError::UnknownRecordClass(value))
        }
    }
}

impl From<RecordClass> for u16 {
    fn from(val: RecordClass) -> Self {
        match val {
            RecordClass::IN => 1,
            RecordClass::CS => 2,
            RecordClass::CH => 3,
            RecordClass::HS => 4,
        }
    }
}