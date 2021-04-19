use binread::io::{Read, Seek};
use binread::{derive_binread, BinRead, BinResult, ReadOptions};

use choadra_nbt::tag::tag::NbtTag;
use choadra_nbt::tag::ty::NbtType;

use crate::aliases::Byte;
use crate::aliases::Float;
use crate::aliases::{Boolean, Int};
use crate::chat::parse_chat;
use crate::direction::Direction;
use crate::item::Slot;
use crate::particle::Particle;
use crate::position::Position;
use crate::string::parse_string;
use crate::uuid::parse_uuid;
use crate::uuid::UUID;
use crate::varint::parse_varint;

pub struct EntityMetadataEntry {
    pub index: u8,
    pub data: TrackedData,
}

impl EntityMetadataEntry {
    pub fn parse_vec<R: Read + Seek>(
        reader: &mut R,
        options: &ReadOptions,
        _args: (),
    ) -> BinResult<Vec<Self>> {
        let mut result = Vec::new();
        let mut byte_holder = [0; 1];
        loop {
            reader.read_exact(&mut byte_holder)?;
            let index = byte_holder[0];
            if index == 0xFF {
                break;
            }

            let data_type = parse_varint(reader, options, ())?;
            let data = TrackedData::read_options(reader, options, (data_type,))?;
            result.push(EntityMetadataEntry { index, data })
        }

        Ok(result)
    }
}

#[derive_binread]
#[br(import(data_type: i32))]
pub enum TrackedData {
    #[br(pre_assert(data_type == 0))]
    Byte(Byte),
    #[br(pre_assert(data_type == 1))]
    VarInt(#[br(parse_with = parse_varint)] Int),
    #[br(pre_assert(data_type == 2))]
    Float(Float),
    #[br(pre_assert(data_type == 3))]
    String(#[br(parse_with = parse_string, args(None))] String),
    #[br(pre_assert(data_type == 4))]
    Chat(#[br(parse_with = parse_chat)] String),
    #[br(pre_assert(data_type == 5))]
    OptionChat(
        #[br(temp, map = |x: u8| x != 0)] Boolean,
        #[br(parse_with = parse_chat, map = Some, if(self_0))] Option<String>,
    ),
    #[br(pre_assert(data_type == 6))]
    Slot(Slot),
    #[br(pre_assert(data_type == 7))]
    Boolean(#[br(map = |x: u8| x != 0)] Boolean),
    #[br(pre_assert(data_type == 8))]
    Rotation { x: Float, y: Float, z: Float },
    #[br(pre_assert(data_type == 9))]
    Position(Position),
    #[br(pre_assert(data_type == 10))]
    OptionPosition(
        #[br(temp, map = |x: u8| x != 0)] Boolean,
        #[br(if(self_0))] Option<Position>,
    ),
    #[br(pre_assert(data_type == 11))]
    Direction(Direction),
    #[br(pre_assert(data_type == 12))]
    OptionUUID(
        #[br(temp, map = |x: u8| x != 0)] Boolean,
        #[br(parse_with = parse_uuid, map = Some, if(self_0))] Option<UUID>,
    ),
    #[br(pre_assert(data_type == 13))]
    BlockId(#[br(parse_with = parse_varint)] Int),
    #[br(pre_assert(data_type == 14))]
    NBT(#[br(temp)] NbtType, #[br(args(self_0))] NbtTag),
    #[br(pre_assert(data_type == 15))]
    Particle(
        #[br(temp, parse_with = parse_varint)] Int,
        #[br(args(self_0))] Particle,
    ),
    #[br(pre_assert(data_type == 16))]
    VillagerData {
        #[br(parse_with = parse_varint)]
        villager_type: Int,
        #[br(parse_with = parse_varint)]
        villager_profession: Int,
        #[br(parse_with = parse_varint)]
        villager_level: Int,
    },
    #[br(pre_assert(data_type == 17))]
    OptionVarInt(
        #[br(temp, parse_with = parse_varint)] Int,
        #[br(calc = if self_0 == 0 { None } else { Some(self_0 -1) })] Option<Int>,
    ),
    #[br(pre_assert(data_type == 18))]
    Pose(Pose),
}

#[derive_binread]
#[br(repr = u8)]
#[derive(Copy, Clone, Debug, Ord, PartialOrd, Eq, PartialEq)]
pub enum Pose {
    Standing = 0,
    FallFlying = 1,
    Sleeping = 2,
    Swimming = 3,
    SpinAttack = 4,
    Sneaking = 5,
    Dying = 6,
}
