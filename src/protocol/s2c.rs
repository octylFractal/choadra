use std::cell::RefCell;
use std::io::Cursor;
use std::rc::Rc;

use binread::io::Read;
use binread::{BinRead, Endian, ReadOptions};
use flate2::read::ZlibDecoder;

use crate::error::{ChoadraError, ChoadraResult};
use crate::protocol::datatype::aliases::Int;
use crate::protocol::datatype::varint::{parse_varint, VarInt};

pub struct PacketReadState(Rc<PacketReadStateInner>);

struct PacketReadStateInner {
    pub compression_threshold: Option<Int>,
    zlib_decoder: RefCell<ZlibDecoder<OptionalRead<Cursor<Vec<u8>>>>>,
}

impl PacketReadState {
    pub fn compression_threshold(&self) -> Option<Int> {
        self.0.compression_threshold
    }

    pub fn set_compression_threshold(&mut self, new_value: Option<Int>) {
        let x = Rc::get_mut(&mut self.0).unwrap();
        x.compression_threshold = new_value;
    }
}

impl Default for PacketReadState {
    fn default() -> Self {
        PacketReadState(Rc::new(PacketReadStateInner {
            compression_threshold: None,
            zlib_decoder: RefCell::new(ZlibDecoder::new(OptionalRead { inner: None })),
        }))
    }
}

impl Clone for PacketReadState {
    fn clone(&self) -> Self {
        PacketReadState(Rc::clone(&self.0))
    }
}

#[derive(Debug)]
struct OptionalRead<R: Read> {
    inner: Option<R>,
}

impl<R: Read> Read for OptionalRead<R> {
    fn read(&mut self, buf: &mut [u8]) -> std::io::Result<usize> {
        match &mut self.inner {
            Some(r) => r.read(buf),
            None => Ok(0),
        }
    }
}

pub fn read_s2c_packet<T: BinRead<Args = (Int,)>, R: Read>(
    mut reader: R,
    args: PacketReadState,
) -> ChoadraResult<T> {
    let mut options = ReadOptions::default();
    options.endian = Endian::Big;
    let length = parse_varint(&mut reader, &options, ())?;
    let (compressed_length, uncompressed_length) = match args.compression_threshold() {
        Some(min) => match parse_varint(&mut reader, &options, ())? {
            0 => (None, length - 1),
            x => {
                if x < min {
                    return Err(ChoadraError::ServerError {
                        msg: format!(
                            "Compression threshold {} not met, got a compressed packet of size {}",
                            min, x
                        ),
                    });
                }
                (Some(length - VarInt(x).bytes() as Int), x)
            }
        },
        None => (None, length),
    };

    let mut uncompressed = vec![0u8; uncompressed_length as usize];
    if let Some(compressed_length) = compressed_length {
        // Decompress into the vec
        let compressed = {
            let mut compressed = vec![0u8; compressed_length as usize];
            reader.read_exact(&mut compressed)?;
            compressed
        };
        let mut decoder = args.0.zlib_decoder.borrow_mut();
        let empty_read = decoder.reset(OptionalRead {
            inner: Some(Cursor::new(compressed)),
        });
        decoder.read_exact(&mut uncompressed)?;
        decoder.reset(empty_read);
    } else {
        // Just read into the vec
        reader.read_exact(&mut uncompressed)?;
    }

    let mut inner_reader = Cursor::new(&uncompressed);
    let packet_id = parse_varint(&mut inner_reader, &options, ())?;

    T::read_options(&mut inner_reader, &options, (packet_id,))
        .map_err(|e| ChoadraError::BinreadError(e))
}
