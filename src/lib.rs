// #![feature(type_alias_impl_trait)]

mod packet;
mod records;
// use std::error::Error;
// use std::str;
// // use std::fs;
//
// mod packet;
//
//
// pub struct Config {
//     operation: String,
//     file_path: String,
// }
//
// impl Config {
//     pub fn build(args: &[String]) -> Result<Config, &'static str> {
//         if args.len() < 3 {
//             return Err(
//                 "Not enough arguments. Arguments should provide operation type and file path",
//             );
//         }
//
//         let operation = args[1].clone();
//         let file_path = args[2].clone();
//
//         match operation.to_lowercase().as_str() {
//             "parse" => {}
//             "pack" => {}
//             _ => return Err("{operation} is not supported, only 'parse' or 'pack'"),
//         }
//
//         Ok(Config {
//             operation,
//             file_path,
//         })
//     }
// }
//
// #[derive(Debug, Default)]
// pub struct HeaderFlags {
//     query_or_response: u8,
//     opcode: u8,
//     authoritative_answer: u8,
//     truncation: u8,
//     recursion_desired: u8,
//     recursion_available: u8,
//     zero: u8,
//     response_code: u8,
// }
//
// impl HeaderFlags {
//     const QR_MASK: u16 = 1 << 15;
//     const OPCODE_MASK: u16 = 1 << 11;
//     const AA_MASK: u16 = 1 << 10;
//     const TC_MASK: u16 = 1 << 9;
//     const RD_MASK: u16 = 1 << 8;
//     const RA_MASK: u16 = 1 << 7;
//     const ZERO_MASK: u16 = 7 << 4;
//     const RC_MASK: u16 = 15;
//
//     pub fn build(header_flag_bytes: u16) -> Result<HeaderFlags, &'static str> {
//         let query_or_response = ((header_flag_bytes & HeaderFlags::QR_MASK) >> 15) as u8;
//         let opcode = ((header_flag_bytes & HeaderFlags::OPCODE_MASK) >> 11) as u8;
//         let authoritative_answer = ((header_flag_bytes & HeaderFlags::AA_MASK) >> 10) as u8;
//         let truncation = ((header_flag_bytes & HeaderFlags::TC_MASK) >> 9) as u8;
//         let recursion_desired = ((header_flag_bytes & HeaderFlags::RD_MASK) >> 8) as u8;
//         let recursion_available = ((header_flag_bytes & HeaderFlags::RA_MASK) >> 7) as u8;
//         let zero = ((header_flag_bytes & HeaderFlags::ZERO_MASK) >> 4) as u8;
//         let response_code = (header_flag_bytes & HeaderFlags::RC_MASK) as u8;
//
//         if zero != 0 {
//             return Err("Packet is malformed, Z flag should be all 0 bits");
//         }
//
//         if query_or_response == 0 && recursion_available != 0 {
//             return Err("Packet is malformed, RA cannot be set if QR is unset");
//         }
//
//         Ok(HeaderFlags {
//             query_or_response,
//             opcode,
//             truncation,
//             authoritative_answer,
//             recursion_desired,
//             recursion_available,
//             zero,
//             response_code,
//         })
//     }
// }
//
// #[derive(Debug, Default)]
// pub struct Header {
//     id: u16,
//     flags: HeaderFlags,
//     question_count: u16,
//     answer_count: u16,
//     authoritative_count: u16,
//     additional_count: u16,
// }
//
// impl Header {
//     pub fn build(header_bytes: &[u8]) -> Result<Header, &'static str> {
//         let flag_bytes = ((header_bytes[2] as u16) << 8) | (header_bytes[3] as u16);
//         let flags = HeaderFlags::build(flag_bytes)?;
//
//         let id = ((header_bytes[0] as u16) << 8) | (header_bytes[1] as u16);
//         let question_count = ((header_bytes[4] as u16) << 8) | (header_bytes[5] as u16);
//         let answer_count = ((header_bytes[6] as u16) << 8) | (header_bytes[7] as u16);
//         let authoritative_count = ((header_bytes[8] as u16) << 8) | (header_bytes[9] as u16);
//         let additional_count = ((header_bytes[10] as u16) << 8) | (header_bytes[11] as u16);
//
//         if flags.query_or_response == 0 && (answer_count != 0 || authoritative_count != 0) {
//             return Err("Malformed packet, QR bit is set and answer count or auth count is > 0");
//         }
//
//         Ok(Header {
//             id,
//             flags,
//             question_count,
//             answer_count,
//             authoritative_count,
//             additional_count,
//         })
//     }
// }
//
// #[derive(Debug, Default)]
// pub struct FQDN
// {
//     name: String,
//     label_count: usize,
//     name_len: usize
// }
//
// impl FQDN
// {
//     pub fn new(name: String, label_count: usize, name_len: usize) -> Result<FQDN, &'static str>
//     {
//         Ok(FQDN
//         {
//             name,
//             label_count,
//             name_len
//         })
//     }
// }
//
// #[derive(Debug, Default)]
// pub struct QuestionSection {
//     qname: FQDN,
//     qtype: u16,
//     qclass: u16,
//     bytes_consumed: usize
// }
//
// impl QuestionSection {
//     pub fn build(data_on_wire: &[u8]) -> Result<QuestionSection, &'static str> {
//         let mut qname = String::new();
//         let mut i = 0;
//         let mut label_count: usize = 0;
//         let mut name_len = 0;
//
//         while i < data_on_wire.len() && data_on_wire[i] != 0 {
//             let strlen = data_on_wire[i] as usize;
//
//             if i + strlen >= data_on_wire.len() {
//                 return Err("Label size on wire is bigger than total packet");
//             }
//
//             let str = str::from_utf8(&data_on_wire[i + 1..i + strlen + 1])
//                 .unwrap_or("");
//
//             if str.is_empty() {
//                 return Err("Unable to generate label");
//             }
//
//             qname.push_str(str);
//             qname.push('.');
//             label_count += 1;
//             name_len += strlen + 1;
//
//             i = i + strlen + 1;
//         }
//
//         if i < data_on_wire.len() && data_on_wire[i] != 0 {
//             return Err("Did not end at the root label");
//         }
//
//         let fqdn = FQDN::new(qname, label_count, name_len)?;
//
//         // check on this and qclass about the err conditions
//         i += 1;
//         if i + 1 >= data_on_wire.len() {
//             return Err("QTYPE not provided");
//         }
//
//         let qtype = (data_on_wire[i] as u16) << 8 | (data_on_wire[i + 1] as u16);
//
//         i += 2;
//         if i + 1 >= data_on_wire.len() {
//             return Err("QCLASS not provided");
//         }
//
//         let qclass = (data_on_wire[i] as u16) << 8 | (data_on_wire[i + 1] as u16);
//
//         Ok(QuestionSection {
//             qname: fqdn,
//             qtype,
//             qclass,
//             bytes_consumed: i + 2
//         })
//     }
// }
//
// #[derive(Debug, Default)]
// pub struct Message {
//     header: Header,
//     question_section: QuestionSection,
// }
//
// impl Message {
//     // fn read_names(data: &[u8]) ->Result<String, &'static str>
//     // {
//     //     const IS_POINTER: u8 = 0xc0;
//     //     let mut index: usize = 0;
//     //
//     //     loop
//     //     {
//     //         if data[index] & IS_POINTER == IS_POINTER
//     //         {
//     //             // skip for now.
//     //             // but this is a recursive call
//     //         }
//     //
//     //         // hitting this that means it is a length
//     //         let label_len: u8 = data[index];
//     //         index += 1;
//     //
//     //         let label = String::from_utf8();
//     //     }
//     // }
//
//     pub fn build(data_on_wire: &[u8]) -> Result<Message, &'static str> {
//         if data_on_wire.len() < 12 {
//             return Err("Packet size is too small, it should be at least 12 bytes");
//         }
//         let packet = vec![0x00];
//         let mut decoder = Decoder::new(packet);
//
//         // extract the header
//         let header = Header::build(&data_on_wire[..13])?;
//
//         // extract the question section, this should be from byte 13 onwards
//         let question_section = QuestionSection::build(&data_on_wire[12..])?;
//
//         if header.flags.query_or_response == 0 && question_section.bytes_consumed + 12 != data_on_wire.len()
//         {
//             return Err("A DNS query packet has additional data")
//         }
//
//         // extract answer, auth, and additional section
//         // These sections are of resource record type
//         // let iter = question_section.bytes_consumed + 12;
//
//         // we have reached a point of refactoring. Reading names from packet is duplicated across two
//         // places. A better design would be to have a name / fqdn class that provides implementation
//         // for reading names from a byte array.
//
//         Ok(Message {
//             header,
//             question_section,
//         })
//     }
// }
//
// // #[cfg(tests)]
// // mod tests {
// //     use super::*;
// //
// //     #[tests]
// //     fn valid_query_header() {
// //         let packet_bytes: [u8; 32] = [
// //             0xf2, 0xe8, 0x01, 0x00, 0x00, 0x01, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x03, 0x77,
// //             0x77, 0x77, 0x06, 0x67, 0x6f, 0x6f, 0x67, 0x6c, 0x65, 0x03, 0x63, 0x6f, 0x6d, 0x00,
// //             0x00, 0x01, 0x00, 0x01,
// //         ];
// //
// //         let parsed_data = Message::build(&packet_bytes).unwrap_or(Message::default());
// //
// //         assert_eq!(parsed_data.header.id, 62184);
// //         assert_eq!(parsed_data.header.flags.query_or_response, 0);
// //         assert_eq!(parsed_data.header.flags.opcode, 0);
// //         assert_eq!(parsed_data.header.flags.authoritative_answer, 0);
// //         assert_eq!(parsed_data.header.flags.truncation, 0);
// //         assert_eq!(parsed_data.header.flags.recursion_desired, 1);
// //         assert_eq!(parsed_data.header.flags.recursion_available, 0);
// //         assert_eq!(parsed_data.header.flags.zero, 0);
// //         assert_eq!(parsed_data.header.flags.response_code, 0);
// //         assert_eq!(parsed_data.header.question_count, 1);
// //         assert_eq!(parsed_data.header.answer_count, 0);
// //         assert_eq!(parsed_data.header.authoritative_count, 0);
// //         assert_eq!(parsed_data.header.additional_count, 0);
// //         assert_eq!(parsed_data.question_section.qname.name, "www.google.com.");
// //         assert_eq!(parsed_data.question_section.qname.label_count, 3);
// //         assert_eq!(parsed_data.question_section.qname.name_len, 15);
// //         assert_eq!(parsed_data.question_section.qtype, 1);
// //         assert_eq!(parsed_data.question_section.qclass, 1);
// //     }
// // }
//
// // pub fn run() -> Result<(), Box<dyn Error>> {
// //     // let contents = fs::read(config.file_path)?;
// //
// //     let packet_bytes: [u8; 32] = [
// //         0x45, 0xa9, 0x01, 0x00, 0x00, 0x01, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x03, 0x77, 0x77,
// //         0x77, 0x06, 0x67, 0x6f, 0x6f, 0x67, 0x6c, 0x65, 0x03, 0x63, 0x6f, 0x6d, 0x00, 0x00, 0x01,
// //         0x00, 0x01,
// //     ];
// //
// //     let parsed_data = Message::build(&packet_bytes)?;
// //
// //     println!("{parsed_data:#?}");
// //
// //     Ok(())
// // }
