use crate::aliases::{Float, Int};
use crate::item::Slot;
use crate::varint::parse_varint;
use binread::derive_binread;

#[derive_binread]
#[br(import(ty: Int))]
#[derive(Debug, PartialEq)]
pub enum Particle {
    // Might actual make the types later, for speed now I'm just grouping them
    #[br(pre_assert(matches!(ty, 0..=2 | 4..=13 | 15..=22 | 24..=31 | 33..=61)))]
    ParticleNoData,
    #[br(pre_assert(ty == 3))]
    Block {
        #[br(parse_with = parse_varint)]
        state: Int,
    },
    #[br(pre_assert(ty == 14))]
    Dust {
        red: Float,
        green: Float,
        blue: Float,
        scale: Float,
    },
    #[br(pre_assert(ty == 23))]
    FallingDust {
        #[br(parse_with = parse_varint)]
        state: Int,
    },
    #[br(pre_assert(ty == 32))]
    Item { item: Slot },
}
