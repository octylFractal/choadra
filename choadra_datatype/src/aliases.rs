use std::io::{Result, Write};

use crate::writeable::Writeable;

pub type Boolean = bool;
pub type Byte = i8;
pub type UnsignedByte = u8;
pub type Short = i16;
pub type UnsignedShort = u16;
pub type Int = i32;
pub type Long = i64;
pub type Float = f32;
pub type Double = f64;

impl Writeable for Boolean {
    type Args = ();

    fn write_to<W: Write>(&self, write: &mut W, _: Self::Args) -> Result<()> {
        write.write_all(&[*self as u8])
    }
}

impl Writeable for Byte {
    type Args = ();

    fn write_to<W: Write>(&self, write: &mut W, _: Self::Args) -> Result<()> {
        write.write_all(&[*self as u8])
    }
}

impl Writeable for UnsignedByte {
    type Args = ();

    fn write_to<W: Write>(&self, write: &mut W, _: Self::Args) -> Result<()> {
        write.write_all(&[*self as u8])
    }
}

impl Writeable for Short {
    type Args = ();

    fn write_to<W: Write>(&self, write: &mut W, _: Self::Args) -> Result<()> {
        let unsigned = *self as u16;
        unsigned.write_to(write, ())
    }
}

impl Writeable for UnsignedShort {
    type Args = ();

    fn write_to<W: Write>(&self, write: &mut W, _: Self::Args) -> Result<()> {
        write.write_all(&[(*self >> 8) as u8, *self as u8])
    }
}

impl Writeable for Int {
    type Args = ();

    fn write_to<W: Write>(&self, write: &mut W, _: Self::Args) -> Result<()> {
        let unsigned = *self as u32;
        unsigned.write_to(write, ())
    }
}

impl Writeable for u32 {
    type Args = ();

    fn write_to<W: Write>(&self, write: &mut W, _: Self::Args) -> Result<()> {
        write_unsigned_int(write, *self)
    }
}

fn write_unsigned_int<W: Write>(write: &mut W, unsigned: u32) -> Result<()> {
    write.write_all(&[
        (unsigned >> 3 * 8) as u8,
        (unsigned >> 2 * 8) as u8,
        (unsigned >> 1 * 8) as u8,
        unsigned as u8,
    ])
}

impl Writeable for Long {
    type Args = ();

    fn write_to<W: Write>(&self, write: &mut W, _: Self::Args) -> Result<()> {
        let unsigned = *self as u64;
        unsigned.write_to(write, ())
    }
}

impl Writeable for u64 {
    type Args = ();

    fn write_to<W: Write>(&self, write: &mut W, _: Self::Args) -> Result<()> {
        write_unsigned_long(write, *self)
    }
}

fn write_unsigned_long<W: Write>(write: &mut W, unsigned: u64) -> Result<()> {
    write.write_all(&[
        (unsigned >> 7 * 8) as u8,
        (unsigned >> 6 * 8) as u8,
        (unsigned >> 5 * 8) as u8,
        (unsigned >> 4 * 8) as u8,
        (unsigned >> 3 * 8) as u8,
        (unsigned >> 2 * 8) as u8,
        (unsigned >> 1 * 8) as u8,
        unsigned as u8,
    ])
}

impl Writeable for Float {
    type Args = ();

    fn write_to<W: Write>(&self, write: &mut W, _: Self::Args) -> Result<()> {
        write_unsigned_int(write, self.to_bits())
    }
}

impl Writeable for Double {
    type Args = ();

    fn write_to<W: Write>(&self, write: &mut W, _: Self::Args) -> Result<()> {
        write_unsigned_long(write, self.to_bits())
    }
}
