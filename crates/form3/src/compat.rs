//! Dependency-free migration adapters for common terminal styling APIs.
//!
//! The adapters intentionally expose output effects, not third-party crate
//! identities. Applications can migrate existing chained colour calls to
//! `form3::compat::Colorize` without changing the bytes users see.

use crate::ansi::{self, AnsiColor, Attr, Color};
use crate::term::{ColorSupport, TermInfo};
use std::fmt;
use std::ops::Deref;

/// Lazily rendered styled text. Chained calls accumulate SGR state and emit a
/// single reset, avoiding the nested-reset bug common in string wrappers.
#[derive(Debug, Clone)]
pub struct StyledText {
    text: String,
    foreground: Option<AnsiColor>,
    background: Option<AnsiColor>,
    attrs: Vec<Attr>,
    color_support: ColorSupport,
}

impl StyledText {
    pub fn new(text: impl Into<String>) -> Self {
        Self {
            text: text.into(),
            foreground: None,
            background: None,
            attrs: Vec::new(),
            color_support: TermInfo::detect().color_support,
        }
    }

    pub fn with_color_support(mut self, support: ColorSupport) -> Self {
        self.color_support = support;
        self
    }

    fn foreground(mut self, color: Color, bright: bool) -> Self {
        self.foreground = Some(AnsiColor::Standard(color, bright));
        self
    }

    fn background(mut self, color: Color, bright: bool) -> Self {
        self.background = Some(AnsiColor::Standard(color, bright));
        self
    }

    fn set_fg(mut self, color: AnsiColor) -> Self {
        self.foreground = Some(color);
        self
    }

    fn set_bg(mut self, color: AnsiColor) -> Self {
        self.background = Some(color);
        self
    }

    fn attr(mut self, attr: Attr) -> Self {
        if !self
            .attrs
            .iter()
            .any(|present| *present as u8 == attr as u8)
        {
            self.attrs.push(attr);
        }
        self
    }

    pub fn plain(&self) -> &str {
        &self.text
    }

    pub fn as_str(&self) -> &str {
        &self.text
    }
}

impl fmt::Display for StyledText {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        if self.color_support == ColorSupport::NoColor
            || (self.foreground.is_none() && self.background.is_none() && self.attrs.is_empty())
        {
            return formatter.write_str(&self.text);
        }
        if let Some(color) = &self.foreground {
            formatter.write_str(&ansi::fg(color))?;
        }
        if let Some(color) = &self.background {
            formatter.write_str(&ansi::bg(color))?;
        }
        for attr in &self.attrs {
            formatter.write_str(&ansi::sgr(*attr))?;
        }
        formatter.write_str(&self.text)?;
        formatter.write_str(ansi::reset())
    }
}

impl Deref for StyledText {
    type Target = str;
    fn deref(&self) -> &Self::Target {
        &self.text
    }
}

impl AsRef<str> for StyledText {
    fn as_ref(&self) -> &str {
        &self.text
    }
}

impl PartialEq<&str> for StyledText {
    fn eq(&self, other: &&str) -> bool {
        self.text == *other
    }
}

/// Chainable effects corresponding to the common `Colorize` vocabulary.
pub trait Colorize {
    // `into_*` conventionally takes `self` by value, but this needs to
    // work on borrowed `&str`/`&String` too (that's the whole point of the
    // trait) — kept as-is to stay a faithful vendored copy of `~/3form`
    // rather than diverging from upstream over a naming lint.
    #[allow(clippy::wrong_self_convention)]
    fn into_styled(&self) -> StyledText;

