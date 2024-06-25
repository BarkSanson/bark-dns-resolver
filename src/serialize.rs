use std::net::Ipv4Addr;

pub(crate) enum EncodingError {
    BufferOverflow
}

pub trait Serialize {
    type Error;

    fn serialize(&self) -> Result<Vec<u8>, Self::Error>;
}

pub trait Deserialize {
    type Error;
    fn deserialize(bytes: &[u8], offset: usize)
        -> Result<(usize, Self), Self::Error> where Self: Sized;
}

pub(crate) fn read_ipv4(bytes: &[u8], offset: usize) -> Result<(Ipv4Addr, usize), EncodingError> {
    if offset + 3 > bytes.len() {
        return Err(EncodingError::BufferOverflow)
    }

    Ok((Ipv4Addr::from(bytes[offset..offset + 4]), offset + 4))
}

pub(crate) fn read_u16(bytes: &[u8], offset: usize) -> Result<(u16, usize), EncodingError> {
    if offset + 1 > bytes.len() {
        return Err(EncodingError::BufferOverflow)
    }

    Ok((u16::from_be_bytes([
        bytes[offset],
        bytes[offset + 1]]), offset + 2))
}

pub(crate) fn read_i32(bytes: &[u8], offset: usize) -> Result<(i32, usize), EncodingError> {
    if offset + 3 > bytes.len() {
        return Err(EncodingError::BufferOverflow)
    }

    Ok((i32::from_be_bytes([
        bytes[offset],
        bytes[offset + 1],
        bytes[offset + 2],
        bytes[offset + 3],
    ]), offset + 4))
}