use std::io::Write;

use binread::io::{Read, Seek, SeekFrom};
use binread::{BinResult, ReadOptions};

use choadra_nbt::modified_utf8::{decode_modified_utf8, encode_modified_utf8};

use crate::varint::{parse_varint, VarInt};
use crate::writeable::Writeable;

pub fn parse_string<R: Read + Seek>(
    reader: &mut R,
    options: &ReadOptions,
    (limit,): (Option<usize>,),
) -> BinResult<String> {
    let length = parse_varint(reader, options, ())? as usize;

    let full_limit = limit.unwrap_or(32767);
    if length > full_limit {
        let pos = reader.seek(SeekFrom::Current(0))?;
        return Err(binread::Error::AssertFail {
            pos,
            message: format!("More than {} bytes declared in length", full_limit),
        });
    }

    let mut content = vec![0u8; length as usize];
    reader.read_exact(&mut content)?;

    decode_modified_utf8(&content).or_else(|e| {
        let pos = reader.seek(SeekFrom::Current(0))?;
        Err(binread::Error::Custom {
            pos,
            err: Box::new(e),
        })
    })
}

impl Writeable for str {
    type Args = (Option<usize>,);
    fn write_to<W: Write>(&self, write: &mut W, (limit,): Self::Args) -> std::io::Result<()> {
        let content = {
            let mut dest = Vec::with_capacity(self.len());
            encode_modified_utf8(self, &mut dest)?;
            dest
        };
        let full_limit = limit.unwrap_or(32767);
        if full_limit > (i32::MAX as usize) {
            return Err(std::io::Error::new(
                std::io::ErrorKind::InvalidData,
                format!("Limit exceeds the size of an Int: {}", full_limit,),
            ));
        }
        if content.len() > full_limit {
            return Err(std::io::Error::new(
                std::io::ErrorKind::InvalidData,
                format!(
                    "String of length {} exceeds limit {}",
                    content.len(),
                    full_limit,
                ),
            ));
        }
        let length = content.len() as i32;
        VarInt(length).write_to(write, ())?;
        write.write_all(&content)?;

        Ok(())
    }
}