    fn black(&self) -> StyledText {
        self.into_styled().foreground(Color::Black, false)
    }
    fn red(&self) -> StyledText {
        self.into_styled().foreground(Color::Red, false)
    }
    fn green(&self) -> StyledText {
        self.into_styled().foreground(Color::Green, false)
    }
    fn yellow(&self) -> StyledText {
        self.into_styled().foreground(Color::Yellow, false)
    }
    fn blue(&self) -> StyledText {
        self.into_styled().foreground(Color::Blue, false)
    }
    fn magenta(&self) -> StyledText {
        self.into_styled().foreground(Color::Magenta, false)
    }
    fn purple(&self) -> StyledText {
        self.magenta()
    }
    fn cyan(&self) -> StyledText {
        self.into_styled().foreground(Color::Cyan, false)
    }
    fn white(&self) -> StyledText {
        self.into_styled().foreground(Color::White, false)
    }
    fn bright_black(&self) -> StyledText {
        self.into_styled().foreground(Color::Black, true)
    }
    fn bright_red(&self) -> StyledText {
        self.into_styled().foreground(Color::Red, true)
    }
    fn bright_green(&self) -> StyledText {
        self.into_styled().foreground(Color::Green, true)
    }
    fn bright_yellow(&self) -> StyledText {
        self.into_styled().foreground(Color::Yellow, true)
    }
    fn bright_blue(&self) -> StyledText {
        self.into_styled().foreground(Color::Blue, true)
    }
    fn bright_magenta(&self) -> StyledText {
        self.into_styled().foreground(Color::Magenta, true)
    }
    fn bright_purple(&self) -> StyledText {
        self.bright_magenta()
    }
    fn bright_cyan(&self) -> StyledText {
        self.into_styled().foreground(Color::Cyan, true)
    }
    fn bright_white(&self) -> StyledText {
        self.into_styled().foreground(Color::White, true)
    }
    fn on_cyan(&self) -> StyledText {
        self.into_styled().background(Color::Cyan, false)
    }
    fn on_yellow(&self) -> StyledText {
        self.into_styled().background(Color::Yellow, false)
    }
    fn bold(&self) -> StyledText {
        self.into_styled().attr(Attr::Bold)
    }
    fn dimmed(&self) -> StyledText {
        self.into_styled().attr(Attr::Dim)
    }
    fn italic(&self) -> StyledText {
        self.into_styled().attr(Attr::Italic)
    }
    fn underline(&self) -> StyledText {
        self.into_styled().attr(Attr::Underline)
    }
    fn reversed(&self) -> StyledText {
        self.into_styled().attr(Attr::Reverse)
    }
    fn hidden(&self) -> StyledText {
        self.into_styled().attr(Attr::Hidden)
    }
    fn strikethrough(&self) -> StyledText {
        self.into_styled().attr(Attr::Strikethrough)
    }
    fn normal(&self) -> StyledText {
        let mut styled = self.into_styled();
        styled.attrs.clear();
        styled
    }
    fn color(&self, color: Color) -> StyledText {
        self.into_styled().foreground(color, false)
    }
    /// Foreground as a 24-bit RGB truecolor value, for brand palettes
    /// that don't map onto the standard 16 colors.
    fn rgb(&self, r: u8, g: u8, b: u8) -> StyledText {
        self.into_styled().set_fg(AnsiColor::Rgb(r, g, b))
    }
    /// Background as a 24-bit RGB truecolor value.
    fn on_rgb(&self, r: u8, g: u8, b: u8) -> StyledText {
        self.into_styled().set_bg(AnsiColor::Rgb(r, g, b))
    }
}

impl Colorize for str {
    fn into_styled(&self) -> StyledText {
        StyledText::new(self)
    }
}

impl Colorize for String {
    fn into_styled(&self) -> StyledText {
        StyledText::new(self)
    }
}

impl Colorize for StyledText {
    fn into_styled(&self) -> StyledText {
        self.clone()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn chained_effects_emit_one_reset() {
        let text = "ready"
            .bright_green()
            .bold()
            .with_color_support(ColorSupport::Standard16)
            .to_string();
        assert_eq!(text, "\x1b[92m\x1b[1mready\x1b[0m");
    }

    #[test]
    fn no_color_preserves_plain_text() {
        assert_eq!(
            "warning"
                .yellow()
                .bold()
                .with_color_support(ColorSupport::NoColor)
                .to_string(),
            "warning"
        );
    }
}
