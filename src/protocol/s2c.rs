use std::io::Cursor;

use binread::io::Read;
use binread::{BinRead, Endian, ReadOptions};
use flate2::read::ZlibDecoder;

use crate::error::{ChoadraError, ChoadraResult};
use crate::protocol::datatype::aliases::Int;
use crate::protocol::datatype::varint::parse_varint;

pub fn read_s2c_packet<T: BinRead<Args = (Int,)>, R: Read>(
    reader: &mut R,
    compressed_threshold: Option<i32>,
) -> ChoadraResult<T> {
    let mut options = ReadOptions::default();
    options.endian = Endian::Big;
    let length = parse_varint(reader, &options, ())?;
    let uncompressed_length = match compressed_threshold {
        Some(max) => match parse_varint(reader, &options, ())? {
            0 => None,
            x => {
                if x > max {
                    return Err(ChoadraError::ServerError {
                        msg: "Compression threshold exceeded".to_string(),
                    });
                }
                Some(x)
            }
        },
        None => None,
    };

    let mut uncompressed = vec![0u8; uncompressed_length.unwrap_or(length) as usize];
    if uncompressed_length.is_some() {
        // Decompress into the vec
        ZlibDecoder::new(reader.take(length as u64)).read_exact(&mut uncompressed)?;
    } else {
        // Just read into the vec
        reader.read_exact(&mut uncompressed)?;
    }

    let mut inner_reader = Cursor::new(&uncompressed);
    let packet_id = parse_varint(&mut inner_reader, &options, ())?;

    T::read_options(&mut inner_reader, &options, (packet_id,))
        .map_err(|e| ChoadraError::BinreadError(e))
}
