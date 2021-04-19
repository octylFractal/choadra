use binread::derive_binread;

#[derive_binread]
#[br(repr = u8)]
#[derive(Copy, Clone, Debug, Ord, PartialOrd, Eq, PartialEq)]
pub enum NbtType {
    End = 0,
    Byte = 1,
    Short = 2,
    Int = 3,
    Long = 4,
    Float = 5,
    Double = 6,
    ByteArray = 7,
    String = 8,
    List = 9,
    Compound = 10,
    IntArray = 11,
    LongArray = 12,
}

impl NbtType {
    pub fn discriminator(&self) -> u32 {
        *self as u32
    }
}
