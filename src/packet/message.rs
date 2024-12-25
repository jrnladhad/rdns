use super::question::Question;
use super::record::Record;
use crate::packet::headers::header::Header;
use thiserror::Error;
use crate::packet::seder::{deserializer::Deserialize, serializer::Serialize, TryFrom, ToBytes};

type MessageResult = Result<Message, MessageError>;

#[derive(Debug, Error)]
pub enum MessageError {
    #[error("Invalid header section")]
    InvalidHeader,
    #[error("Invalid question section")]
    InvalidQuestion,
    #[error("Invalid answer section")]
    InvalidAnswer,
    #[error("Invalid authority section")]
    InvalidAuthority,
    #[error("Invalid additional section")]
    InvalidAdditional,
}

trait HeaderState {}
struct HeaderUnset;
struct HeaderSet(Header);

impl HeaderState for HeaderUnset {}
impl HeaderState for HeaderSet {}

trait QuestionState {}
struct QuestionUnset;
struct QuestionSet(Question);

impl QuestionState for QuestionUnset {}
impl QuestionState for QuestionSet {}

#[derive(Debug, PartialEq)]
pub struct Message {
    header: Header,
    question: Question,
    answer_records: Vec<Record>,
    authority_records: Vec<Record>,
    additional_records: Vec<Record>,
}

struct MessageBuilder<H, Q>
where
    H: HeaderState,
    Q: QuestionState,
{
    header: H,
    question: Q,
    answer_records: Vec<Record>,
    authority_records: Vec<Record>,
    additional_records: Vec<Record>,
}

impl Default for MessageBuilder<HeaderUnset, QuestionUnset> {
    fn default() -> Self {
        MessageBuilder {
            header: HeaderUnset,
            question: QuestionUnset,
            answer_records: vec![],
            authority_records: vec![],
            additional_records: vec![],
        }
    }
}

impl TryFrom for Message {
    type Error = MessageError;

    fn try_from_bytes(decoder: &mut Deserialize) -> MessageResult {
        let header =  Header::try_from_bytes(decoder).map_err(|_| MessageError::InvalidHeader)?;
        let question = Question::try_from_bytes(decoder).map_err(|_| MessageError::InvalidQuestion)?;

        let mut answers: Vec<Record> = Vec::with_capacity(header.answer_count() as usize);
        for _ in 0..header.answer_count() {
            let answer = Record::try_from_bytes(decoder).map_err(|_| MessageError::InvalidAnswer)?;
            answers.push(answer);
        }

        let mut authorities: Vec<Record> = Vec::with_capacity(header.authority_count() as usize);
        for _ in 0..header.authority_count() {
            let answer = Record::try_from_bytes(decoder).map_err(|_| MessageError::InvalidAuthority)?;
            authorities.push(answer);
        }

        let mut additional: Vec<Record> = Vec::with_capacity(header.additional_count() as usize);
        for _ in 0..header.additional_count() {
            let answer = Record::try_from_bytes(decoder).map_err(|_| MessageError::InvalidAdditional)?;
            additional.push(answer);
        }

        let message = MessageBuilder::new()
            .header(header)
            .question(question)
            .answer(answers)
            .authority(authorities)
            .additional(additional)
            .build();

        Ok(message)
    }
}

impl ToBytes for Message {
    fn to_bytes(&self, encoder: &mut Serialize) {
        self.header.to_bytes(encoder);
        self.question.to_bytes(encoder);

        for record in &self.answer_records {
            record.to_bytes(encoder);
        }

        for record in &self.authority_records {
            record.to_bytes(encoder);
        }

        for record in &self.additional_records {
            record.to_bytes(encoder);
        }
    }
}

impl MessageBuilder<HeaderUnset, QuestionUnset> {
    pub fn new() -> Self {
        MessageBuilder::default()
    }

    pub fn header(self, header: Header) -> MessageBuilder<HeaderSet, QuestionUnset> {
        MessageBuilder {
            header: HeaderSet(header),
            question: self.question,
            answer_records: self.answer_records,
            authority_records: self.authority_records,
            additional_records: self.additional_records,
        }
    }
}

