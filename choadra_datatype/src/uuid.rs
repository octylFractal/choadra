use binread::io::{Read, Seek};
use binread::{BinReaderExt, BinResult, ReadOptions};

pub type UUID = uuid::Uuid;

pub fn parse_uuid<R: Read + Seek>(
    reader: &mut R,
    options: &ReadOptions,
    _args: (),
) -> BinResult<UUID> {
    let value = reader.read_type::<u128>(options.endian)?;
    Ok(UUID::from_u128(value))
}
