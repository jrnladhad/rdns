use crate::packet::bin_reader::BinReader;
use std::marker::PhantomData;
use thiserror::Error;

const PTR_MASK: u8 = 11 << 6;
const OFFSET_MASK: u16 = 0x3FFF;
const MAX_REDIRECTIONS: u16 = 3;
const MAX_FQDN_LENGTH: usize = 255;

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
    #[error("Unable to read data from the buffer")]
    InsufficientData,
}

#[derive(Debug, Clone)]
pub struct FqdnUnset;
#[derive(Debug, Clone)]
pub struct FqdnSet;

trait FqdnState {}
impl FqdnState for FqdnUnset {}
impl FqdnState for FqdnSet {}

pub type FqdnResult<T> = Result<T, FqdnError>;

enum FqdnParsingFSM {
    Start,
    Length,
    Pointer,
    End,
}

#[derive(Debug, PartialEq, Clone)]
pub struct Fqdn {
    // TODO: This should be changed to a fixed array of size 64 and each label should be 256 char *
    // this would allow us to put it on the stack, rather than on heap.
    labels: Vec<String>
}

pub struct FqdnBuilder<S>
where
    S: FqdnState,
{
    labels: Vec<String>,
    state: PhantomData<S>,
}

impl Fqdn {
    pub fn from_bytes(decoder: &mut BinReader) -> FqdnResult<Fqdn> {
        let fqdn = FqdnBuilder::new().generate_from_bytes(decoder)?.build();

        Ok(fqdn)
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

impl FqdnBuilder<FqdnUnset> {
    pub fn new() -> Self {
        FqdnBuilder {
            labels: Vec::with_capacity(MAX_FQDN_LENGTH),
            state: PhantomData,
        }
    }

    pub fn generate_from_bytes(
        mut self,
        decoder: &mut BinReader,
    ) -> FqdnResult<FqdnBuilder<FqdnSet>> {
        let _ = self.generate_labels_recursively(decoder, 0)?;

        Ok(FqdnBuilder {
            labels: self.labels,
            state: PhantomData,
        })
    }

    pub fn generate_from_string(self, qname: String) -> FqdnBuilder<FqdnSet> {
        let labels: Vec<&str> = qname.split('.').collect();
        let mut final_labels: Vec<String> = Vec::new();

        for label in labels {
            final_labels.push(label.to_owned());
        }

        FqdnBuilder {
            labels: final_labels,
            state: PhantomData,
        }
    }

    fn generate_labels_recursively(
        &mut self,
        decoder: &mut BinReader,
        jump_count: u16,
    ) -> FqdnResult<()> {
        if jump_count > MAX_REDIRECTIONS {
            return Err(FqdnError::TooManyRedirections);
        }

        let mut is_indirection = false;
        let mut parsing_fsm = FqdnParsingFSM::Start;

        loop {
            parsing_fsm = match parsing_fsm {
                FqdnParsingFSM::Start => Self::get_parsing_state(decoder)?,

                FqdnParsingFSM::Length => {
                    if self.labels.len() >= MAX_FQDN_LENGTH {
                        return Err(FqdnError::FqdnTooLong);
                    }

                    let label = Self::get_label(decoder)?;
                    self.labels.push(label);

                    FqdnParsingFSM::Start
                }

                FqdnParsingFSM::Pointer => {
                    let label_ptr = decoder
                        .read_u16()
                        .map_err(|_| FqdnError::MissingLabelPointerOffset)?;

                    let offset = label_ptr & OFFSET_MASK;
                    let mut cloned_decoder = decoder.cheap_clone(offset);
                    let _ =
                        self.generate_labels_recursively(&mut cloned_decoder, jump_count + 1)?;
                    is_indirection = true;

                    FqdnParsingFSM::End
                }

                FqdnParsingFSM::End => {
                    if is_indirection == false
                    {
                        let _ = decoder.read_u8().map_err(|_| FqdnError::InsufficientData);
                    }

                    break;
                }
            }
        }

        Ok(())
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
}

impl FqdnBuilder<FqdnSet> {
    pub fn build(self) -> Fqdn {
        Fqdn {
            labels: self.labels,
        }
    }
}

#[cfg(test)]
mod test {
    use crate::packet::bin_reader::BinReader;
    use crate::packet::fqdn::Fqdn;

    #[test]
    fn read_name_all_lower() {
        let packet_bytes: [u8; 20] = [
            0x03, 0x77, 0x77, 0x77, 0x06, 0x67, 0x6f, 0x6f, 0x67, 0x6c, 0x65, 0x03, 0x63, 0x6f,
            0x6d, 0x00, 0x00, 0x01, 0x00, 0x01,
        ];
        let expected = String::from("www.google.com");

        let mut decoder = BinReader::new(&packet_bytes);
        let fqdn = Fqdn::from_bytes(&mut decoder).unwrap();

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

        let fqdn = Fqdn::from_bytes(&mut decoder).unwrap();

        assert_eq!(fqdn.to_str(), expected);
    }

    #[test]
    fn read_ptr_from_between() {
        let packet_bytes: [u8; 24] = [
            0x01, 0x61, 0x0c, 0x72, 0x6F, 0x6F, 0x74, 0x2d, 0x73, 0x65, 0x72, 0x76, 0x65, 0x72,
            0x73, 0x03, 0x6e, 0x65, 0x74, 0x00, // a.root-servers.net.
            0x01, 0x62, 0xc0, 0x02, // b.root-servers.net.
        ];

        let expected = String::from("b.root-servers.net");

        let decoder = BinReader::new(&packet_bytes);
        let mut decoder = decoder.cheap_clone(20);

        let fqdn = Fqdn::from_bytes(&mut decoder).unwrap();

        assert_eq!(fqdn.to_str(), expected);
    }

    #[test]
    fn read_ptr_multi_jump() {
        let packet_bytes: [u8; 19] = [
            0x01, 0x61, 0x03, 0x66, 0x6F, 0x6F, 0x03, 0x63, 0x6F, 0x6D, 0x00, // a.foo.com.
            0x01, 0x62, 0xc0, 0x02, // b.foo.com.
            0x01, 0x64, 0xc0, 0x0b, // d.b.foo.com.
        ];

        let expected = String::from("d.b.foo.com");

        let decoder = BinReader::new(&packet_bytes);
        let mut decoder = decoder.cheap_clone(15);

        let fqdn = Fqdn::from_bytes(&mut decoder).unwrap();

        assert_eq!(fqdn.to_str(), expected);
    }
}
