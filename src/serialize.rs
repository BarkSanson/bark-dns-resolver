use std::net::Ipv4Addr;

pub enum DeserializationError {
    BufferOverflow,
    InvalidData(String),
    Io(std::io::Error)
}

pub(crate) trait Serialize {
    fn serialize(&self) -> Vec<u8>;
}

pub(crate) trait Deserialize {
    fn deserialize(bytes: &[u8], offset: usize)
        -> Result<(usize, Self), DeserializationError> where Self: Sized;
}

pub(crate) fn read_ipv4(bytes: &[u8], offset: usize)
    -> Result<(usize, Ipv4Addr), DeserializationError> {
    if offset + 3 > bytes.len() {
        return Err(DeserializationError::BufferOverflow)
    }

    let bytes: [u8; 4] =
        bytes[offset..offset + 4].try_into().expect("Couldn't convert bytes into Ipv4");

    Ok((4, Ipv4Addr::from(bytes)))
}

pub(crate) fn read_u16(bytes: &[u8], offset: usize)
    -> Result<(usize, u16), DeserializationError> {
    if offset + 1 > bytes.len() {
        return Err(DeserializationError::BufferOverflow)
    }

    Ok((2, u16::from_be_bytes([
        bytes[offset],
        bytes[offset + 1]])))
}

pub(crate) fn read_i32(bytes: &[u8], offset: usize)
    -> Result<(usize, i32), DeserializationError> {
    if offset + 3 > bytes.len() {
        return Err(DeserializationError::BufferOverflow)
    }

    Ok((4, i32::from_be_bytes([
        bytes[offset],
        bytes[offset + 1],
        bytes[offset + 2],
        bytes[offset + 3],
    ])))
}