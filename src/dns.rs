use std::thread::panicking;
use regex::Regex;
use crate::serialize::{Deserialize, Serialize};


pub struct DomainName(String);

impl DomainName {
    pub fn from_string(domain_name: &str) -> Self {
        Self::is_valid(domain_name.to_string());

        Self(domain_name.to_string())
    }

    fn is_valid(domain_name: String) -> bool {
        // I literally copied this regex from stackoverflow
        let reg = Regex::new(
            r"^(([a-zA-Z0-9]|[a-zA-Z0-9][a-zA-Z0-9\-]*[a-zA-Z0-9])\.)*([A-Za-z0-9]|[A-Za-z0-9][A-Za-z0-9\-]*[A-Za-z0-9])$")
            .unwrap();

        if !reg.is_match(&domain_name) {
            panic!("Invalid domain name");
        }

        true
    }
}

impl Serialize for DomainName {
    fn as_bytes(&self) -> Vec<u8> {
        let mut bytes = vec![];

        for label in self.0.split('.') {
            let label_length = label.len() as u8;
            bytes.push(label_length);
            bytes.extend_from_slice(label.as_bytes());
        }

        bytes.push(0);

        bytes
    }
}

#[derive(Copy, Clone)]
pub enum Type {
    A = 1,
    NameServer = 2,
    CName = 5,
    SOA = 6,
    WKS = 11,
    PTR = 12,
    MailExchange = 15
}

#[derive(Copy, Clone)]
pub enum Class {
    Internet = 1,
    Chaos = 3 // ??
}

pub struct ResourceRecord {
    name: String,
    rr_type: Type, // Name's rr_type and not type because of the keyword
    class: Class,
    ttl: u32,
    rdlength: u16,
    rdata: usize // TODO: change usize
}

#[derive(Copy, Clone)]
enum Opcode {
    StandardQuery = 0,
    StatusQuery = 2
}

impl Opcode {
    fn from_u8(opcode: u8) -> Self {
        match opcode {
            0 => Opcode::StandardQuery,
            2 => Opcode::StatusQuery,
            _ => panic!("Invalid opcode")
        }
    }
}

#[derive(Copy, Clone)]
enum ResponseCode {
    NoError = 0,
    FormatError = 1,
    ServerError = 2,
    NameError = 3,
    NotImplementedError = 4,
    RefusedError = 5
}

impl ResponseCode {
    fn from_u8(response_code: u8) -> Self {
        match response_code {
            0 => ResponseCode::NoError,
            1 => ResponseCode::FormatError,
            2 => ResponseCode::ServerError,
            3 => ResponseCode::NameError,
            4 => ResponseCode::NotImplementedError,
            5 => ResponseCode::RefusedError,
            _ => panic!("Invalid response code")
        }
    }
}

#[derive(Copy, Clone)]
enum MessageType {
    Query = 0,
    Response = 1
}

impl MessageType {
    fn from_u8(message_type: u8) -> Self {
        match message_type {
            0 => MessageType::Query,
            1 => MessageType::Response,
            _ => panic!("Invalid message type")
        }
    }
}

pub struct Header {
    id: u16,
    qr: MessageType,
    opcode: Opcode,
    authoritative: bool,
    truncation: bool,
    recursion_desired: bool,
    recursion_available: bool,
    response_code: ResponseCode,
    qdcount: u16,
    ancount: u16,
    nscount: u16,
    arcount: u16
}

impl Header {
    fn standard_query_from_id(id: u16) -> Self {
        Self {
            id,
            qr: MessageType::Query,
            opcode: Opcode::StandardQuery,
            authoritative: false,
            truncation: false,
            recursion_desired: true,
            recursion_available: false,
            response_code: ResponseCode::NoError,
            qdcount: 1,
            ancount: 0,
            nscount: 0,
            arcount: 0
        }
    }
}

