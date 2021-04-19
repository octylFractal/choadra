use binread::{BinResult, ReadOptions};

use crate::modified_utf8::decode_modified_utf8;
use binread::io::{Read, Seek};

pub fn parse_modified_utf8<R: Read + Seek>(
    reader: &mut R,
    options: &ReadOptions,
    _args: (),
) -> BinResult<String> {
    let c = options.count.expect("No count for options");

    let mut content = vec![0; c];
    reader.read_exact(&mut content)?;
    Ok(decode_modified_utf8(&content)?)
}
