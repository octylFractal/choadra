use std::io::Write;

pub trait Writeable {
    type Args;

    fn write_to<W: Write>(&self, write: &mut W, args: Self::Args) -> std::io::Result<()>;
}
