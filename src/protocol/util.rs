use std::io::SeekFrom;

use binread::io::{Read, Seek};
use binread::{BinRead, BinResult, ReadOptions};

pub(crate) fn parse_until_eof<A, B, R>(
    reader: &mut R,
    options: &ReadOptions,
    args: A,
) -> BinResult<Vec<B>>
where
    A: Copy + 'static,
    B: BinRead<Args = A>,
    R: Read + Seek,
{
    let mut result = Vec::new();

    let old_pos = reader.stream_position()?;
    let stream_len = reader.seek(SeekFrom::End(0))?;

    if old_pos != stream_len {
        reader.seek(SeekFrom::Start(old_pos))?;
    } else {
        // We're already at EOF, don't need to fall into the `while`
        return Ok(result);
    }

    loop {
        result.push(B::read_options(reader, options, args)?);

        if reader.stream_position()? >= stream_len {
            break;
        }
    }
    Ok(result)
}
