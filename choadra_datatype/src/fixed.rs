use binread::io::{Read, Seek};
use binread::{BinReaderExt, BinResult, ReadOptions};
use fixed::types::extra::{LeEqU32, U3};
use fixed::FixedI32;

pub type EffectPositionValue = FixedI32<U3>;

pub fn parse_fixed<T: LeEqU32, R: Read + Seek>(
    reader: &mut R,
    options: &ReadOptions,
    _args: (),
) -> BinResult<FixedI32<T>> {
    let value = reader.read_type::<i32>(options.endian)?;
    let divided = (value as f32) / ((1 << T::I32) as f32);
    Ok(FixedI32::from_num(divided))
}
