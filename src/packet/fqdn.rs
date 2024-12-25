use thiserror::Error;
use std::marker::PhantomData;
use crate::packet::seder::{deserializer::Deserialize, serializer::Serialize, TryFrom, ToBytes};

const PTR_MASK: u8 = 11 << 6;
const LEN_MASK: u8 = 0;
const OFFSET_MASK: u16 = 0x3FFF;
const POINTER_MARKER: u16 = 0xC0_00;
const MAX_REDIRECTIONS: u8 = 3;
const MAX_FQDN_LENGTH: u8 = 255;
const MAX_NUMBER_OF_LABELS: u8 = 127;

#[derive(Error, Debug)]
#[derive(PartialEq)]
pub enum FqdnError {
    #[error("No label length provided")]
    MissingLabelLength,
    #[error("No pointer offset provided")]
    MissingLabelPointerOffset,
    #[error("Label length {0} is larger than the label")]
    NotEnoughLabelData(u8),
    #[error("Label is not encoded correctly, cannot read label data")]
    IncorrectLabelEncoding,
    #[error("Byte read {0} and this does not represent correct length or pointer")]
    MalformedLenOrPtrInfo(u8),
    #[error("Exceeding maximum allowed labels that is 127")]
    ExceedingMaxNumberOfLabels,
    #[error("FQDN in the packet is exceeding the max limit")]
    FqdnTooLong,
    #[error("Offset provided in the packet does not point to existing FQDN in packet")]
    IncorrectPointerOffset,
    #[error("Name decompression resulted in {0} redirections exceeding the limit of 3")]
    TooManyRedirections(u8),
    #[error("Unable to read data from the buffer")]
    InsufficientData,
    #[error("Encoding is not ASCII character")]
    NotAsciiCharacter,
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
    // TODO: AOI, if fixed array of size 64 is performant and secure. Could be put on stack
    labels: Vec<String>
}

pub struct FqdnBuilder<S>
where
    S: FqdnState,
{
    labels: Vec<String>,
    fqdn_length: u8,
    state: PhantomData<S>,
}

impl TryFrom for Fqdn {
    type Error = FqdnError;

    fn try_from_bytes(decoder: &mut Deserialize) -> FqdnResult<Fqdn> {
        let fqdn = FqdnBuilder::new().generate_from_bytes(decoder)?.build();

        Ok(fqdn)
    }
}

impl ToBytes for Fqdn {
    fn to_bytes(&self, encoder: &mut Serialize) {
        let mut name_compressed = false;

        for i in 0..self.labels.len() {
            let label = &self.labels[i];
            let partial_fqdn = self.convert_to_string(i);

            match encoder.set_name_compression(partial_fqdn) {
                None => encoder.write_string(label),
                Some(pos) => {
                    let offset = POINTER_MARKER | pos;
                    encoder.write_u16(offset);
                    name_compressed = true;
                    break;
                }
            };
        }

        if !name_compressed {
            encoder.write_u8(0);
        }
    }
}

impl Fqdn {
    pub fn to_owned_str(&self) -> String {
        self.convert_to_string(0)
    }

