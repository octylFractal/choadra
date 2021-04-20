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

#[derive(Clone)]
pub struct PacketWriteState(Rc<PacketWriteStateInner>);

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
        let uncompressed_content = self.write_uncompressed_packet()?;

        let final_content =
            <C2SPacket<T>>::compress_packet_if_needed(args.clone(), uncompressed_content)?;

        write.write_all(&final_content)?;
        write.flush()?;

        Ok(())
    }
}

impl<T> C2SPacket<T>
where
    T: Writeable<Args = ()>,
{
    fn write_uncompressed_packet(&self) -> std::io::Result<Vec<u8>> {
        let mut uncompressed_content = Vec::new();
        VarInt(self.id).write_to(&mut uncompressed_content, ())?;
        &self.inner.write_to(&mut uncompressed_content, ())?;
        Ok(uncompressed_content)
    }
}

impl<T> C2SPacket<T> {
    fn compress_packet_if_needed(
        args: PacketWriteState,
        uncompressed_content: Vec<u8>,
    ) -> std::io::Result<Vec<u8>> {
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
                    data_length_bytes + content.len(),
                    uncompressed_length as Int,
                    content,
                )
            } else {
                (
                    uncompressed_length + VarInt(0).bytes(),
                    0,
                    uncompressed_content,
                )
            };

            let mut result = Vec::with_capacity(
                VarInt(packet_length as Int).bytes() + VarInt(data_length).bytes() + data.len(),
            );
            VarInt(packet_length as Int).write_to(&mut result, ())?;
            VarInt(data_length).write_to(&mut result, ())?;
            result.write_all(&data)?;
            Ok(result)
        } else {
            // Uncompressed packet format
            let mut result = Vec::with_capacity(
                VarInt(uncompressed_length as Int).bytes() + uncompressed_content.len(),
            );
            VarInt(uncompressed_length as Int).write_to(&mut result, ())?;
            result.write_all(&uncompressed_content)?;
            Ok(result)
        }
    }
}
