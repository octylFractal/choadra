use std::fmt::{Debug, Formatter};

use binread::derive_binread;

use crate::protocol::datatype::aliases::{Byte, Double, Int, Long, Short};
use crate::protocol::datatype::angle::Angle;
use crate::protocol::datatype::chat::parse_chat;
use crate::protocol::datatype::uuid::{parse_uuid, UUID};
use crate::protocol::datatype::varint::parse_varint;
use crate::protocol::util::parse_until_eof;

#[derive_binread]
#[br(import(id: Int))]
#[derive(Debug)]
pub enum S2CPlayPacket {
    #[br(pre_assert(id == 0x00))]
    SpawnEntity(SpawnEntity),
    #[br(pre_assert(id == 0x0E))]
    ChatMessage(ChatMessage),
    #[br(pre_assert(id == 0x19))]
    Disconnect(Disconnect),
    #[br(pre_assert(id == 0x1F))]
    KeepAlive(KeepAlive),
    Unknown(#[br(args(id))] Unknown),
}

#[derive_binread]
#[derive(Debug)]
pub struct SpawnEntity {
    #[br(parse_with = parse_varint)]
    pub entity_id: Int,
    #[br(parse_with = parse_uuid)]
    pub uuid: UUID,
    #[br(parse_with = parse_varint)]
    pub entity_type: Int,
    pub x: Double,
    pub y: Double,
    pub z: Double,
    pub pitch: Angle,
    pub yaw: Angle,
    pub data: Int,
    pub velocity_x: Short,
    pub velocity_y: Short,
    pub velocity_z: Short,
}

#[derive_binread]
#[derive(Debug)]
pub struct ChatMessage {
    #[br(parse_with = parse_chat)]
    pub message: String,
    pub position: ChatPosition,
    #[br(parse_with = parse_uuid)]
    pub sender: UUID,
}

#[derive_binread]
#[br(repr(Byte))]
#[derive(Debug)]
pub enum ChatPosition {
    ChatBox = 0,
    SystemMessage = 1,
    GameInfo = 2,
}

#[derive_binread]
#[derive(Debug)]
pub struct Disconnect {
    #[br(parse_with = parse_chat)]
    pub reason: String,
}

#[derive_binread]
#[derive(Debug)]
pub struct KeepAlive {
    pub id: Long,
}

#[derive_binread]
#[br(import(id: Int))]
pub struct Unknown {
    #[br(calc = id)]
    pub id: Int,
    #[br(parse_with = parse_until_eof)]
    pub data: Vec<u8>,
}

impl Debug for Unknown {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "Unknown[id=")?;
        Debug::fmt(&self.id, f)?;
        write!(f, ",data=[")?;
        Debug::fmt(&self.data.len(), f)?;
        write!(f, " bytes]]")
    }
}
