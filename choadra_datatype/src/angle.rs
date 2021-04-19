use std::fmt::{Display, Formatter};

use binread::derive_binread;

#[derive_binread]
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub struct Angle(u8);

impl Display for Angle {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}/256", self.0)
    }
}

impl Angle {
    pub fn to_degrees(&self) -> f64 {
        (360.0 * (self.0 as f64)) / 256.0
    }

    pub fn to_radians(&self) -> f64 {
        (std::f64::consts::PI * 2.0 * (self.0 as f64)) / 256.0
    }
}
