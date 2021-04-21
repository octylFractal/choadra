use binread::derive_binread;

use crate::protocol::datatype::aliases::Int;
use crate::protocol::datatype::chat::parse_chat;

#[derive_binread]
#[br(import(id: Int))]
#[derive(Debug)]
pub enum S2CPlayPacket {
    #[br(pre_assert(id == 0x00))]
    Disconnect(Disconnect),
}

#[derive_binread]
#[derive(Debug)]
pub struct Disconnect {
    #[br(parse_with = parse_chat)]
    pub reason: String,
}
