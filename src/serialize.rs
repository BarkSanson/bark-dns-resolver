pub trait Serialize {
    fn as_bytes(&self) -> Vec<u8>;
}

pub trait Deserialize {
    fn from_bytes(bytes: &[u8]) -> Self;
}