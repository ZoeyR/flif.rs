mod decoder;
mod description;
mod numbers;
pub mod error;

pub use description::{Flif, Header};
pub use decoder::Decoder;

mod private {
    pub trait Sealed {}
}

#[cfg(test)]
mod tests {
    #[test]
    fn test_varint_read() {
        use util::ReadVarintExt;

        let buf = [0x82, 0x5F, 0x82, 0x2F];

        let first = buf.as_ref().read_varint().unwrap();
        let second = buf[2..].as_ref().read_varint().unwrap();
        assert_eq!(first, 351);
        assert_eq!(second, 303);
    }

    #[test]
    fn test_varint_overflow_read() {
        use util::ReadVarintExt;

        let buf = [0xFF, 0xFF, 0xFF, 0xFF, 0x7F];
        let num = buf.as_ref().read_varint().unwrap();
        assert_eq!(num, 351);
    }
}
