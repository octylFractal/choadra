use binread::io::{Read, Seek};
use binread::{derive_binread, BinRead, BinReaderExt, BinResult, ReadOptions};

use choadra_nbt::tag::tag::NbtTag;
use choadra_nbt::tag::ty::NbtType;

use crate::aliases::{Boolean, Byte, Int};
use crate::varint::parse_varint;

#[derive_binread]
#[derive(Debug, PartialEq)]
pub struct Slot {
    #[br(temp, map = |x: u8| x != 0)]
    present: Boolean,
    #[br(if(present))]
    item: Option<ItemStack>,
}

#[derive_binread]
#[derive(Debug, PartialEq)]
pub struct ItemStack {
    #[br(parse_with = parse_varint)]
    id: Int,
    count: Byte,
    #[br(parse_with = parse_optional_nbt)]
    nbt: Option<NbtTag>,
}

fn parse_optional_nbt<R: Read + Seek>(
    reader: &mut R,
    options: &ReadOptions,
    _args: (),
) -> BinResult<Option<NbtTag>> {
    let ty = reader.read_type::<NbtType>(options.endian)?;
    Option::<NbtTag>::read_options(
        reader,
        &{
            let mut o = ReadOptions::default();
            o.endian = options.endian;
            o
        },
        (ty,),
    )
}
