use std::io::SeekFrom;

use binread::io::{Read, Seek};
use binread::{BinRead, BinResult, ReadOptions};
use num_bigint::BigInt;

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

pub(crate) fn mojang_hex_encode(bytes: &[u8]) -> String {
    let big_int = BigInt::from_signed_bytes_be(bytes);
    big_int.to_str_radix(16)
}

#[cfg(test)]
mod tests {
    use sha1::{Digest, Sha1};

    use super::*;

    #[test]
    fn test_mojang_hex_encode() {
        let cases = vec![
            ("Notch", "4ed1f46bbe04bc756bcb17c0c7ce3e4632f06a48"),
            ("jeb_", "-7c9d5b0044c130109a5d7b5fb5c317c02b4e28c1"),
            ("simon", "88e16a1019277b15d58faf0541e11910eb756f6"),
        ];
        for (input, expected) in cases {
            assert_eq!(expected, mojang_hex_encode(&Sha1::digest(input.as_bytes())));
        }
    }
}
