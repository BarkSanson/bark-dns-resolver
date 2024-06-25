
pub(crate) enum EncodingError {
}

pub trait Serialize {
    type Error;

    fn serialize(&self) -> Result<Vec<u8>, Self::Error>;
}

pub trait Deserialize {
    type Error;
    fn deserialize(bytes: &[u8]) -> Result<Self, Self::Error> where Self: Sized;
}