use std::fmt::{Display, Formatter};
use std::io::Write;

use binread::io::{Read, Seek};
use binread::{BinRead, BinReaderExt, BinResult, ReadOptions};

use crate::writeable::Writeable;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Position {
    x: i32,
    y: i16,
    z: i32,
}

impl Display for Position {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "({}, {}, {})", self.x, self.y, self.z)
    }
}

#[derive(Debug, Eq, PartialEq)]
pub enum PositionError {
    OutOfRange { axis: Axis },
}

#[derive(Debug, Eq, PartialEq)]
pub enum Axis {
    X,
    Y,
    Z,
}

impl Position {
    pub fn try_new(x: i32, y: i16, z: i32) -> Result<Position, PositionError> {
        if !matches!(x, -33554432..=33554431) {
            return Err(PositionError::OutOfRange { axis: Axis::X });
        }
        if !matches!(y, -2048..=2047) {
            return Err(PositionError::OutOfRange { axis: Axis::Y });
        }
        if !matches!(z, -33554432..=33554431) {
            return Err(PositionError::OutOfRange { axis: Axis::Z });
        }
        Ok(Position { x, y, z })
    }
}

impl BinRead for Position {
    type Args = ();

    fn read_options<R: Read + Seek>(
        reader: &mut R,
        options: &ReadOptions,
        _args: Self::Args,
    ) -> BinResult<Self> {
        let long_value = reader.read_type::<u64>(options.endian)?;
        Ok(Position::from(long_value))
    }
}

impl Writeable for Position {
    type Args = ();

    fn write_to<W: Write>(&self, write: &mut W, _: Self::Args) -> std::io::Result<()> {
        u64::from(self).write_to(write, ())
    }
}

const BITS_26: u64 = 0x3FFFFFF;
const BITS_12: u64 = 0xFFF;

impl From<u64> for Position {
    fn from(value: u64) -> Self {
        // TODO make this work in pure binops
        let mut x = (value >> 38) as i32;
        let mut y = (value & BITS_12) as i16;
        let mut z = ((value >> 12) & BITS_26) as i32;
        if x > 2i32.pow(25) {
            x -= 2i32.pow(26);
        }
        if y > 2i16.pow(11) {
            y -= 2i16.pow(12);
        }
        if z > 2i32.pow(25) {
            z -= 2i32.pow(26);
        }
        // We don't need to check this, the way these work we accept any long-packed value
        Position { x, y, z }
    }
}

impl From<Position> for u64 {
    fn from(p: Position) -> Self {
        to_long_packed_form(p.x, p.y, p.z)
    }
}

impl From<&Position> for u64 {
    fn from(p: &Position) -> Self {
        to_long_packed_form(p.x, p.y, p.z)
    }
}

fn to_long_packed_form(x: i32, y: i16, z: i32) -> u64 {
    ((x as u64 & BITS_26) << 38) | ((z as u64 & BITS_26) << 12) | (y as u64 & BITS_12)
}

#[cfg(test)]
mod tests {
    use once_cell::sync::Lazy;

    use super::*;

    #[test]
    fn try_new() {
        assert_eq!(
            Ok(Position {
                x: 100,
                y: 32,
                z: -100
            }),
            Position::try_new(100, 32, -100)
        );
    }

    #[test]
    fn try_new_failures() {
        assert_eq!(
            Err(PositionError::OutOfRange { axis: Axis::X }),
            Position::try_new(i32::MAX, 0, 0)
        );
        assert_eq!(
            Err(PositionError::OutOfRange { axis: Axis::X }),
            Position::try_new(i32::MIN, 0, 0)
        );
        assert_eq!(
            Err(PositionError::OutOfRange { axis: Axis::Y }),
            Position::try_new(0, i16::MAX, 0)
        );
        assert_eq!(
            Err(PositionError::OutOfRange { axis: Axis::Y }),
            Position::try_new(0, i16::MIN, 0)
        );
        assert_eq!(
            Err(PositionError::OutOfRange { axis: Axis::Z }),
            Position::try_new(0, 0, i32::MAX)
        );
        assert_eq!(
            Err(PositionError::OutOfRange { axis: Axis::Z }),
            Position::try_new(0, 0, i32::MIN)
        );
    }

    static TESTS: Lazy<Vec<(Position, u64)>> = Lazy::new(|| {
        vec![
            (Position { x: 0, y: 0, z: 0 }, 0),
            (Position { x: 1, y: 1, z: 1 }, 274877911041),
            (
                Position {
                    x: 30000000,
                    y: 255,
                    z: 30000000,
                },
                8246337331200000255,
            ),
            (
                Position {
                    x: -29999999,
                    y: -2000,
                    z: -20399999,
                },
                10200407331586971696,
            ),
        ]
    });

    #[test]
    fn from_u64() {
        for (expected, seed) in TESTS.iter() {
            assert_eq!(expected, &Position::from(*seed));
        }
    }

    #[test]
    fn to_u64() {
        for (seed, expected) in TESTS.iter() {
            assert_eq!(*expected, u64::from(seed));
        }
    }
}
