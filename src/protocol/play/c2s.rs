use crate::protocol::c2s::IdentityPacket;
use crate::protocol::datatype::writeable::Writeable;

pub trait C2SPlayPacket: IdentityPacket + Writeable<Args = ()> {}
