use std::fmt::{Display, Formatter};

use console::{style, Style, StyledObject};

pub(crate) fn style_e<D>(val: D) -> StyledObject<D> {
    style(val).for_stderr()
}

pub(crate) fn new_style_e() -> Style {
    Style::new().for_stderr()
}

pub(crate) struct TextComponent {
    pub text: String,
    pub style: Style,
    pub children: Vec<TextComponent>,
}

impl TextComponent {
    pub fn of(text: impl ToString) -> Self {
        TextComponent {
            text: text.to_string(),
            style: Style::default(),
            children: Vec::new(),
        }
    }

    pub fn of_styled(text: impl ToString, style: Style) -> Self {
        TextComponent {
            text: text.to_string(),
            style,
            children: Vec::new(),
        }
    }

    pub fn of_style(style: Style) -> Self {
        TextComponent {
            text: String::new(),
            style,
            children: Vec::new(),
        }
    }

    pub fn mutate_children<F>(mut self, mutator: F) -> Self
    where
        F: FnOnce(&mut Vec<TextComponent>) -> (),
    {
        mutator(&mut self.children);
        self
    }
}

impl Display for TextComponent {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.style.apply_to(&self.text))?;
        for child in self.children.iter() {
            write!(f, "{}", self.style.apply_to(&child))?;
        }

        Ok(())
    }
}
