use std::io::Cursor;

use binread::io::Read;
use binread::{derive_binread, BinRead, BinResult, Endian, ReadOptions};
use flate2::read::ZlibDecoder;

use crate::error::{ChoadraError, ChoadraResult};
use crate::protocol::datatype::aliases::Int;
use crate::protocol::datatype::varint::parse_varint;

#[derive_binread]
#[br(import(id: Int))]
#[derive(Debug)]
pub enum S2CPacket<T: BinRead<Args = (Int,)>> {
    /// A known packet.
    Inner(#[br(args(id))] T),
    /// An unhandled variant, the data for it is left inside.
    Unhandled {
        #[br(calc = id)]
        id: Int,
        content: Vec<u8>,
    },
}

impl<T: BinRead<Args = (Int,)>> S2CPacket<T> {
    pub fn read<R: Read>(reader: &mut R, compressed_threshold: Option<i32>) -> ChoadraResult<T> {
        S2CPacket::<T>::read_raw(reader, compressed_threshold)?.into_choadra_result()
    }

    pub fn read_raw<R: Read>(reader: &mut R, compressed_threshold: Option<i32>) -> BinResult<Self> {
        let mut options = ReadOptions::default();
        options.endian = Endian::Big;
        let length = parse_varint(reader, &options, ())?;
        let uncompressed_length = match compressed_threshold {
            Some(max) => match parse_varint(reader, &options, ())? {
                0 => None,
                x => {
                    if x > max {
                        return Err(binread::Error::Custom {
                            pos: 0,
                            err: Box::new("Compression max exceeded"),
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

        Self::read_options(&mut inner_reader, &options, (packet_id,))
    }

    pub fn into_choadra_result(self) -> ChoadraResult<T> {
        match self {
            S2CPacket::Inner(t) => Ok(t),
            S2CPacket::Unhandled { id, .. } => Err(ChoadraError::ServerError {
                msg: format!("Unhandled packet: 0x{:X}", id),
            }),
        }
    }
}
