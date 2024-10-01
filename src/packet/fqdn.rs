use thiserror::Error;
use crate::packet::bin_reader::BinReader;

const PTR_MASK:u8 = 11 << 6;

#[derive(Error, Debug)]
pub enum FqdnError {
    #[error("No label length provided")]
    MissingLabelLength,
    #[error("Label length too long")]
    LabelLengthTooLong,
    #[error("Label length {0} is larger than the label")]
    NotEnoughLabelData(u8),
    #[error("Label is not encoded correctly, cannot read label data")]
    IncorrectLabelEncoding,
    #[error("Byte does not represent the correct length or pointer")]
    MalformedLenOrPtrInfo,
    #[error("FQDN in the packet is exceeding the max limit")]
    FqdnTooLong
}

pub type FqdnResult<T> = Result<T, FqdnError>;

pub struct FQDN {
    labels: Vec<String>
}

enum FqdnParsingFSM {
    PtrOrLen,
    Length,
    // LabelPointer,
    End
}

impl FQDN {
    pub fn from_bytes(decoder: &mut BinReader) -> FqdnResult<FQDN> {
        let mut parsing_fsm = FqdnParsingFSM::PtrOrLen;
        let mut labels:Vec<String> = Vec::new();

        loop {
            parsing_fsm = match parsing_fsm {
                FqdnParsingFSM::PtrOrLen => {
                    let ptr_or_len = Some(
                        decoder
                        .peek()
                        .map_err(|_| FqdnError::MissingLabelLength)?);

                    match ptr_or_len {
                        Some(0) => FqdnParsingFSM::End,
                        Some(data) if data & PTR_MASK == PTR_MASK => FqdnParsingFSM::Length, // Change this to Pointer
                        Some(data) if data & PTR_MASK != PTR_MASK => FqdnParsingFSM::Length,
                        Some(_) | None => return Err(FqdnError::MalformedLenOrPtrInfo)
                    }
                },

                FqdnParsingFSM::Length => {
                    let label_len = decoder
                        .read_u8()
                        .map_err(|_| FqdnError::MissingLabelLength)?;

                    if label_len > 63 {
                        return Err(FqdnError::LabelLengthTooLong);
                    }

                    let label = decoder.read_n_bytes(label_len as usize)
                        .map_err(|_| FqdnError::NotEnoughLabelData(label_len))?;

                    let label = String::from_utf8(Vec::from(label))
                        .map_err(|_| FqdnError::IncorrectLabelEncoding)?
                        .to_lowercase();

                    labels.push(label);

                    if labels.len() > 255 {
                        return Err(FqdnError::FqdnTooLong)
                    }

                    FqdnParsingFSM::PtrOrLen
                },

                FqdnParsingFSM::End => {
                    break
                }
            }
        }

        Ok(FQDN {
            labels
        })
    }

    pub fn to_str(&self) -> String {
        let fqdn = self.labels
            .iter()
            .fold(String::new(), |acc, label| {
                if acc.is_empty() { acc + label } else { acc + "." + label }
            });
        fqdn
    }
}

#[cfg(test)]
mod test
{
    use crate::packet::bin_reader::BinReader;
    use crate::packet::fqdn::FQDN;

    #[test]
    fn read_name_all_lower()
    {
        let packet_bytes: [u8; 20] = [
            0x03, 0x77, 0x77, 0x77,
            0x06, 0x67, 0x6f, 0x6f,
            0x67, 0x6c, 0x65, 0x03,
            0x63, 0x6f, 0x6d, 0x00,
            0x00, 0x01, 0x00, 0x01
        ];
        let expected = String::from("www.google.com");

        let mut decoder = BinReader::new(&packet_bytes);
        let fqdn = FQDN::from_bytes(&mut decoder).unwrap();

        assert_eq!(fqdn.to_str(), expected);
    }
}