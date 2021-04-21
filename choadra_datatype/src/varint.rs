use std::io::Write;
use std::ops::BitOrAssign;

use binread::io::Read;
use binread::{BinResult, ReadOptions};
use num_traits::{AsPrimitive, PrimInt};

use crate::aliases::{Int, Long};
use crate::writeable::Writeable;

pub fn parse_varint<R: Read>(reader: &mut R, options: &ReadOptions, args: ()) -> BinResult<Int> {
    parse_var_i(reader, options, args)
}

pub fn parse_varlong<R: Read>(reader: &mut R, options: &ReadOptions, args: ()) -> BinResult<Long> {
    parse_var_i(reader, options, args)
}

#[derive(Copy, Clone, Eq, PartialEq)]
pub struct VarInt(pub Int);
#[derive(Copy, Clone, Eq, PartialEq)]
pub struct VarLong(pub Long);

impl VarInt {
    /// Return the number of bytes need to represent the number.
    pub const fn bytes(self) -> usize {
        let mut j = 1;
        while j < 5 {
            if (self.0 & -1 << j * 7) == 0 {
                return j;
            }
            j += 1;
        }

        return 5;
    }
}

impl Writeable for VarInt {
    type Args = ();
    fn write_to<W: Write>(&self, write: &mut W, _: Self::Args) -> std::io::Result<()> {
        write_var_i(write, self.0)
    }
}

impl Writeable for VarLong {
    type Args = ();
    fn write_to<W: Write>(&self, write: &mut W, _: Self::Args) -> std::io::Result<()> {
        write_var_i(write, self.0)
    }
}

trait BitCount {
    const BITS: usize;
}

impl BitCount for i32 {
    const BITS: usize = 32;
}

impl BitCount for i64 {
    const BITS: usize = 64;
}

const ADDITIONAL_BYTES_BIT: u8 = 1 << 7;

fn parse_var_i<I, R>(reader: &mut R, _options: &ReadOptions, _args: ()) -> BinResult<I>
where
    I: PrimInt + BitOrAssign + 'static,
    u8: AsPrimitive<I>,
    R: Read,
{
    let bits = I::zero().count_zeros();
    let mut num_read: u32 = 0;
    let mut result: I = I::zero();
    let mut byte_holder = [0; 1];
    loop {
        reader.read_exact(&mut byte_holder)?;
        let value: u8 = byte_holder[0] & !ADDITIONAL_BYTES_BIT;
        result |= value.as_() << (num_read as usize);

        let additional_bytes = byte_holder[0] & ADDITIONAL_BYTES_BIT != 0;

        if additional_bytes {
            // Read 7 zero bits
            num_read += 7;
        } else {
            // Read however many bits were available, up to the leading 1
            num_read += 8 - value.leading_zeros();
        }

        if num_read > bits {
            return Err(binread::Error::Io(std::io::Error::new(
                std::io::ErrorKind::InvalidData,
                &*format!(
                    "Variable integer is too long: read {}, max bits {}",
                    num_read, bits
                ),
            )));
        }

        if !additional_bytes {
            return Ok(result);
        }
    }
}

trait IntoLowerByte {
    fn into_lower_byte(self) -> u8;
}

impl IntoLowerByte for u32 {
    fn into_lower_byte(self) -> u8 {
        self as u8
    }
}

impl IntoLowerByte for u64 {
    fn into_lower_byte(self) -> u8 {
        self as u8
    }
}

