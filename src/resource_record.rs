use std::io::Bytes;
use std::net::Ipv4Addr;

use crate::domain_name::DomainName;
use crate::msg::MessageError;
use crate::serialize::{Deserialize, DeserializationError, read_i32, read_ipv4, read_u16, Serialize};

#[derive(Copy, Clone)]
pub(crate) enum Class {
    Internet = 1,
    Chaos = 3
}

impl From<u16> for Class {
    fn from(value: u16) -> Self {
        match value {
            1 => Class::Internet,
            3 => Class::Chaos,
            _ => panic!("Invalid class code")
        }
    }
}

#[derive(Copy, Clone)]
pub(crate) enum Type {
    A = 1,
    NameServer = 2,
    CName = 5,
    SOA = 6,
    WKS = 11,
    PTR = 12,
    MailExchange = 15
}

impl TryFrom<u16> for Type {
    type Error = MessageError;

    fn try_from(value: u16) -> Result<Self, Self::Error> {
        match value {
            1 => Ok(Type::A),
            2 => Ok(Type::NameServer),
            5 => Ok(Type::CName),
            6 => Ok(Type::SOA),
            11 => Ok(Type::WKS),
            12 => Ok(Type::PTR),
            15 => Ok(Type::MailExchange),
            _ => Err(MessageError::InvalidMessageType)
        }
    }
}

pub(crate) enum ResponseData {
    A(Ipv4Addr),
    CName(DomainName)
    // TODO: implement:
    // - CNAME
    // - SOA
    // - WKS
    // - PTR
    // - HINFO
    // - MINFO
    // - MX
    // - TXT
}

pub struct ResourceRecordHeader {
    name: DomainName,
    rr_type: Type,
    rr_class: Class,
    ttl: i32,
    rdlength: u16,
}

impl ResourceRecordHeader {
    pub(crate) fn new(
        name: DomainName,
        rr_type: Type,
        rr_class: Class,
        ttl: i32,
        rdlength: u16,
    ) -> Self {
        Self {
            name,
            rr_type,
            rr_class,
            ttl,
            rdlength,
        }
    }

    pub(crate) fn rr_type(&self) -> Type {
        self.rr_type
    }
}

impl Serialize for ResourceRecordHeader {
    fn serialize(&self) -> Vec<u8> {
        todo!()
    }
}

impl Deserialize for ResourceRecordHeader {
    fn deserialize(bytes: &[u8], offset: usize) -> Result<(usize, Self), DeserializationError> {
        let (mut read_bytes, name) = DomainName::deserialize(bytes, offset)?;

        let (off, rr_type) = read_u16(bytes, offset + read_bytes)?;
        let rr_type = match Type::try_from(rr_type) {
            Ok(rr_type) => rr_type,
            Err(_) => return Err(DeserializationError::InvalidData(format!("Invalid RR Type, {:?}", rr_type)))
        };
        read_bytes += off;

        let (off, rr_class) = read_u16(bytes, offset + read_bytes)?;
        let rr_class = match Class::try_from(rr_class) {
            Ok(rr_class) => rr_class,
            Err(_) => return Err(DeserializationError::InvalidData(format!("Invalid RR Type, {:?}", rr_class)))
        };
        read_bytes += off;

        let (off, ttl) = read_i32(bytes, offset + read_bytes)?;
        read_bytes += off;

        let (off, rdlength) = read_u16(bytes, offset + read_bytes)?;
        read_bytes += off;

        Ok((read_bytes, Self {
            name,
            rr_type,
            rr_class,
            ttl,
            rdlength
        }))
    }
}

// I decided to make a separate deserialize and serialize for ResourceRecord
// instead of making ResourceRecord implement the Serialize + Deserialize
// traits because I couldn't come up with better solutions for these problems:
// - I don't want to read the header twice. If I implemented Serialize + Deserialize I would have
// to read the header twice because first I would need to know which type of record I'm working
// with and, second, inside de deserialize function of that specific RR, I would need to read
// once again the header.
// - I don't want to specify the associated type error each time I want to use a Box<dyn ResourceRecord>
// I know this might not be the best solution since these traits may be a bit confusing, but I think
// this is the best way to keep advancing with the project and don't get stuck with this specific part
pub(crate) trait ResourceRecord {
    fn deserialize(header: ResourceRecordHeader, bytes: &[u8], offset: usize)
        -> Result<(usize, Self), DeserializationError> where Self: Sized;
    fn serialize() -> Vec<u8> where Self: Sized;
}

pub(crate) struct AResourceRecord {
    header: ResourceRecordHeader,
    ip: Ipv4Addr
}

impl AResourceRecord {
    fn new(header: ResourceRecordHeader, ip: Ipv4Addr) -> Self {
        Self {
            header,
            ip
        }
    }
}

impl ResourceRecord for AResourceRecord {
    fn deserialize(header: ResourceRecordHeader, bytes: &[u8], offset: usize)
        -> Result<(usize, Self), DeserializationError> {
        let (off, ip) = read_ipv4(bytes, offset)?;

        Ok((offset + off, Self { header, ip }))
    }

    fn serialize() -> Vec<u8> {
        todo!()
    }
}

pub(crate) struct CNameResourceRecord {
    header: ResourceRecordHeader,
    cname: DomainName
}

impl ResourceRecord for CNameResourceRecord {
    fn deserialize(header: ResourceRecordHeader, bytes: &[u8], offset: usize)
        -> Result<(usize, Self), DeserializationError>
    where
        Self: Sized
    {
        let (off, cname) = DomainName::deserialize(bytes, offset)?;

        Ok((off, Self {
            header,
            cname
        }))
    }

    fn serialize() -> Vec<u8>
    where
        Self: Sized
    {
        todo!()
    }
}

pub(crate) struct ResourceRecordFactory;

impl ResourceRecordFactory {
    pub(crate) fn get_rr(header: ResourceRecordHeader, bytes: &[u8], offset: usize)
        -> Result<(usize, Box<dyn ResourceRecord>), DeserializationError> {
        match header.rr_type() {
            Type::A => {
                let (off, rr) = AResourceRecord::deserialize(header, bytes, offset)?;
                Ok((off, Box::new(rr)))
            },
            Type::CName => {
                let (off, rr) = CNameResourceRecord::deserialize(header, bytes, offset)?;
                Ok((off, Box::new(rr)))
            },
            _ => todo!()
        }
    }
}