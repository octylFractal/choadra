use console::{style, StyledObject};

pub(crate) fn style_e<D>(val: D) -> StyledObject<D> {
    style(val).for_stderr()
}
