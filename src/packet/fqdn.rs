use crate::packet::bin_reader::BinReader;
use thiserror::Error;

const PTR_MASK: u8 = 11 << 6;
const OFFSET_MASK: u16 = 0x3FFF;
const MAX_JUMP_REDIRECTION: u16 = 2;

#[derive(Error, Debug)]
pub enum FqdnError {
    #[error("No label length provided")]
    MissingLabelLength,
    #[error("No pointer offset provided")]
    MissingLabelPointerOffset,
    #[error("Label length too long")]
    LabelLengthTooLong,
    #[error("Label length {0} is larger than the label")]
    NotEnoughLabelData(u8),
    #[error("Label is not encoded correctly, cannot read label data")]
    IncorrectLabelEncoding,
    #[error("Byte does not represent the correct length or pointer")]
    MalformedLenOrPtrInfo,
    #[error("FQDN in the packet is exceeding the max limit")]
    FqdnTooLong,
    #[error("Offset provided in the packet does not point to existing FQDN in packet")]
    IncorrectPointerOffset,
    #[error("Too many redirections while reading name")]
    TooManyRedirections,
}

pub type FqdnResult<T> = Result<T, FqdnError>;

pub struct FQDN {
    labels: Vec<String>,
}

enum FqdnParsingFSM {
    Start,
    Length,
    Pointer,
    End,
}

impl FQDN {
    fn get_label(decoder: &mut BinReader) -> FqdnResult<String> {
        let label_len = decoder
            .read_u8()
            .map_err(|_| FqdnError::MissingLabelLength)?;

        if label_len > 63 {
            return Err(FqdnError::LabelLengthTooLong);
        }

        let label = decoder
            .read_n_bytes(label_len as u16)
            .map_err(|_| FqdnError::NotEnoughLabelData(label_len))?;

        let label = String::from_utf8(Vec::from(label))
            .map_err(|_| FqdnError::IncorrectLabelEncoding)?
            .to_lowercase();

        Ok(label)
    }

    fn get_parsing_state(decoder: &BinReader) -> FqdnResult<FqdnParsingFSM> {
        let ptr_or_len = Some(decoder.peek().map_err(|_| FqdnError::MissingLabelLength)?);

        match ptr_or_len {
            Some(0) => Ok(FqdnParsingFSM::End),
            Some(data) if data & PTR_MASK == PTR_MASK => Ok(FqdnParsingFSM::Pointer),
            Some(data) if data & PTR_MASK != PTR_MASK => Ok(FqdnParsingFSM::Length),
            Some(_) | None => Err(FqdnError::MalformedLenOrPtrInfo),
        }
    }

    fn recursively_create_name(
        decoder: &mut BinReader,
        jump_count: u16,
    ) -> FqdnResult<Vec<String>> {
        if jump_count > MAX_JUMP_REDIRECTION {
            return Err(FqdnError::TooManyRedirections);
        }

        let mut parsing_fsm = FqdnParsingFSM::Start;
        let mut labels: Vec<String> = Vec::new();

        loop {
            parsing_fsm = match parsing_fsm {
                FqdnParsingFSM::Start => Self::get_parsing_state(decoder)?,

                FqdnParsingFSM::Length => {
                    let label = Self::get_label(decoder)?;
                    labels.push(label);

                    if labels.len() > 255 {
                        return Err(FqdnError::FqdnTooLong);
                    }

                    FqdnParsingFSM::Start
                }

                FqdnParsingFSM::Pointer => {
                    let label_ptr = decoder
                        .read_u16()
                        .map_err(|_| FqdnError::MissingLabelPointerOffset)?;

                    let offset = label_ptr & OFFSET_MASK;
                    let mut cloned_decoder = decoder.cheap_clone(offset);
                    let mut parsed_labels =
                        Self::recursively_create_name(&mut cloned_decoder, jump_count + 1)?;

                    labels.append(&mut parsed_labels);

                    FqdnParsingFSM::End
                }

                FqdnParsingFSM::End => break,
            }
        }

        Ok(labels)
    }

    pub fn from_bytes(decoder: &mut BinReader) -> FqdnResult<FQDN> {
        let labels = Self::recursively_create_name(decoder, 0)?;

        Ok(FQDN { labels })
    }

    pub fn to_str(&self) -> String {
        let fqdn = self.labels.iter().fold(String::new(), |acc, label| {
            if acc.is_empty() {
                acc + label
            } else {
                acc + "." + label
            }
        });

        fqdn
    }
}

#[cfg(test)]
mod test {
    use crate::packet::bin_reader::BinReader;
    use crate::packet::fqdn::FQDN;

    #[test]
    fn read_name_all_lower() {
        let packet_bytes: [u8; 20] = [
            0x03, 0x77, 0x77, 0x77, 0x06, 0x67, 0x6f, 0x6f, 0x67, 0x6c, 0x65, 0x03, 0x63, 0x6f,
            0x6d, 0x00, 0x00, 0x01, 0x00, 0x01,
        ];
        let expected = String::from("www.google.com");

        let mut decoder = BinReader::new(&packet_bytes);
        let fqdn = FQDN::from_bytes(&mut decoder).unwrap();

        assert_eq!(fqdn.to_str(), expected);
    }

    #[test]
    fn read_ptr_label() {
        let packet_bytes: [u8; 35] = [
            0xf2, 0xe8, 0x81, 0x80, 0x00, 0x01, 0x00, 0x01, 0x00, 0x00, 0x00, 0x00, 0x03, 0x77,
            0x77, 0x77, 0x06, 0x67, 0x6f, 0x6f, 0x67, 0x6c, 0x65, 0x03, 0x63, 0x6f, 0x6d, 0x00,
            0x00, 0x01, 0x00, 0x01, 0xc0, 0x0c, 0x00,
        ];

        let expected = String::from("www.google.com");

        let decoder = BinReader::new(&packet_bytes);
        let mut decoder = decoder.cheap_clone(32);

        let fqdn = FQDN::from_bytes(&mut decoder).unwrap();

        assert_eq!(fqdn.to_str(), expected);
    }

    #[test]
    fn read_ptr_from_between() {
        let packet_bytes: [u8; 24] = [
            0x01, 0x61, 0x0c, 0x72, 0x6F, 0x6F, 0x74, 0x2d, 0x73, 0x65, 0x72, 0x76, 0x65, 0x72,
            0x73, 0x03, 0x6e, 0x65, 0x74, 0x00, 0x01, 0x62, 0xc0, 0x02,
        ];

        let expected = String::from("b.root-servers.net");

        let decoder = BinReader::new(&packet_bytes);
        let mut decoder = decoder.cheap_clone(20);

        let fqdn = FQDN::from_bytes(&mut decoder).unwrap();

        assert_eq!(fqdn.to_str(), expected);
    }
}