impl Serialize for Header {
    fn as_bytes(&self) -> Vec<u8> {
        let mut bytes = vec![];

        let id_bytes = self.id.to_be_bytes();
        bytes.extend_from_slice(&id_bytes);

        let qr= self.qr as u8;
        let opcode = self.opcode as u8;
        let aa: u8 = if self.authoritative { 1 } else { 0 };
        let tc: u8 = if self.truncation { 1 } else { 0 };
        let rd: u8 = if self.recursion_desired { 1 } else { 0 };
        let ra: u8 = if self.recursion_available { 1 } else { 0 };
        let rcode = self.response_code as u8;

        let upper_flags = qr << 7 | opcode << 3 | aa << 2 | tc << 1 | rd;
        let lower_flags = ra << 7 | 0 << 6 | 0 << 5 | 0 << 4 | rcode;
        bytes.extend_from_slice(&[upper_flags, lower_flags]);

        let counts = [self.qdcount, self.ancount, self.nscount, self.arcount];

        for count in counts.iter() {
            let count_bytes = count.to_be_bytes();
            bytes.extend_from_slice(&count_bytes);
        }

        bytes
    }
}

impl Deserialize for Header {
    fn from_bytes(bytes: &[u8]) -> Self {
        let id = u16::from_be_bytes([bytes[0], bytes[1]]);
        let flags = bytes[2];
        let qr = MessageType::from_u8((flags & 0b10000000) >> 7);
        let opcode = Opcode::from_u8((flags & 0b01111000) >> 3);
        let aa = (flags & 0b00000100) >> 2;
        let tc = (flags & 0b00000010) >> 1;
        let rd = flags & 0b00000001;
        let ra = (bytes[3] & 0b10000000) >> 7;
        let response_code = ResponseCode::from_u8(bytes[3] & 0b00001111);
        let qdcount = u16::from_be_bytes([bytes[4], bytes[5]]);
        let ancount = u16::from_be_bytes([bytes[6], bytes[7]]);
        let nscount = u16::from_be_bytes([bytes[8], bytes[9]]);
        let arcount = u16::from_be_bytes([bytes[10], bytes[11]);

        Self {
            id,
            qr,
            opcode,
            authoritative: aa == 1,
            truncation: tc == 1,
            recursion_desired: rd == 1,
            recursion_available: ra == 1,
            response_code,
            qdcount,
            ancount,
            nscount,
            arcount
        }
    }
}

pub struct Question {
    qname: DomainName,
    qtype: Type,
    qclass: Class
}

impl Question {
    fn new(domain_name: DomainName) -> Self {
        Self {
            qname: domain_name,
            qtype: Type::A,
            qclass: Class::Internet
        }
    }
}

impl Serialize for Question {
    fn as_bytes(&self) -> Vec<u8> {
        let mut bytes = vec![];

        let hostname_bytes = self.qname.0.as_bytes();
        bytes.extend_from_slice(&hostname_bytes);

        let qtype_bytes = self.qtype as u16;
        bytes.extend_from_slice(&qtype_bytes.to_be_bytes());

        let qclass_bytes = self.qclass as u16;
        bytes.extend_from_slice(&qclass_bytes.to_be_bytes());

        bytes
    }
}

pub struct Answer;
pub struct Authority;
pub struct Additional;

pub struct DNSMessage {
    header: Header,
    question: Question,
    answer: Option<Answer>,
    authority: Option<Authority>,
    additional: Option<Additional>
}

impl DNSMessage {
    pub fn new_from_components(
        header: Header,
        question: Question,
        answer: Option<Answer>,
        authority: Option<Authority>,
        additional: Option<Additional>) -> Self {
        Self {
            header,
            question,
            answer,
            authority,
            additional
        }
    }

    pub fn new_query_from_hostname(hostname: DomainName) -> Self {
        let id = rand::random::<u16>();
        let header = Header::standard_query_from_id(id);
        let question = Question::new(hostname);

        Self {
            header,
            question,
            answer: None,
            authority: None,
            additional: None
        }
    }
}

impl Serialize for DNSMessage {
    fn as_bytes(&self) -> Vec<u8> {
        let mut unflattened_bytes = vec![
            self.header.as_bytes(),
            self.question.as_bytes()
        ];

        unflattened_bytes.concat()
    }
}