impl MessageBuilder<HeaderSet, QuestionUnset> {
    pub fn question(self, question: Question) -> MessageBuilder<HeaderSet, QuestionSet> {
        MessageBuilder {
            header: self.header,
            question: QuestionSet(question),
            answer_records: self.answer_records,
            authority_records: self.authority_records,
            additional_records: self.additional_records,
        }
    }
}

impl MessageBuilder<HeaderSet, QuestionSet> {
    pub fn build(self) -> Message {
        Message {
            header: self.header.0,
            question: self.question.0,
            answer_records: self.answer_records,
            authority_records: self.authority_records,
            additional_records: self.additional_records,
        }
    }
}

impl<H, Q> MessageBuilder<H, Q>
where
    H: HeaderState,
    Q: QuestionState,
{
    pub fn answer(self, answer: Vec<Record>) -> Self {
        MessageBuilder {
            header: self.header,
            question: self.question,
            answer_records: answer,
            authority_records: self.authority_records,
            additional_records: self.additional_records,
        }
    }

    pub fn authority(self, authority: Vec<Record>) -> Self {
        MessageBuilder {
            header: self.header,
            question: self.question,
            answer_records: self.answer_records,
            authority_records: authority,
            additional_records: self.additional_records,
        }
    }

    pub fn additional(self, additional: Vec<Record>) -> Self {
        MessageBuilder {
            header: self.header,
            question: self.question,
            answer_records: self.answer_records,
            authority_records: self.authority_records,
            additional_records: additional,
        }
    }
}

#[cfg(test)]
mod message_unittest {
    use crate::packet::seder::{deserializer::Deserialize, serializer::Serialize, TryFrom, ToBytes};
    use crate::packet::message::{Message, MessageBuilder};
    use crate::packet::record::record_unittest::{get_sample_a_record};
    use crate::packet::headers::header::header_unittest::get_response_header;
    use crate::packet::question::question_unittest::{get_google_a_question};

    #[test]
    fn google_a_ques_answer() {
        let wire_data: [u8; 48] = [
            0xf2, 0xe8, 0x81, 0x80, 0x00, 0x01, 0x00, 0x01, 0x00, 0x00, 0x00, 0x00, 0x03, 0x77,
            0x77, 0x77, 0x06, 0x67, 0x6f, 0x6f, 0x67, 0x6c, 0x65, 0x03, 0x63, 0x6f, 0x6d, 0x00,
            0x00, 0x01, 0x00, 0x01, 0xc0, 0x0c, 0x00, 0x01, 0x00, 0x01, 0x00, 0x00, 0x00, 0x68,
            0x00, 0x04, 0xac, 0xd9, 0x0e, 0xc4,
        ];

        let expected_header = get_response_header(62184);

        let expected_question = get_google_a_question();

        let answer_records = vec![get_sample_a_record()];

        let expected_message = MessageBuilder::new()
            .header(expected_header)
            .question(expected_question)
            .answer(answer_records)
            .build();

        let mut decoder = Deserialize::new(&wire_data);

        let actual_message = Message::try_from_bytes(&mut decoder).unwrap();

        assert_eq!(actual_message, expected_message);
    }

    #[test]
    fn serialize_google_response_message() {
        let expected_wire_data: [u8; 48] = [
            0xf2, 0xe8, 0x81, 0x80, 0x00, 0x01, 0x00, 0x01, 0x00, 0x00, 0x00, 0x00, 0x03, 0x77,
            0x77, 0x77, 0x06, 0x67, 0x6f, 0x6f, 0x67, 0x6c, 0x65, 0x03, 0x63, 0x6f, 0x6d, 0x00,
            0x00, 0x01, 0x00, 0x01, 0xc0, 0x0c, 0x00, 0x01, 0x00, 0x01, 0x00, 0x00, 0x00, 0x68,
            0x00, 0x04, 0xac, 0xd9, 0x0e, 0xc4,
        ];

        let expected_header = get_response_header(62184);

        let expected_question = get_google_a_question();

        let answer_records = vec![get_sample_a_record()];

        let expected_message = MessageBuilder::new()
            .header(expected_header)
            .question(expected_question)
            .answer(answer_records)
            .build();

        let mut encoder = Serialize::new();
        expected_message.to_bytes(&mut encoder);

        assert_eq!(encoder.bin_data(), expected_wire_data);
    }
}
