use std::io::Write;

use crate::protocol::c2s::IdentityPacket;
use crate::protocol::datatype::aliases::{Byte, Int, Long};
use crate::protocol::datatype::position::Position;
use crate::protocol::datatype::varint::VarInt;
use crate::protocol::datatype::writeable::Writeable;

pub trait C2SPlayPacket: IdentityPacket + Writeable<Args = ()> {}

#[derive(Debug)]
pub struct ChatMessage {
    pub message: String,
}

impl C2SPlayPacket for ChatMessage {}

impl IdentityPacket for ChatMessage {
    const ID: Int = 0x03;
}

impl Writeable for ChatMessage {
    type Args = ();

    fn write_to<W: Write>(&self, write: &mut W, _: Self::Args) -> Result<(), std::io::Error> {
        self.message.write_to(write, (Some(256),))?;

        Ok(())
    }
}

#[derive(Debug)]
pub struct ClientStatus {
    pub action: ClientStatusAction,
}

#[derive(Debug, Copy, Clone)]
pub enum ClientStatusAction {
    Respawn = 0,
    RequestStats = 1,
}

impl C2SPlayPacket for ClientStatus {}

impl IdentityPacket for ClientStatus {
    const ID: Int = 0x04;
}

impl Writeable for ClientStatus {
    type Args = ();

    fn write_to<W: Write>(&self, write: &mut W, _: Self::Args) -> Result<(), std::io::Error> {
        VarInt(self.action as Int).write_to(write, ())?;

        Ok(())
    }
}

#[derive(Debug)]
pub struct KeepAlive {
    pub id: Long,
}

impl C2SPlayPacket for KeepAlive {}

impl IdentityPacket for KeepAlive {
    const ID: Int = 0x10;
}

impl Writeable for KeepAlive {
    type Args = ();

    fn write_to<W: Write>(&self, write: &mut W, _: Self::Args) -> Result<(), std::io::Error> {
        self.id.write_to(write, ())?;

        Ok(())
    }
}

#[derive(Debug)]
pub struct PlayerDigging {
    pub status: PlayerDiggingStatus,
    pub location: Position,
    pub face: PlayerDiggingFace,
}

impl C2SPlayPacket for PlayerDigging {}

impl IdentityPacket for PlayerDigging {
    const ID: Int = 0x1B;
}

impl Writeable for PlayerDigging {
    type Args = ();

    fn write_to<W: Write>(&self, write: &mut W, _: Self::Args) -> Result<(), std::io::Error> {
        VarInt(self.status as Int).write_to(write, ())?;
        self.location.write_to(write, ())?;
        (self.face as Byte).write_to(write, ())?;

        Ok(())
    }
}

#[derive(Debug, Copy, Clone)]
pub enum PlayerDiggingStatus {
    StartedDigging = 0,
    CancelledDigging = 1,
    FinishedDigging = 2,
    DropItemStack = 3,
    DropItem = 4,
    UpdateHeldItem = 5,
    SwapItem = 6,
}

#[derive(Debug, Copy, Clone)]
pub enum PlayerDiggingFace {
    Bottom = 0,
    Top = 1,
    North = 2,
    South = 3,
    West = 4,
    East = 5,
}
