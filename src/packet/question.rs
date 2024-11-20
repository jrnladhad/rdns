use crate::packet::bin_reader::BinReader;
use crate::packet::fqdn::Fqdn;
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

#[derive(Debug, PartialEq, Clone, Copy)]
enum QuestionType {
    A,
    NS,
    CNAME,
    SOA,
    WKS,
    PTR,
    INFO,
    MINFO,
    MX,
    TXT,
    AXFR,
    ALL,
    UNKNOWN,
}

impl From<u16> for QuestionType {
    fn from(value: u16) -> Self {
        match value {
            1 => QuestionType::A,
            2 => QuestionType::NS,
            5 => QuestionType::CNAME,
            6 => QuestionType::SOA,
            255 => QuestionType::ALL,
            _ => QuestionType::UNKNOWN,
        }
    }
}

#[derive(Debug, PartialEq, Copy, Clone)]
enum QuestionClass {
    IN,
    CH,
    HS,
    ANY,
    UNKNOWN,
}

impl From<u16> for QuestionClass {
    fn from(value: u16) -> Self {
        match value {
            1 => QuestionClass::IN,
            3 => QuestionClass::CH,
            4 => QuestionClass::HS,
            255 => QuestionClass::ANY,
            _ => QuestionClass::UNKNOWN,
        }
    }
}

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
struct QuestionTypeSet(QuestionType);

trait QuestionTypeState {}
impl QuestionTypeState for QuestionTypeUnset {}
impl QuestionTypeState for QuestionTypeSet {}

#[derive(Debug, PartialEq)]
pub struct Question {
    qname: Fqdn,
    qtype: QuestionType,
    qclass: QuestionClass,
}

pub struct QuestionBuilder<QN, QT>
where
    QN: FqdnState,
    QT: QuestionTypeState
{
    qname: QN,
    qtype: QT,
    qclass: QuestionClass
}

impl Default for QuestionBuilder<FqdnUnset, QuestionTypeUnset> {
    fn default() -> Self {
        QuestionBuilder {
            qname: FqdnUnset,
            qtype: QuestionTypeUnset,
            qclass: QuestionClass::IN
        }
    }
}

impl Question {
    pub fn from_bytes(decoder: &mut BinReader) -> QuestionResult<Question> {
        let qname = Fqdn::from_bytes(decoder).map_err(|_| QuestionError::NameReadingError)?;

        let qtype = QuestionType::from(
            decoder
                .read_u16()
                .map_err(|_| QuestionError::TypeReadingError)?,
        );

        let qclass = QuestionClass::from(
            decoder
                .read_u16()
                .map_err(|_| QuestionError::ClassReadingError)?,
        );

        let question = QuestionBuilder::new()
            .question_name(qname)
            .question_type(qtype)
            .question_class(qclass)
            .build();

        Ok(question)
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
            qclass: self.qclass
        }
    }
}

impl QuestionBuilder<FqdnSet, QuestionTypeUnset> {
    pub fn question_type(self, qtype: QuestionType) -> QuestionBuilder<FqdnSet, QuestionTypeSet> {
        QuestionBuilder {
            qname: self.qname,
            qtype: QuestionTypeSet(qtype),
            qclass: self.qclass
        }
    }
}

impl QuestionBuilder<FqdnSet, QuestionTypeSet> {
    pub fn build(self) -> Question {
        Question {
            qname: self.qname.0,
            qtype: self.qtype.0,
            qclass: self.qclass
        }
    }
}

impl<QN, QT> QuestionBuilder<QN, QT>
where
    QN: FqdnState,
    QT: QuestionTypeState
{
    pub fn question_class(self, qclass: QuestionClass) -> QuestionBuilder<QN, QT> {
        QuestionBuilder {
            qname: self.qname,
            qtype: self.qtype,
            qclass
        }
    }
}

#[cfg(test)]
mod test {
    use crate::packet::bin_reader::BinReader;
    use crate::packet::fqdn::{FqdnBuilder};
    use crate::packet::question::{Question, QuestionBuilder};
    use crate::packet::question::QuestionClass::IN;
    use crate::packet::question::QuestionType::A;

    #[test]
    fn read_question_success() {
        let packet_bytes: [u8; 20] = [
            0x03, 0x77, 0x77, 0x77, 0x06, 0x67, 0x6f, 0x6f,
            0x67, 0x6c, 0x65, 0x03, 0x63, 0x6f, 0x6d, 0x00,
            0x00, 0x01, 0x00, 0x01
        ];

        let expected_qname = FqdnBuilder::new()
            .generate_from_string(String::from("www.google.com"))
            .build();

        let expected_question = QuestionBuilder::new()
            .question_name(expected_qname)
            .question_class(IN)
            .question_type(A)
            .build();

        let mut decoder = BinReader::new(&packet_bytes);
        let actual_question = Question::from_bytes(&mut decoder).unwrap();

        assert_eq!(actual_question, expected_question);
    }
}
