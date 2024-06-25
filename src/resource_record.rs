use std::net::Ipv4Addr;
use std::ptr::read;
use crate::domain_name::DomainName;
use crate::serialize::{Deserialize, EncodingError, read_i32, read_ipv4, read_u16, Serialize};

#[derive(Copy, Clone)]
pub enum Class {
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
pub enum Type {
    A = 1,
    NameServer = 2,
    CName = 5,
    SOA = 6,
    WKS = 11,
    PTR = 12,
    MailExchange = 15
}

impl TryFrom<u16> for Type {
    type Error = EncodingError;

    fn try_from(value: u16) -> Result<Self, Self::Error> {
        match value {
            1 => Ok(Type::A),
            2 => Ok(Type::NameServer),
            5 => Ok(Type::CName),
            6 => Ok(Type::SOA),
            11 => Ok(Type::WKS),
            12 => Ok(Type::PTR),
            15 => Ok(Type::MailExchange),
            _ => Err()
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
}

impl Serialize for ResourceRecordHeader {
    type Error = EncodingError;

    fn serialize(&self) -> Result<Vec<u8>, Self::Error> {
        todo!()
    }
}

impl Deserialize for ResourceRecordHeader {
    type Error = EncodingError;

    fn deserialize(bytes: &[u8], offset: usize) -> Result<(Self, usize), Self::Error> {
        let (name, mut read_bytes) = DomainName::deserialize(bytes, offset)?;

        let (rr_type_u16, new_read_bytes) = read_u16(bytes, offset + read_bytes)?;
        let rr_type = Type::try_from(rr_type_u16)?;
        read_bytes += new_read_bytes;

        let (rr_class_u16, new_read_bytes) = read_u16(bytes, offset + read_bytes)?;
        let rr_class= Class::try_from(rr_class_u16)?;
        read_bytes += new_read_bytes;

        let (ttl, new_read_bytes) = read_i32(bytes, offset + read_bytes)?;
        read_bytes += new_read_bytes;

        let (rdlength, new_read_bytes) = read_u16(bytes, offset + read_bytes)?;
        read_bytes = new_read_bytes;

        Ok((Self {
            name,
            rr_type,
            rr_class,
            ttl,
            rdlength
        }, read_bytes))
    }
}

trait ResourceRecord: Serialize + Deserialize {}

pub(crate) struct AResourceRecord {
    header: ResourceRecordHeader,
    data: Ipv4Addr
}

impl AResourceRecord {
    fn new(header: ResourceRecordHeader, data: Ipv4Addr) -> Self {
        Self {
            header,
            data
        }
    }
}

impl Serialize for AResourceRecord {
    type Error = EncodingError;

    fn serialize(&self) -> Result<Vec<u8>, Self::Error> {
        let mut bytes = Vec::from(self.header.serialize()?);
        bytes.extend_from_slice(&self.data.octets());

        Ok(bytes)
    }
}

impl Deserialize for AResourceRecord {
    type Error = EncodingError;

    fn deserialize(bytes: &[u8], offset: usize) -> Result<(Self, usize), Self::Error>
    where
        Self: Sized
    {
        let (header, mut read_bytes) = ResourceRecordHeader::deserialize(bytes, offset)?;
        let (ipv4, new_read_bytes) = read_ipv4(bytes, offset + read_bytes)?;
        read_bytes += new_read_bytes;

        Ok((Self {
            header,
            data: ipv4
        }, read_bytes))
    }
}

impl ResourceRecord for AResourceRecord {}
