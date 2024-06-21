pub(crate) trait FromWithBytes {
    fn from_with_bytes(bytes: &[u8]) -> (Self, usize);
}