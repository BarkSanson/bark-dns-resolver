// TODO:
// - Check if TryFrom instead of From is better. Just panicking in case of a wrong code
// is probably not the best option. However, this implementation will work for now.
// - Limit label length to 63 octets

use std::ptr::read;
use crate::domain_name::DomainName;
use crate::resource_record::{AResourceRecord, Class, CNameResourceRecord, ResourceRecord, ResourceRecordFactory, ResourceRecordHeader, Type};
use crate::serialize::{Deserialize, DeserializationError, read_u16, Serialize};

const MESSAGE_HEADER_LENGTH: usize = 12;
const AA_FLAG_SHIFT: usize = 2;
const TC_FLAG_SHIFT: usize = 1;
const RA_FLAG_SHIFT: usize = 7;
const QR_FLAG_SHIFT: usize = 7;
const OPCODE_SHIFT: usize = 3;

pub enum MessageError {
    InvalidOpcode,
    InvalidResponseCode,
    InvalidMessageType
}

#[derive(Copy, Clone)]
enum Opcode {
    StandardQuery = 0,
    StatusQuery = 2
}

impl TryFrom<u8> for Opcode {
    type Error = MessageError;

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        match value {
            0 => Ok(Opcode::StandardQuery),
            2 => Ok(Opcode::StatusQuery),
            _ => Err(MessageError::InvalidOpcode)
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

impl TryFrom<u8> for ResponseCode {
    type Error = MessageError;

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        match value {
            0 => Ok(ResponseCode::NoError),
            1 => Ok(ResponseCode::FormatError),
            2 => Ok(ResponseCode::ServerError),
            3 => Ok(ResponseCode::NameError),
            4 => Ok(ResponseCode::NotImplementedError),
            5 => Ok(ResponseCode::RefusedError),
            _ => Err(MessageError::InvalidResponseCode)
        }
    }
}

#[derive(Copy, Clone)]
enum MessageType {
    Query = 0,
    Response = 1
}

impl TryFrom<u8> for MessageType {
    type Error = MessageError;

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        match value {
            0 => Ok(MessageType::Query),
            1 => Ok(MessageType::Response),
            _ => Err(MessageError::InvalidMessageType)
        }
    }
}

pub(crate) struct MessageHeader {
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

impl MessageHeader {
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

impl Serialize for MessageHeader {
    fn serialize(&self) -> Vec<u8> {
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

        let upper_flags =
            qr << QR_FLAG_SHIFT | opcode << OPCODE_SHIFT | aa << AA_FLAG_SHIFT | tc << TC_FLAG_SHIFT | rd;
        let lower_flags = ra << RA_FLAG_SHIFT | 0 << 6 | 0 << 5 | 0 << 4 | rcode;
        bytes.extend_from_slice(&[upper_flags, lower_flags]);

        let counts = [self.qdcount, self.ancount, self.nscount, self.arcount];

        for count in counts.iter() {
            let count_bytes = count.to_be_bytes();
            bytes.extend_from_slice(&count_bytes);
        }

        bytes
    }
}

impl Deserialize for MessageHeader {
    fn deserialize(bytes: &[u8], offset: usize) -> Result<(usize, Self), DeserializationError>
    where
        Self: Sized
    {
        if offset + MESSAGE_HEADER_LENGTH > bytes.len() {
            return Err(DeserializationError::BufferOverflow);
        }

        let mut read_bytes = 0usize;
        let (off, id) = read_u16(bytes, offset)?;
        read_bytes += off;

        let flags = bytes[offset + read_bytes];
        read_bytes += 1;

        let qr = match MessageType::try_from((flags & 0b10000000) >> QR_FLAG_SHIFT) {
            Ok(qr) => qr,
            Err(_) => return Err(DeserializationError::InvalidData(format!("Invalid QR, {:?}", (flags & 0b10000000) >> QR_FLAG_SHIFT)))
        };
        let opcode = match Opcode::try_from((flags & 0b01111000) >> OPCODE_SHIFT) {
            Ok(opcode) => opcode,
            Err(_) => return Err(DeserializationError::InvalidData(format!("Invalid Opcode, {:?}", (flags & 0b10000000) >> QR_FLAG_SHIFT)))
        };
        let aa = (flags & 0b00000100) >> AA_FLAG_SHIFT;
        let tc = (flags & 0b00000010) >> TC_FLAG_SHIFT;
        let rd = flags & 0b00000001;

        let flags = bytes[offset + read_bytes];
        read_bytes += 1;

        let ra = (flags & 0b10000000) >> RA_FLAG_SHIFT;
        let response_code = match ResponseCode::try_from(flags & 0b00001111) {
            Ok(rc) => rc,
            Err(_) => return Err(DeserializationError::InvalidData(format!("Invalid Response Code, {:?}", (flags & 0b10000000) >> QR_FLAG_SHIFT)))
        };

        let (off, qdcount) = read_u16(bytes, offset + read_bytes)?;
        read_bytes += off;

        let (off, ancount) = read_u16(bytes, offset + read_bytes)?;
        read_bytes += off;

        let (off, nscount) = read_u16(bytes, offset + read_bytes)?;
        read_bytes += off;

        let (off, arcount) = read_u16(bytes, offset + read_bytes)?;
        read_bytes += off;

        Ok((read_bytes, Self {
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
        }))
    }
}

pub(crate) struct Question {
    qname: DomainName,
    qtype: Type,
    qclass: Class
}

impl Question {
    fn new(qname: DomainName, qtype: Type, qclass: Class) -> Self {
        Self {
            qname,
            qtype,
            qclass
        }
    }

