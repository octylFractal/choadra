use std::io::BufWriter;
use std::net::TcpStream;

use crate::client::def::EncryptableStream;
use crate::client::{ChoadraClient, Handshaking, Login, Status};
use crate::error::ChoadraResult;
use crate::protocol::c2s::PacketWriteState;
use crate::protocol::handshake::c2s::{ConnectionState, Handshake};
use crate::protocol::s2c::PacketReadState;

impl ChoadraClient<Handshaking> {
    pub fn new(stream: TcpStream) -> Self {
        ChoadraClient {
            writer: EncryptableStream::Plain(BufWriter::new(
                stream.try_clone().expect("Failed to clone TcpStream"),
            )),
            reader: EncryptableStream::Plain(stream),
            packet_write_state: PacketWriteState::default(),
            packet_read_state: PacketReadState::default(),
            state: Handshaking,
        }
    }

    pub fn request_status(mut self) -> ChoadraResult<ChoadraClient<Status>> {
        let peer = self.writer.get_ref().get_ref().peer_addr()?;
        self.write_c2s_packet(Handshake::new_current_protocol(
            peer.ip(),
            peer.port(),
            ConnectionState::Status,
        ))?;

        Ok(self.into_other_variant(Status))
    }

    pub fn request_login(mut self) -> ChoadraResult<ChoadraClient<Login>> {
        let peer = self.writer.get_ref().get_ref().peer_addr()?;
        self.write_c2s_packet(Handshake::new_current_protocol(
            peer.ip(),
            peer.port(),
            ConnectionState::Login,
        ))?;

        Ok(self.into_other_variant(Login))
    }
}
