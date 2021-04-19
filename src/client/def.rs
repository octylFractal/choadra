use std::io::{BufReader, BufWriter};
use std::net::TcpStream;

use crate::error::ChoadraResult;
use crate::protocol::c2s::PacketWriteState;
use crate::protocol::datatype::writeable::Writeable;

#[derive(Debug)]
pub struct ChoadraClient<S> {
    pub(crate) writer: BufWriter<TcpStream>,
    pub(crate) reader: BufReader<TcpStream>,
    pub(crate) packet_write_state: PacketWriteState,
    pub(crate) state: S,
}

impl<S> ChoadraClient<S> {
    pub(crate) fn into_other_variant<N>(self, new_state: N) -> ChoadraClient<N> {
        ChoadraClient {
            writer: self.writer,
            reader: self.reader,
            packet_write_state: self.packet_write_state,
            state: new_state,
        }
    }

    pub(crate) fn write<T: Writeable<Args = PacketWriteState>>(
        &mut self,
        object: T,
    ) -> ChoadraResult<()> {
        let state = self.packet_write_state.clone();
        object.write_to(&mut self.writer, state)?;

        Ok(())
    }
}

#[derive(Debug)]
pub struct Handshaking;

#[derive(Debug)]
pub struct Status;

#[derive(Debug)]
pub struct Login;

#[derive(Debug)]
pub struct Play;
