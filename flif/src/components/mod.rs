pub(crate) mod header;
pub(crate) mod metadata;
pub(crate) mod transformations;

pub use self::header::{BytesPerChannel, Header, SecondHeader};
pub use self::metadata::{Metadata, ChunkType};
pub use self::transformations::Transformation;
