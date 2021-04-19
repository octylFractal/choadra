use std::collections::HashMap;
use std::iter::FromIterator;

use binread::io::{Read, Seek};
use binread::{derive_binread, BinResult, Endian, ReadOptions};

use crate::tag::compound::{parse_compound_tag, read_single_entry};
use crate::tag::string::parse_modified_utf8;
use crate::tag::ty::NbtType;

/// Represents an NBT tag.
#[derive_binread]
#[br(import(ty: NbtType))]
#[derive(Debug, PartialEq)]
pub enum NbtTag {
    #[br(pre_assert(ty == NbtType::End))]
    End,
    #[br(pre_assert(ty == NbtType::Byte))]
    Byte(i8),
    #[br(pre_assert(ty == NbtType::Short))]
    Short(i16),
    #[br(pre_assert(ty == NbtType::Int))]
    Int(i32),
    #[br(pre_assert(ty == NbtType::Long))]
    Long(i64),
    #[br(pre_assert(ty == NbtType::Float))]
    Float(f32),
    #[br(pre_assert(ty == NbtType::Double))]
    Double(f64),
    #[br(pre_assert(ty == NbtType::ByteArray))]
    ByteArray(#[br(temp)] i32, #[br(count = self_0)] Vec<i8>),
    #[br(pre_assert(ty == NbtType::String))]
    String(
        #[br(temp)] i16,
        #[br(parse_with = parse_modified_utf8, count = self_0)] String,
    ),
    #[br(pre_assert(ty == NbtType::List))]
    List {
        ty: NbtType,
        #[br(temp, assert(ty != NbtType::End || length <= 0))]
        length: i32,
        #[br(count = length, args(ty))]
        tags: Vec<NbtTag>,
    },
    #[br(pre_assert(ty == NbtType::Compound))]
    Compound(#[br(parse_with = parse_compound_tag)] HashMap<String, NbtTag>),
    #[br(pre_assert(ty == NbtType::IntArray))]
    IntArray(Vec<i32>),
    #[br(pre_assert(ty == NbtType::LongArray))]
    LongArray(Vec<i64>),
}

impl NbtTag {
    /// Deserialize a Tag, does not do any special handling.
    /// Caller must decompress if needed.
    pub fn deserialize_from_root<R: Read + Seek>(reader: &mut R) -> BinResult<NbtTag> {
        let entry = read_single_entry(reader, &{
            let mut o = ReadOptions::default();
            o.endian = Endian::Big;
            o
        })?;
        match entry {
            Some((k, v)) => {
                let mut map = HashMap::with_capacity(1);
                map.insert(k, v);
                Ok(NbtTag::Compound(map))
            }
            None => Ok(NbtTag::Compound(HashMap::new())),
        }
    }

    pub fn new_compound_ref<'a, I: IntoIterator<Item = (&'a str, NbtTag)>>(iter: I) -> NbtTag {
        NbtTag::Compound(HashMap::from_iter(
            iter.into_iter().map(|(k, v)| (k.to_string(), v)),
        ))
    }

    pub fn new_compound<I: IntoIterator<Item = (String, NbtTag)>>(iter: I) -> NbtTag {
        NbtTag::Compound(HashMap::from_iter(iter))
    }
}

impl NbtTag {
    pub fn id(&self) -> NbtType {
        match self {
            NbtTag::End => NbtType::End,
            NbtTag::Byte(_) => NbtType::Byte,
            NbtTag::Short(_) => NbtType::Short,
            NbtTag::Int(_) => NbtType::Int,
            NbtTag::Long(_) => NbtType::Long,
            NbtTag::Float(_) => NbtType::Float,
            NbtTag::Double(_) => NbtType::Double,
            NbtTag::ByteArray(_) => NbtType::ByteArray,
            NbtTag::String(_) => NbtType::String,
            NbtTag::List { .. } => NbtType::List,
            NbtTag::Compound(_) => NbtType::Compound,
            NbtTag::IntArray(_) => NbtType::IntArray,
            NbtTag::LongArray(_) => NbtType::LongArray,
        }
    }
}
