use binread::BinRead;

#[derive(BinRead, Debug, Ord, PartialOrd, Eq, PartialEq)]
#[br(repr = u8)]
pub enum Direction {
    DOWN = 0,
    UP = 1,
    NORTH = 2,
    SOUTH = 3,
    WEST = 4,
    EAST = 5,
}