    fn new_from_domain_name(qname: DomainName) -> Self {
        Self {
            qname,
            qtype: Type::A,
            qclass: Class::Internet
        }
    }
}

impl Serialize for Question {
    fn serialize(&self) -> Vec<u8> {
        let mut bytes = vec![];

        let hostname_bytes = self.qname.serialize();
        bytes.extend_from_slice(&hostname_bytes);

        let qtype_bytes = self.qtype as u16;
        bytes.extend_from_slice(&qtype_bytes.to_be_bytes());

        let qclass_bytes = self.qclass as u16;
        bytes.extend_from_slice(&qclass_bytes.to_be_bytes());

        bytes
    }
}

impl Deserialize for Question {
    fn deserialize(bytes: &[u8], offset: usize)
        -> Result<(usize, Self), DeserializationError>
    where
        Self: Sized
    {
        let mut read_bytes = 0usize;
        let (off, qname) = DomainName::deserialize(&bytes, offset)?;
        read_bytes += off;

        let (off, qtype) = read_u16(bytes, offset + read_bytes)?;
        let qtype = match Type::try_from(qtype) {
            Ok(qtype) => qtype,
            Err(_) => return Err(DeserializationError::InvalidData(format!("Invalid QType, {:?}", qtype)))
        };
        read_bytes += off;

        let (off, qclass) = read_u16(bytes, offset + read_bytes)?;
        let qclass = match Class::try_from(qclass) {
            Ok(qtype) => qtype,
            Err(_) => return Err(DeserializationError::InvalidData(format!("Invalid QClass, {:?}", qclass)))
        };
        read_bytes += off;

        Ok((read_bytes, Self {
            qname,
            qtype,
            qclass
        }))
    }
}

pub(crate) struct DNSMessage {
    header: MessageHeader,
    question: Question,
    answers: Option<Vec<Box<dyn ResourceRecord>>>,
    authorities: Option<Vec<Box<dyn ResourceRecord>>>,
    additional: Option<Vec<Box<dyn ResourceRecord>>>
}

impl DNSMessage {
    pub(crate) fn new_from_components(
        header: MessageHeader,
        question: Question,
        answers: Option<Vec<Box<dyn ResourceRecord>>>,
        authorities: Option<Vec<Box<dyn ResourceRecord>>>,
        additional: Option<Vec<Box<dyn ResourceRecord>>>) -> Self {
        Self {
            header,
            question,
            answers,
            authorities,
            additional
        }
    }

    pub(crate) fn new_query_from_hostname(hostname: DomainName) -> Self {
        let id = rand::random::<u16>();
        let header = MessageHeader::standard_query_from_id(id);
        let question = Question::new_from_domain_name(hostname);

        Self {
            header,
            question,
            answers: None,
            authorities: None,
            additional: None
        }
    }
}

impl Serialize for DNSMessage {
    fn serialize(&self) -> Vec<u8> {
        let unflattened_bytes = vec![
            self.header.serialize(),
            self.question.serialize()
        ];

        unflattened_bytes.concat()
    }
}

impl Deserialize for DNSMessage {
    fn deserialize(bytes: &[u8], offset: usize) -> Result<(usize, Self), DeserializationError>{
        let mut read_bytes = 0usize;
        let (off, header) = MessageHeader::deserialize(bytes, offset)?;
        read_bytes += off;

        let (off, question) = Question::deserialize(bytes, offset + read_bytes)?;
        read_bytes += off;

        let mut answers = None;
        if header.ancount != 0 {
            let mut answers_vec: Vec<Box<dyn ResourceRecord>> = Vec::new();
            for _ in 0..header.ancount {
                let (off, rr_header) =
                    ResourceRecordHeader::deserialize(bytes, offset + read_bytes)?;
                read_bytes += off;

                let (off, rr) =
                    ResourceRecordFactory::get_rr(rr_header, bytes, offset + read_bytes)?;
                read_bytes += off;

                answers_vec.push(rr);
            }

            answers = Some(answers_vec);
        }

        let authorities = None;
        if header.nscount != 0 {

        }

        let additional = None;
        if header.arcount != 0 {

        }

        Ok((read_bytes, Self {
            header,
            question,
            answers,
            authorities,
            additional
        }))
    }
}