fn write_var_i<I, W>(writer: &mut W, mut value: I) -> std::io::Result<()>
where
    I: PrimInt + AsPrimitive<u8>,
    W: Write,
{
    loop {
        let mut temp = (value.as_()) & !ADDITIONAL_BYTES_BIT;
        value = value.unsigned_shr(7);
        if value != I::zero() {
            temp |= ADDITIONAL_BYTES_BIT;
        }
        writer.write_all(&[temp])?;
        if value == I::zero() {
            break;
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use std::fmt::Debug;

    use binread::io::Cursor;
    use once_cell::sync::Lazy;

    use super::*;

    fn ez_parse<I>(data: &[u8]) -> I
    where
        I: PrimInt + BitOrAssign + 'static,
        u8: AsPrimitive<I>,
    {
        parse_var_i(&mut Cursor::new(data), &ReadOptions::default(), ())
            .expect("Failed to parse var int")
    }

    fn ez_failure<I>(data: &[u8]) -> binread::Error
    where
        I: PrimInt + Debug + BitOrAssign + 'static,
        u8: AsPrimitive<I>,
    {
        parse_var_i::<I, _>(&mut Cursor::new(data), &ReadOptions::default(), ())
            .expect_err("Shouldn't have succeeded at parsing")
    }

    static VARINT_CASES: Lazy<Vec<(i32, &[u8])>> = Lazy::new(|| {
        vec![
            (4i32, &[4]),
            (0i32, &[0]),
            (0x80i32, &[0x80, 0x01]),
            (0xB1Ai32, &[0x9A, 0x16]),
            (-0x80_00_00_00i32, &[0x80, 0x80, 0x80, 0x80, 0x8]),
            (0x01_01_01_01i32, &[0x81, 0x82, 0x84, 0x08]),
        ]
    });

    #[test]
    fn parse_varint_test() {
        for (expected, input) in VARINT_CASES.iter() {
            eprintln!("{:X?}", input);
            assert_eq!(*expected, ez_parse::<Int>(*input));
        }
    }

    #[test]
    fn write_varint_test() {
        for (input, expected) in VARINT_CASES.iter() {
            let mut new_data = Vec::new();
            VarInt(*input).write_to(&mut new_data, ()).unwrap();
            assert_eq!(*expected, &new_data);
        }
    }

    #[test]
    fn parse_varint_empty_array() {
        let failure = ez_failure::<Int>(&[]);
        let io_failure = match failure {
            binread::Error::Io(io) => io,
            x => panic!("{:?} was not expected", x),
        };
        assert_eq!(
            "failed to fill whole buffer",
            io_failure
                .into_inner()
                .expect("Non-custom error")
                .to_string()
        );
    }

    #[test]
    fn parse_varint_too_large() {
        let failure = ez_failure::<Int>(&[0x80, 0x80, 0x80, 0x80, 0x10]);
        let io_failure = match failure {
            binread::Error::Io(io) => io,
            x => panic!("{:?} was not expected", x),
        };
        assert_eq!(
            "Variable integer is too long: read 33, max bits 32",
            io_failure
                .into_inner()
                .expect("Non-custom error")
                .to_string()
        );
    }

    static VARLONG_CASES: Lazy<Vec<(i64, &[u8])>> = Lazy::new(|| {
        vec![
            (4i64, &[4]),
            (0i64, &[0]),
            (0x80i64, &[0x80, 0x01]),
            (0x80_00_00_00i64, &[0x80, 0x80, 0x80, 0x80, 0x8]),
            (0x01_01_01_01i64, &[0x81, 0x82, 0x84, 0x08]),
            (
                -0x80_00_00_00_00_00_00_00i64,
                &[0x80, 0x80, 0x80, 0x80, 0x80, 0x80, 0x80, 0x80, 0x80, 0x1],
            ),
            (
                0x01_01_01_01__01_01_01_01i64,
                &[0x81, 0x82, 0x84, 0x88, 0x90, 0xA0, 0xC0, 0x80, 0x1],
            ),
        ]
    });

    #[test]
    fn parse_varlong_test() {
        for (expected, input) in VARLONG_CASES.iter() {
            assert_eq!(*expected, ez_parse::<Long>(*input));
        }
    }

    #[test]
    fn write_varlong_test() {
        for (input, expected) in VARLONG_CASES.iter() {
            let mut new_data = Vec::new();
            VarLong(*input).write_to(&mut new_data, ()).unwrap();
            assert_eq!(*expected, &new_data);
        }
    }

    #[test]
    fn parse_varlong_empty_array() {
        let failure = ez_failure::<Long>(&[]);
        let io_failure = match failure {
            binread::Error::Io(io) => io,
            x => panic!("{:?} was not expected", x),
        };
        assert_eq!(
            "failed to fill whole buffer",
            io_failure
                .into_inner()
                .expect("Non-custom error")
                .to_string()
        );
    }

    #[test]
    fn parse_varlong_too_large() {
        let failure =
            ez_failure::<Long>(&[0x80, 0x80, 0x80, 0x80, 0x80, 0x80, 0x80, 0x80, 0x80, 0x02]);
        let io_failure = match failure {
            binread::Error::Io(io) => io,
            x => panic!("{:?} was not expected", x),
        };
        assert_eq!(
            "Variable integer is too long: read 65, max bits 64",
            io_failure
                .into_inner()
                .expect("Non-custom error")
                .to_string()
        );
    }
}
