use crate::protocol::c2s::IdentityPacket;
use crate::protocol::datatype::aliases::{Int, Long};
use crate::protocol::datatype::writeable::Writeable;
use std::io::Write;

#[derive(Debug)]
pub struct Request;

impl IdentityPacket for Request {
    const ID: Int = 0x00;
}

impl Writeable for Request {
    type Args = ();

    fn write_to<W: Write>(&self, _: &mut W, _: Self::Args) -> std::io::Result<()> {
        Ok(())
    }
}

#[derive(Debug)]
pub struct Ping(pub Long);

impl IdentityPacket for Ping {
    const ID: Int = 0x01;
}

impl Writeable for Ping {
    type Args = ();

    fn write_to<W: Write>(&self, write: &mut W, _: Self::Args) -> std::io::Result<()> {
        self.0.write_to(write, ())?;
        Ok(())
    }
}