    fn convert_to_string(&self, i: usize) -> String{
        let labels = &self.labels[i..];

        let fqdn = labels.iter().fold(String::new(), |acc, label| {
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
            labels: Vec::with_capacity(MAX_NUMBER_OF_LABELS as usize),
            fqdn_length: 0,
            state: PhantomData,
        }
    }

    pub fn generate_from_bytes(
        mut self,
        decoder: &mut Deserialize,
    ) -> FqdnResult<FqdnBuilder<FqdnSet>> {
        self.generate_labels_recursively(decoder, 0)?;

        Ok(FqdnBuilder {
            labels: self.labels,
            fqdn_length: self.fqdn_length,
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
            fqdn_length: self.fqdn_length,
            state: PhantomData,
        }
    }

    fn generate_labels_recursively(
        &mut self,
        decoder: &mut Deserialize,
        jump_count: u8,
    ) -> FqdnResult<()> {
        if jump_count > MAX_REDIRECTIONS {
            return Err(FqdnError::TooManyRedirections(jump_count));
        }

        let mut is_indirection = false;
        let mut parsing_fsm = FqdnParsingFSM::Start;

        loop {
            parsing_fsm = match parsing_fsm {
                FqdnParsingFSM::Start => Self::get_parsing_state(decoder)?,

                FqdnParsingFSM::Length => {
                    let label_len = decoder.peek().map_err(|_| FqdnError::MissingLabelLength)?;

                    self.fqdn_length += label_len;
                    if self.fqdn_length == MAX_FQDN_LENGTH {
                        return Err(FqdnError::FqdnTooLong);
                    }

                    if self.labels.len() == MAX_NUMBER_OF_LABELS as usize {
                        return Err(FqdnError::ExceedingMaxNumberOfLabels)
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

                    if offset >= decoder.cursor() {
                        return Err(FqdnError::IncorrectPointerOffset)
                    }

                    let mut cloned_decoder = decoder.cheap_clone(offset);
                    self.generate_labels_recursively(&mut cloned_decoder, jump_count + 1)?;
                    is_indirection = true;

                    FqdnParsingFSM::End
                }

                FqdnParsingFSM::End => {
                    if !is_indirection
                    {
                        let _ = decoder.read_u8().map_err(|_| FqdnError::InsufficientData);
                    }

                    break;
                }
            }
        }

        Ok(())
    }

    fn get_parsing_state(decoder: &Deserialize) -> FqdnResult<FqdnParsingFSM> {
        let ptr_or_len = Some(decoder.peek().map_err(|_| FqdnError::MissingLabelLength)?);

        match ptr_or_len {
            Some(0) => Ok(FqdnParsingFSM::End),
            Some(data) if data & PTR_MASK == PTR_MASK => Ok(FqdnParsingFSM::Pointer),
            Some(data) if data & PTR_MASK == LEN_MASK => Ok(FqdnParsingFSM::Length),
            Some(data) => Err(FqdnError::MalformedLenOrPtrInfo(data)),
            None => Err(FqdnError::MissingLabelLength),
        }
    }

    fn get_label(decoder: &mut Deserialize) -> FqdnResult<String> {
        let label_len = decoder
            .read_u8()
            .map_err(|_| FqdnError::MissingLabelLength)?;

        let label = decoder
            .read_n_bytes(label_len as u16)
            .map_err(|_| FqdnError::NotEnoughLabelData(label_len))?;

        // TODO: Once we have punycode support, can get rid of this
        if label.iter().any(|byte| !byte.is_ascii()) {
            return Err(FqdnError::NotAsciiCharacter);
        }

        // TODO: The label should technically be punycode encoding
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
mod fqdn_unittest {
    use crate::packet::seder::{deserializer::Deserialize, TryFrom};
    use crate::packet::fqdn::{Fqdn, FqdnError};

    #[test]
    fn name_all_lowercase() {
        let packet_bytes: [u8; 20] = [
            0x03, 0x77, 0x77, 0x77, 0x06, 0x67, 0x6f, 0x6f, 0x67, 0x6c, 0x65, 0x03, 0x63, 0x6f,
            0x6d, 0x00, 0x00, 0x01, 0x00, 0x01,
        ];
        let expected = String::from("www.google.com");

        let mut decoder = Deserialize::new(&packet_bytes);
        let fqdn = Fqdn::try_from_bytes(&mut decoder).unwrap();

        assert_eq!(fqdn.to_owned_str(), expected);
    }

    #[test]
    fn name_mixed_case() {
        let packet_bytes: [u8; 20] = [
            0x03, 0x77, 0x57, 0x77, 0x06, 0x67, 0x4f, 0x4f, 0x67, 0x6c, 0x65, 0x03, 0x63, 0x6f,
            0x6d, 0x00, 0x00, 0x01, 0x00, 0x01,
        ];
        let expected = String::from("www.google.com");

        let mut decoder = Deserialize::new(&packet_bytes);
        let fqdn = Fqdn::try_from_bytes(&mut decoder).unwrap();

        assert_eq!(fqdn.to_owned_str(), expected);
    }

    #[test]
    fn fqdn_with_ptr_jump() {
        let packet_bytes: [u8; 35] = [
            0xf2, 0xe8, 0x81, 0x80, 0x00, 0x01, 0x00, 0x01, 0x00, 0x00, 0x00, 0x00, 0x03, 0x77,
            0x77, 0x77, 0x06, 0x67, 0x6f, 0x6f, 0x67, 0x6c, 0x65, 0x03, 0x63, 0x6f, 0x6d, 0x00,
            0x00, 0x01, 0x00, 0x01, 0xc0, 0x0c, 0x00,
        ];

        let expected = String::from("www.google.com");

        let decoder = Deserialize::new(&packet_bytes);
        let mut decoder = decoder.cheap_clone(32);

        let fqdn = Fqdn::try_from_bytes(&mut decoder).unwrap();

        assert_eq!(fqdn.to_owned_str(), expected);
    }

    #[test]
    fn read_ptr_from_between() {
        let packet_bytes: [u8; 24] = [
            0x01, 0x61, 0x0c, 0x72, 0x6F, 0x6F, 0x74, 0x2d, 0x73, 0x65, 0x72, 0x76, 0x65, 0x72,
            0x73, 0x03, 0x6e, 0x65, 0x74, 0x00, // a.root-servers.net.
            0x01, 0x62, 0xc0, 0x02, // b.root-servers.net.
        ];

        let expected = String::from("b.root-servers.net");

        let decoder = Deserialize::new(&packet_bytes);
        let mut decoder = decoder.cheap_clone(20);

        let fqdn = Fqdn::try_from_bytes(&mut decoder).unwrap();

        assert_eq!(fqdn.to_owned_str(), expected);
    }

    #[test]
    fn read_ptr_multi_jump() {
        let packet_bytes: [u8; 19] = [
            0x01, 0x61, 0x03, 0x66, 0x6F, 0x6F, 0x03, 0x63, 0x6F, 0x6D, 0x00, // a.foo.com.
            0x01, 0x62, 0xc0, 0x02, // b.foo.com.
            0x01, 0x64, 0xc0, 0x0b, // d.b.foo.com.
        ];

        let expected = String::from("d.b.foo.com");

        let decoder = Deserialize::new(&packet_bytes);
        let mut decoder = decoder.cheap_clone(15);

        let fqdn = Fqdn::try_from_bytes(&mut decoder).unwrap();

        assert_eq!(fqdn.to_owned_str(), expected);
    }

    #[test]
    fn error_too_many_jumps() {
        let packet_bytes: [u8; 31] = [
            0x01, 0x61, 0x03, 0x66, 0x6F, 0x6F, 0x03, 0x63, 0x6F, 0x6D, 0x00, // a.foo.com.
            0x01, 0x62, 0xc0, 0x02, // b.foo.com.
            0x01, 0x64, 0xc0, 0x0b, // d.b.foo.com.
            0x01, 0x65, 0xc0, 0x0f, // e.d.b.foo.com.
            0x01, 0x66, 0xc0, 0x13, // f.e.d.b.foo.com.
            0x01, 0x67, 0xc0, 0x17, // g.f.e.d.b.foo.com
        ];

        let decoder = Deserialize::new(&packet_bytes);
        let mut decoder = decoder.cheap_clone(23);

        assert_eq!(Fqdn::try_from_bytes(&mut decoder), Err(FqdnError::TooManyRedirections(4)));
    }

    #[test]
    fn error_invalid_ascii_char() {
        let packet_bytes: [u8; 20] = [
            0x03, 0x77, 0x77, 0x77, 0x06, 0xff, 0x6f, 0x6f, 0x67, 0x6c, 0x65, 0x03, 0x63, 0x6f,
            0x6d, 0x00, 0x00, 0x01, 0x00, 0x01,
        ];

        let mut decoder = Deserialize::new(&packet_bytes);

        assert_eq!(Fqdn::try_from_bytes(&mut decoder), Err(FqdnError::NotAsciiCharacter));
    }

    #[test]
    fn error_malformed_length() {
        let packet_bytes: [u8; 20] = [
            0x03, 0x77, 0x77, 0x77, 0x46, 0x67, 0x6f, 0x6f, 0x67, 0x6c, 0x65, 0x03, 0x63, 0x6f,
            0x6d, 0x00, 0x00, 0x01, 0x00, 0x01,
        ];

        let mut decoder = Deserialize::new(&packet_bytes);

        assert_eq!(Fqdn::try_from_bytes(&mut decoder), Err(FqdnError::MalformedLenOrPtrInfo(0x46)));
    }

    #[test]
    fn error_malformed_ptr() {
        let packet_bytes: [u8; 15] = [
            0x01, 0x61, 0x03, 0x66, 0x6F, 0x6F, 0x03, 0x63, 0x6F, 0x6D, 0x00, // a.foo.com.
            0x01, 0x62, 0x80, 0x02, // b.foo.com.
        ];

        let decoder = Deserialize::new(&packet_bytes);
        let mut decoder = decoder.cheap_clone(11);

        assert_eq!(Fqdn::try_from_bytes(&mut decoder), Err(FqdnError::MalformedLenOrPtrInfo(0x80)));
    }

    #[test]
    fn error_exceeding_label_length() {
        let label: [u8; 2] = [0x01, 0x61];
        let label_counts = 130;

        let wire_data: Vec<u8> = label.iter().cloned().cycle().take(label_counts * label.len()).collect();

        let mut decoder = Deserialize::new(&wire_data);

        assert_eq!(Fqdn::try_from_bytes(&mut decoder), Err(FqdnError::ExceedingMaxNumberOfLabels));
    }

    #[test]
    fn error_label_length_without_label() {
        let wire_data: [u8; 8] = [
            0x03, 0x77, 0x77, 0x77, 0x06, 0x067, 0x6f, 0x6f
        ];

        let mut decoder = Deserialize::new(&wire_data);

        assert_eq!(Fqdn::try_from_bytes(&mut decoder), Err(FqdnError::NotEnoughLabelData(6)));
    }

    #[test]
    fn error_pointer_is_in_future() {
        let wire_data: [u8; 15] = [
            0x01, 0x61, 0x03, 0x66, 0x6F, 0x6F, 0x03, 0x63, 0x6F, 0x6D, 0x00, // a.foo.com.
            0x01, 0x62, 0xc0, 0x0f, // b.foo.com.
        ];

        let mut decoder = Deserialize::new(&wire_data);
        decoder = decoder.cheap_clone(11);

        assert_eq!(Fqdn::try_from_bytes(&mut decoder), Err(FqdnError::IncorrectPointerOffset))
    }
}
