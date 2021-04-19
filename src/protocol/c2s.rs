use std::cell::RefCell;
use std::io::Write;
use std::rc::Rc;

use flate2::write::ZlibEncoder;
use flate2::Compression;

use crate::protocol::datatype::aliases::Int;
use crate::protocol::datatype::varint::VarInt;
use crate::protocol::datatype::writeable::Writeable;

#[derive(Debug)]
pub struct C2SPacket<T> {
    pub id: Int,
    pub inner: T,
}

impl<T: IdentityPacket> C2SPacket<T> {
    pub fn new(inner: T) -> Self {
        C2SPacket { id: T::ID, inner }
    }
}

pub trait IdentityPacket {
    const ID: Int;
}

#[derive(Debug, Clone)]
pub struct PacketWriteState(Rc<PacketWriteStateInner>);

#[derive(Debug)]
struct PacketWriteStateInner {
    pub compression_threshold: Option<Int>,
    zlib_encoder: RefCell<ZlibEncoder<OptionalWrite<Vec<u8>>>>,
}

impl PacketWriteState {
    pub fn compression_threshold(&self) -> Option<Int> {
        self.0.compression_threshold
    }

    pub fn set_compression_threshold(&mut self, new_value: Option<Int>) {
        let x = Rc::get_mut(&mut self.0).unwrap();
        x.compression_threshold = new_value;
    }
}

impl Default for PacketWriteState {
    fn default() -> Self {
        PacketWriteState(Rc::new(PacketWriteStateInner {
            compression_threshold: None,
            zlib_encoder: RefCell::new(ZlibEncoder::new(
                OptionalWrite { inner: None },
                Compression::default(),
            )),
        }))
    }
}

#[derive(Debug)]
struct OptionalWrite<W: Write> {
    inner: Option<W>,
}

impl<W: Write> Write for OptionalWrite<W> {
    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        match &mut self.inner {
            None => Ok(buf.len()),
            Some(w) => w.write(buf),
        }
    }

    fn flush(&mut self) -> std::io::Result<()> {
        match &mut self.inner {
            None => Ok(()),
            Some(w) => w.flush(),
        }
    }
}

impl<T> Writeable for C2SPacket<T>
where
    T: Writeable<Args = ()>,
{
    type Args = PacketWriteState;

    fn write_to<W: Write>(&self, write: &mut W, args: Self::Args) -> std::io::Result<()> {
        let mut uncompressed_content = Vec::new();
        VarInt(self.id).write_to(&mut uncompressed_content, ())?;
        &self.inner.write_to(&mut uncompressed_content, ())?;

        let uncompressed_length = uncompressed_content.len();
        if let Some(threshold) = args.compression_threshold() {
            // Compressed packet format
            let (packet_length, data_length, data) = if uncompressed_length >= threshold as usize {
                // Take the encoder from the args, should never fail
                let mut encoder = args.0.zlib_encoder.borrow_mut();
                // Swap the vec into the zlib ctx, keep the allocated sink box for restoring later
                let sink_writer = encoder.reset(OptionalWrite {
                    inner: Some(Vec::new()),
                })?;
                // Write all content to the vec
                encoder.write_all(&uncompressed_content)?;
                // Swap back to sink_writer, which flushes out to vec
                let content = encoder.reset(sink_writer)?.inner.unwrap();

                let data_length_bytes = VarInt(uncompressed_length as Int).bytes();
                (
                    data_length_bytes + content.len() as u32,
                    uncompressed_length as Int,
                    content,
                )
            } else {
                (
                    uncompressed_length as u32 + VarInt(0).bytes(),
                    0,
                    uncompressed_content,
                )
            };

            VarInt(packet_length as Int).write_to(write, ())?;
            VarInt(data_length).write_to(write, ())?;
            write.write_all(&data)?;
        } else {
            // Uncompressed packet format
            VarInt(uncompressed_length as Int).write_to(write, ())?;
            write.write_all(&uncompressed_content)?;
        }

        write.flush()?;

        Ok(())
    }
}
