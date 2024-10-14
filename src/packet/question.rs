use crate::packet::bin_reader::BinReader;
use crate::packet::fqdn::FQDN;
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

#[derive(Debug, PartialEq)]
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

#[derive(Debug, PartialEq)]
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

pub struct Question {
    qname: FQDN,
    qtype: QuestionType,
    qclass: QuestionClass,
}

impl Question {
    pub fn from_bytes(decoder: &mut BinReader) -> QuestionResult<Question> {
        let qname = FQDN::from_bytes(decoder).map_err(|_| QuestionError::NameReadingError)?;

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

        Ok(Question {
            qname,
            qtype,
            qclass,
        })
    }
}

#[cfg(test)]
mod test {
    use crate::packet::bin_reader::BinReader;
    use crate::packet::question::{Question, QuestionClass, QuestionType};

    #[test]
    fn read_question_success() {
        let packet_bytes: [u8; 20] = [
            0x03, 0x77, 0x77, 0x77, 0x06, 0x67, 0x6f, 0x6f,
            0x67, 0x6c, 0x65, 0x03, 0x63, 0x6f, 0x6d, 0x00,
            0x00, 0x01, 0x00, 0x01
        ];

        let expected_qname = String::from("www.google.com");
        let expected_qtype = QuestionType::from(1);
        let expected_qclass = QuestionClass::from(1);

        let mut decoder = BinReader::new(&packet_bytes);
        let actual_question = Question::from_bytes(&mut decoder).unwrap();

        assert_eq!(actual_question.qname.to_str(), expected_qname);
        assert_eq!(actual_question.qtype, expected_qtype);
        assert_eq!(actual_question.qclass, expected_qclass);
    }
}
