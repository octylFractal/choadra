//! Modified UTF-8 coders, for Java DataInput's UTF-8 format.

use std::io::{Cursor, Read, Write};

pub fn encode_modified_utf8<W: Write>(s: &str, mut dest: W) -> Result<(), std::io::Error> {
    for c in s.encode_utf16() {
        match c {
            0x0001..=0x007F => {
                dest.write_all(&[c as u8])?;
            }
            0x0000 | 0x0080..=0x07FF => {
                dest.write_all(&[(0xC0 | (0x1F & (c >> 6))) as u8, (0x80 | (0x3F & c)) as u8])?;
            }
            0x0800..=0xFFFF => {
                dest.write_all(&[
                    (0xE0 | (0x0F & (c >> 12))) as u8,
                    (0x80 | (0x3F & (c >> 6))) as u8,
                    (0x80 | (0x3F & c)) as u8,
                ])?;
            }
        };
    }

    Ok(())
}

const HIGHEST_ONE_BIT: u8 = 1 << 7;
const HIGHEST_TWO_BITS: u8 = 3 << 6;
const HIGHEST_THREE_BITS: u8 = 7 << 5;
const HIGHEST_FOUR_BITS: u8 = 15 << 4;
const ONE_BYTE: u8 = 0;
const TWO_BYTES: u8 = 3 << 6;
const THREE_BYTES: u8 = 7 << 5;
const CHECKSUM_FLAG: u8 = 1 << 7;
const INVALID_BYTE_COUNT: u8 = 15 << 4;

pub fn decode_modified_utf8(v: &[u8]) -> Result<String, std::io::Error> {
    // pre-size to data length, it may be too large but that is OK
    let mut dest = Vec::with_capacity(v.len());

    let mut cursor = Cursor::new(v);
    let mut byte_holder = [0; 1];
    while (cursor.position() as usize) < v.len() {
        let first_byte = {
            cursor.read_exact(&mut byte_holder).unwrap();
            byte_holder[0]
        };
        // Check 1-byte case
        if first_byte & HIGHEST_ONE_BIT == ONE_BYTE {
            dest.push(first_byte as u16);
            continue;
        }

        // Check 2-byte case
        if first_byte & HIGHEST_THREE_BITS == TWO_BYTES {
            let second_byte = {
                cursor.read_exact(&mut byte_holder).unwrap();
                byte_holder[0]
            };
            if second_byte & HIGHEST_TWO_BITS != CHECKSUM_FLAG {
                return Err(std::io::Error::new(
                    std::io::ErrorKind::InvalidData,
                    "Highest two bits (2-byte case, second byte) were invalid",
                ));
            }
            dest.push(((first_byte as u16 & 0x1F) << 6) | (second_byte as u16 & 0x3F));
            continue;
        }

        // Check 3-byte case
        if first_byte & HIGHEST_FOUR_BITS == THREE_BYTES {
            let second_byte = {
                cursor.read_exact(&mut byte_holder).unwrap();
                byte_holder[0]
            };
            if second_byte & HIGHEST_TWO_BITS != CHECKSUM_FLAG {
                return Err(std::io::Error::new(
                    std::io::ErrorKind::InvalidData,
                    "Highest two bits (3-byte case, second byte) were invalid",
                ));
            }
            let third_byte = {
                cursor.read_exact(&mut byte_holder).unwrap();
                byte_holder[0]
            };
            if third_byte & HIGHEST_TWO_BITS != CHECKSUM_FLAG {
                return Err(std::io::Error::new(
                    std::io::ErrorKind::InvalidData,
                    "Highest two bits (3-byte case, third byte) were invalid",
                ));
            }
            dest.push(
                ((first_byte as u16 & 0x0F) << 12)
                    | ((second_byte as u16 & 0x3F) << 6)
                    | (third_byte as u16 & 0x3F),
            );
            continue;
        }

        // These should all be error cases
        if first_byte & HIGHEST_FOUR_BITS == INVALID_BYTE_COUNT {
            return Err(std::io::Error::new(
                std::io::ErrorKind::InvalidData,
                "Highest four bits were invalid",
            ));
        }
        if first_byte & HIGHEST_TWO_BITS == CHECKSUM_FLAG {
            return Err(std::io::Error::new(
                std::io::ErrorKind::InvalidData,
                "Highest two bits (1-byte case, first byte) were invalid",
            ));
        }
        unreachable!("Neither valid nor invalid byte? {}", first_byte);
    }

    Ok(String::from_utf16(&dest)
        .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e))?)
}
