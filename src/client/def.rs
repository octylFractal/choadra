use std::collections::VecDeque;
use std::fmt::{Debug, Formatter};
use std::io::{BufWriter, Write};
use std::net::TcpStream;

use binread::io::Read;
use binread::BinRead;

use crate::error::ChoadraResult;
use crate::protocol::aes::AesStream;
use crate::protocol::c2s::{C2SPacket, IdentityPacket, PacketWriteState};
use crate::protocol::datatype::aliases::Int;
use crate::protocol::datatype::uuid::UUID;
use crate::protocol::datatype::writeable::Writeable;
use crate::protocol::play::s2c::S2CPlayPacket;
use crate::protocol::s2c::{read_s2c_packet, PacketReadState};

pub struct ChoadraClient<S> {
    pub(crate) writer: EncryptableStream<BufWriter<TcpStream>>,
    pub(crate) reader: EncryptableStream<TcpStream>,
    pub(crate) packet_write_state: PacketWriteState,
    pub(crate) packet_read_state: PacketReadState,
    pub(crate) state: S,
}

impl<S: Debug> Debug for ChoadraClient<S> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "ChoadraClient[peer={},state={:?}]",
            self.writer
                .get_ref()
                .get_ref()
                .peer_addr()
                .map_or_else(|_| "Unknown".to_string(), |peer| peer.to_string()),
            self.state
        )
    }
}

impl<S> ChoadraClient<S> {
    pub fn state(&self) -> &S {
        &self.state
    }

    pub(crate) fn into_other_variant<N>(self, new_state: N) -> ChoadraClient<N> {
        ChoadraClient {
            writer: self.writer,
            reader: self.reader,
            packet_write_state: self.packet_write_state,
            packet_read_state: self.packet_read_state,
            state: new_state,
        }
    }

    pub(crate) fn write_c2s_packet<T: IdentityPacket + Writeable<Args = ()>>(
        &mut self,
        object: T,
    ) -> ChoadraResult<()> {
        let state = self.packet_write_state.clone();
        C2SPacket::new(object).write_to(&mut self.writer, state)?;

        Ok(())
    }

    pub(crate) fn read_s2c_packet<T: BinRead<Args = (Int,)>>(&mut self) -> ChoadraResult<T> {
        let state = self.packet_read_state.clone();
        read_s2c_packet(&mut self.reader, state)
    }
}

pub(crate) enum EncryptableStream<S> {
    Plain(S),
    Encrypted(AesStream<S>),
}

impl<S> EncryptableStream<S> {
    pub fn get_ref(&self) -> &S {
        match self {
            EncryptableStream::Plain(s) => s,
            EncryptableStream::Encrypted(s) => s.get_ref(),
        }
    }

    pub fn into_inner(self) -> S {
        match self {
            EncryptableStream::Plain(s) => s,
            EncryptableStream::Encrypted(s) => s.into_inner(),
        }
    }
}

impl<R: Read> Read for EncryptableStream<R> {
    fn read(&mut self, buf: &mut [u8]) -> std::io::Result<usize> {
        match self {
            EncryptableStream::Plain(s) => s.read(buf),
            EncryptableStream::Encrypted(s) => s.read(buf),
        }
    }
}

impl<W: Write> Write for EncryptableStream<W> {
    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        match self {
            EncryptableStream::Plain(s) => s.write(buf),
            EncryptableStream::Encrypted(s) => s.write(buf),
        }
    }

    fn flush(&mut self) -> std::io::Result<()> {
        match self {
            EncryptableStream::Plain(s) => s.flush(),
            EncryptableStream::Encrypted(s) => s.flush(),
        }
    }
}

#[derive(Debug)]
pub struct Handshaking;

#[derive(Debug)]
pub struct Status;

#[derive(Debug)]
pub struct Login;

#[derive(Debug)]
pub struct Play {
    pub username: String,
    pub uuid: UUID,
    /// True if we know the server is in the Play state too
    pub(crate) really_playing: bool,
    pub(crate) packet_queue: VecDeque<S2CPlayPacket>,
}

#[derive(Debug)]
pub struct Credentials {
    pub token: String,
    pub profile: String,
}
