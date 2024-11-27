use super::question::Question;
use super::record::Record;
use crate::packet::{bin_reader::BinReader, headers::header::Header};
use thiserror::Error;

#[derive(Debug, Error)]
enum MessageError {
    #[error("Invalid header")]
    InvalidHeader,
    #[error("Invalid question")]
    InvalidQuestion,
    #[error("Invalid answer")]
    InvalidAnswer,
    #[error("Invalid authority")]
    InvalidAuthority,
    #[error("Invalid additional")]
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

pub struct Message {
    header: Header,
    question: Question,
    answer_records: Vec<Record>,
    authority_records: Vec<Record>,
    additional_records: Vec<Record>,
}

pub struct MessageBuilder<H, Q>
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

impl Message {
    pub fn from_bytes(decoder: &mut BinReader) -> Result<Message, MessageError> {
        let header = Header::from_bytes(decoder).map_err(|_| MessageError::InvalidHeader)?;
        let question = Question::from_bytes(decoder).map_err(|_| MessageError::InvalidQuestion)?;

        let mut answers: Vec<Record> = Vec::with_capacity(header.answer_count() as usize);
        for _ in 0..header.answer_count() {
            let answer = Record::from_bytes(decoder).map_err(|_| MessageError::InvalidAnswer)?;
            answers.push(answer);
        }

        let mut authorities: Vec<Record> = Vec::with_capacity(header.answer_count() as usize);
        for _ in 0..header.answer_count() {
            let answer = Record::from_bytes(decoder).map_err(|_| MessageError::InvalidAnswer)?;
            authorities.push(answer);
        }

        let mut additional: Vec<Record> = Vec::with_capacity(header.answer_count() as usize);
        for _ in 0..header.answer_count() {
            let answer = Record::from_bytes(decoder).map_err(|_| MessageError::InvalidAnswer)?;
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
