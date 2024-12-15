use crate::packet::fqdn::Fqdn;
use crate::packet::seder::deserializer::Deserialize;
use crate::packet::seder::serializer::Serialize;
use crate::packet::seder::{FromBytes, ToBytes};
use crate::records::record_class::RecordClass;
use crate::records::record_type::RecordType;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum QuestionError {
    #[error("Could not read name from the packet")]
    NameReadingError,
    #[error("Could not read the question type")]
    TypeReadingError,
    #[error("Could not read the class type")]
    ClassReadingError,
}

type QuestionResult<T> = Result<T, QuestionError>;

#[derive(Debug, Clone)]
struct FqdnUnset;
#[derive(Debug, Clone)]
struct FqdnSet(Fqdn);

trait FqdnState {}
impl FqdnState for FqdnUnset {}
impl FqdnState for FqdnSet {}

#[derive(Debug, Clone)]
struct QuestionTypeUnset;
#[derive(Debug, Clone)]
struct QuestionTypeSet(RecordType);

trait QuestionTypeState {}
impl QuestionTypeState for QuestionTypeUnset {}
impl QuestionTypeState for QuestionTypeSet {}

#[derive(Debug, PartialEq)]
pub struct Question {
    qname: Fqdn,
    qtype: RecordType,
    qclass: RecordClass,
}

pub struct QuestionBuilder<QN, QT>
where
    QN: FqdnState,
    QT: QuestionTypeState,
{
    qname: QN,
    qtype: QT,
    qclass: RecordClass,
}

impl Default for QuestionBuilder<FqdnUnset, QuestionTypeUnset> {
    fn default() -> Self {
        QuestionBuilder {
            qname: FqdnUnset,
            qtype: QuestionTypeUnset,
            qclass: RecordClass::IN,
        }
    }
}

impl FromBytes for Question {
    type Error = QuestionError;

    fn from_bytes(decoder: &mut Deserialize) -> QuestionResult<Question> {
        let qname = Fqdn::from_bytes(decoder).map_err(|_| QuestionError::NameReadingError)?;

        let qtype = decoder
            .read_u16()
            .map_err(|_| QuestionError::TypeReadingError)?;
        let qtype = RecordType::from(qtype);

        let qclass = decoder
            .read_u16()
            .map_err(|_| QuestionError::ClassReadingError)?;
        let qclass = RecordClass::from(qclass);

        let question = QuestionBuilder::new()
            .question_name(qname)
            .question_type(qtype)
            .question_class(qclass)
            .build();

        Ok(question)
    }
}

impl ToBytes for Question {
    fn to_bytes(&self, encoder: &mut Serialize) {
        self.qname.to_bytes(encoder);
        encoder.write_u16(self.qtype.into());
        encoder.write_u16(self.qclass.into());
    }
}

impl QuestionBuilder<FqdnUnset, QuestionTypeUnset> {
    pub fn new() -> Self {
        QuestionBuilder::default()
    }

    pub fn question_name(self, qname: Fqdn) -> QuestionBuilder<FqdnSet, QuestionTypeUnset> {
        QuestionBuilder {
            qname: FqdnSet(qname),
            qtype: self.qtype,
            qclass: self.qclass,
        }
    }
}

impl QuestionBuilder<FqdnSet, QuestionTypeUnset> {
    pub fn question_type(self, qtype: RecordType) -> QuestionBuilder<FqdnSet, QuestionTypeSet> {
        QuestionBuilder {
            qname: self.qname,
            qtype: QuestionTypeSet(qtype),
            qclass: self.qclass,
        }
    }
}

impl QuestionBuilder<FqdnSet, QuestionTypeSet> {
    pub fn build(self) -> Question {
        Question {
            qname: self.qname.0,
            qtype: self.qtype.0,
            qclass: self.qclass,
        }
    }
}

impl<QN, QT> QuestionBuilder<QN, QT>
where
    QN: FqdnState,
    QT: QuestionTypeState,
{
    pub fn question_class(self, qclass: RecordClass) -> QuestionBuilder<QN, QT> {
        QuestionBuilder {
            qname: self.qname,
            qtype: self.qtype,
            qclass,
        }
    }
}

#[cfg(test)]
pub mod question_unittest {
    use crate::packet::fqdn::FqdnBuilder;
    use crate::packet::question::RecordClass::IN;
    use crate::packet::question::RecordType::A;
    use crate::packet::question::{Question, QuestionBuilder};
    use crate::packet::seder::{deserializer::Deserialize, serializer::Serialize, FromBytes, ToBytes};

    pub fn get_google_a_question() -> Question {
        let expected_qname = FqdnBuilder::new()
            .generate_from_string(String::from("www.google.com"))
            .build();

        let expected_question = QuestionBuilder::new()
            .question_name(expected_qname)
            .question_class(IN)
            .question_type(A)
            .build();

        expected_question
    }

    #[test]
    fn read_question_success() {
        let packet_bytes: [u8; 20] = [
            0x03, 0x77, 0x77, 0x77, 0x06, 0x67, 0x6f, 0x6f, 0x67, 0x6c, 0x65, 0x03, 0x63, 0x6f,
            0x6d, 0x00, 0x00, 0x01, 0x00, 0x01,
        ];

        let expected_qname = FqdnBuilder::new()
            .generate_from_string(String::from("www.google.com"))
            .build();

        let expected_question = QuestionBuilder::new()
            .question_name(expected_qname)
            .question_class(IN)
            .question_type(A)
            .build();

        let mut decoder = Deserialize::new(&packet_bytes);
        let actual_question = Question::from_bytes(&mut decoder).unwrap();

        assert_eq!(actual_question, expected_question);
    }

    #[test]
    fn serialize_question() {
        let expected_serialization: [u8; 20] = [
            0x03, 0x77, 0x77, 0x77, 0x06, 0x67, 0x6f, 0x6f, 0x67, 0x6c, 0x65, 0x03, 0x63, 0x6f,
            0x6d, 0x00, 0x00, 0x01, 0x00, 0x01,
        ];

        let qname = FqdnBuilder::new()
            .generate_from_string(String::from("www.google.com"))
            .build();

        let question = QuestionBuilder::new()
            .question_name(qname)
            .question_class(IN)
            .question_type(A)
            .build();

        let mut encoder = Serialize::new();
        question.to_bytes(&mut encoder);

        assert_eq!(encoder.bin_data(), expected_serialization);
    }
}
