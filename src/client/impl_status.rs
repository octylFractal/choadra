use std::time::{Duration, Instant};

use crate::client::{ChoadraClient, Status};
use crate::error::{ChoadraError, ChoadraResult};
use crate::protocol::datatype::aliases::Long;
use crate::protocol::status::c2s::{Ping, Request};
use crate::protocol::status::s2c::{Pong, Response, S2CStatusPacket};

impl ChoadraClient<Status> {
    pub fn ping(&mut self) -> ChoadraResult<Duration> {
        let rng = rand::random::<Long>();
        let packet = Ping(rng);
        let now = Instant::now();
        self.write_c2s_packet(packet)?;
        let s2c_packet = self.read_s2c_packet()?;
        let elapsed = now.elapsed();
        let packet: Pong = match s2c_packet {
            S2CStatusPacket::Pong(p) => p,
            _ => {
                return Err(ChoadraError::ServerError {
                    msg: format!("Got {:?} instead of a Pong", s2c_packet),
                });
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
        self.write_c2s_packet(Request)?;
        let s2c_packet = self.read_s2c_packet()?;
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
