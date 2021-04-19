use std::io::Write;

use crate::protocol::datatype::aliases::{Int, UnsignedShort};
use crate::protocol::datatype::varint::VarInt;
use crate::protocol::datatype::writeable::Writeable;
use crate::protocol::c2s::IdentityPacket;

const CURRENT_PROTOCOL_VERSION: i32 = 754;

#[derive(Debug)]
pub struct Handshake {
    pub protocol_version: Int,
    pub server_address: String,
    pub server_port: UnsignedShort,
    pub next_state: ConnectionState,
}

impl Handshake {
    pub fn new_current_protocol<S: ToString>(
        address: S,
        port: UnsignedShort,
        next_state: ConnectionState,
    ) -> Self {
        Handshake {
            protocol_version: CURRENT_PROTOCOL_VERSION,
            server_address: address.to_string(),
            server_port: port,
            next_state,
        }
    }
}

impl IdentityPacket for Handshake {
    const ID: Int = 0x00;
}

impl Writeable for Handshake {
    type Args = ();
    fn write_to<W: Write>(&self, write: &mut W, _: Self::Args) -> Result<(), std::io::Error> {
        VarInt(self.protocol_version).write_to(write, ())?;
        self.server_address.write_to(write, (Some(255),))?;
        self.server_port.write_to(write, ())?;
        (self.next_state as u8).write_to(write, ())?;

        Ok(())
    }
}

#[derive(Copy, Clone, Debug)]
pub enum ConnectionState {
    Status = 1,
    Login = 2,
}
