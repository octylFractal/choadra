use std::collections::HashMap;
use std::fmt::Debug;

use binread::io::{Read, Seek};
use binread::{BinRead, BinReaderExt, BinResult, ReadOptions};

use crate::tag::string::parse_modified_utf8;
use crate::tag::ty::NbtType;

pub fn parse_compound_tag<T, R>(
    reader: &mut R,
    options: &ReadOptions,
    _args: (),
) -> BinResult<HashMap<String, T>>
where
    T: Debug + BinRead<Args = (NbtType,)>,
    R: Read + Seek,
{
    let mut result = HashMap::new();

    loop {
        match read_single_entry(reader, options)? {
            Some((name, data)) => {
                result.insert(name, data);
            }
            None => break,
        }
    }

    Ok(result)
}

/// Reads a single entry, returning None if it was an End tag
pub(crate) fn read_single_entry<T, R>(
    reader: &mut R,
    options: &ReadOptions,
) -> BinResult<Option<(String, T)>>
where
    T: Debug + BinRead<Args = (NbtType,)>,
    R: Read + Seek,
{
    let ty = reader.read_type::<NbtType>(options.endian)?;
    eprintln!("{:#?}", ty);
    if ty == NbtType::End {
        return Ok(None);
    }

    let name_length = reader.read_type::<u16>(options.endian)?;
    eprintln!("{:#?}", name_length);
    let name = parse_modified_utf8(
        reader,
        &{
            let mut o = ReadOptions::default();
            o.endian = options.endian;
            o.count = Some(name_length as usize);
            o
        },
        (),
    )?;
    eprintln!("{:#?}", name);
    let data = T::read_options(
        reader,
        &{
            let mut o = ReadOptions::default();
            o.endian = options.endian;
            o
        },
        (ty,),
    )?;
    eprintln!("{:#?}", data);
    Ok(Some((name, data)))
}
