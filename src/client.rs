use crate::error::{ChoadraError, ChoadraResult};
use crate::protocol::c2s::{C2SPacket, PacketWriteState};
use crate::protocol::datatype::aliases::Long;
use crate::protocol::datatype::writeable::Writeable;
use crate::protocol::handshake::c2s::{ConnectionState, Handshake};
use crate::protocol::status::c2s::{Ping, Request};
use crate::protocol::status::s2c::{Pong, S2CStatusPacket, Response};
use std::io::{BufReader, BufWriter};
use std::net::TcpStream;
use std::time::{Duration, Instant};
use crate::protocol::s2c::read_s2c_packet;

#[derive(Debug)]
pub struct ChoadraClient<S> {
    writer: BufWriter<TcpStream>,
    reader: BufReader<TcpStream>,
    packet_write_state: PacketWriteState,
    state: S,
}

impl<S> ChoadraClient<S> {
    fn into_other_variant<N>(self, new_state: N) -> ChoadraClient<N> {
        ChoadraClient {
            writer: self.writer,
            reader: self.reader,
            packet_write_state: self.packet_write_state,
            state: new_state,
        }
    }

    fn write<T: Writeable<Args = PacketWriteState>>(&mut self, object: T) -> ChoadraResult<()> {
        let state = self.packet_write_state.clone();
        object.write_to(&mut self.writer, state)?;

        Ok(())
    }
}

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

impl ChoadraClient<Status> {
    pub fn ping(&mut self) -> ChoadraResult<Duration> {
        let rng = rand::random::<Long>();
        let packet = C2SPacket::new(Ping(rng));
        let now = Instant::now();
        self.write(packet)?;
        let s2c_packet = read_s2c_packet(
            &mut self.reader,
            self.packet_write_state.compression_threshold(),
        )?;
        let elapsed = now.elapsed();
        let packet: Pong = match s2c_packet {
            S2CStatusPacket::Pong(p) => p,
            S2CStatusPacket::Response(_) => {
                return Err(ChoadraError::ServerError {
                    msg: "Got a Response instead of a Pong".to_string(),
                })
            }
        };

        if packet.0 != rng {
            return Err(ChoadraError::ServerError {
                msg: format!("Sent {} but got {}", rng, packet.0),
            });
        }

        Ok(elapsed)
    }

    pub fn status(&mut self) -> ChoadraResult<Response> {
        let packet = C2SPacket::new(Request);
        self.write(packet)?;
        let s2c_packet = read_s2c_packet(
            &mut self.reader,
            self.packet_write_state.compression_threshold(),
        )?;
        let response = match s2c_packet {
            S2CStatusPacket::Response(r) => r,
            S2CStatusPacket::Pong(_) => {
                return Err(ChoadraError::ServerError {
                    msg: "Got a Pong instead of a Response".to_string(),
                })
            }
        };

        Ok(response)
    }
}

#[derive(Debug)]
pub struct Handshaking;

#[derive(Debug)]
pub struct Status;
