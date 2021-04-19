use binread::io::{Read, Seek};
use binread::{BinResult, ReadOptions};

use crate::string::parse_string;

// TODO maybe add an actual JSON chat interface here, for now we'll just read the string
pub fn parse_chat<R: Read + Seek>(
    reader: &mut R,
    options: &ReadOptions,
    _args: (),
) -> BinResult<String> {
    parse_string(reader, options, (Some(262144),))
}
