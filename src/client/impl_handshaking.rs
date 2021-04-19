use std::io::{BufReader, BufWriter};
use std::net::TcpStream;

use crate::client::{ChoadraClient, Handshaking, Status};
use crate::error::ChoadraResult;
use crate::protocol::c2s::{C2SPacket, PacketWriteState};
use crate::protocol::handshake::c2s::{ConnectionState, Handshake};

impl ChoadraClient<Handshaking> {
    pub fn new(stream: TcpStream) -> Self {
        ChoadraClient {
            writer: BufWriter::new(stream.try_clone().expect("Failed to clone TcpStream")),
            reader: BufReader::new(stream),
            packet_write_state: PacketWriteState::default(),
            state: Handshaking,
        }
    }

    pub fn request_status(mut self) -> ChoadraResult<ChoadraClient<Status>> {
        let peer = self.writer.get_ref().peer_addr()?;
        let handshake =
            Handshake::new_current_protocol(peer.ip(), peer.port(), ConnectionState::Status);
        self.write(C2SPacket::new(handshake))?;

        Ok(self.into_other_variant(Status))
    }
}
