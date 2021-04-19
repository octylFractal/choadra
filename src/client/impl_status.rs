use std::time::{Duration, Instant};

use crate::client::{ChoadraClient, Status};
use crate::error::{ChoadraError, ChoadraResult};
use crate::protocol::c2s::C2SPacket;
use crate::protocol::datatype::aliases::Long;
use crate::protocol::s2c::read_s2c_packet;
use crate::protocol::status::c2s::{Ping, Request};
use crate::protocol::status::s2c::{Pong, Response, S2CStatusPacket};

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
