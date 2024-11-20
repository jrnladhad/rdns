use super::header::Header;
use super::question::Question;

struct Message {
    header: Header,
    question: Question,
    // answer:  Vec<Record>,
    // additional: Vec<Record>,
    // authority: Vec<Record>
}
