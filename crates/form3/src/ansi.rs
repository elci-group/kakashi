//! Raw SGR (Select Graphic Rendition) sequence construction. Callers pick
//! *whether* to emit these (see [`crate::term`]); this module only knows
//! how to spell them correctly.

/// One of the 8 standard terminal colors. Paired with a `bright` flag in
/// [`AnsiColor::Standard`] to reach all 16.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Color {
    Black,
    Red,
    Green,
    Yellow,
    Blue,
    Magenta,
    Cyan,
    White,
}

impl Color {
    fn base_offset(self) -> u8 {
        match self {
            Color::Black => 0,
            Color::Red => 1,
            Color::Green => 2,
            Color::Yellow => 3,
            Color::Blue => 4,
            Color::Magenta => 5,
            Color::Cyan => 6,
            Color::White => 7,
        }
    }
}

/// A color to render, either from the 16-color standard palette or as a
/// 24-bit RGB truecolor value (`38;2;r;g;b` / `48;2;r;g;b`) for brand
/// palettes that don't map cleanly onto the standard 16.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AnsiColor {
    Standard(Color, bool),
    Rgb(u8, u8, u8),
}

/// A single SGR text attribute (bold, underline, ...).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Attr {
    Bold,
    Dim,
    Italic,
    Underline,
    Reverse,
    Hidden,
    Strikethrough,
}

/// Foreground-color escape sequence for `color`.
pub fn fg(color: &AnsiColor) -> String {
    match *color {
        AnsiColor::Standard(c, bright) => {
            let code = 30 + c.base_offset() + if bright { 60 } else { 0 };
            format!("\x1b[{code}m")
        }
        AnsiColor::Rgb(r, g, b) => format!("\x1b[38;2;{r};{g};{b}m"),
    }
}

/// Background-color escape sequence for `color`.
pub fn bg(color: &AnsiColor) -> String {
    match *color {
        AnsiColor::Standard(c, bright) => {
            let code = 40 + c.base_offset() + if bright { 60 } else { 0 };
            format!("\x1b[{code}m")
        }
        AnsiColor::Rgb(r, g, b) => format!("\x1b[48;2;{r};{g};{b}m"),
    }
}

/// Escape sequence for a single text attribute.
pub fn sgr(attr: Attr) -> String {
    let code = match attr {
        Attr::Bold => 1,
        Attr::Dim => 2,
        Attr::Italic => 3,
        Attr::Underline => 4,
        Attr::Reverse => 7,
        Attr::Hidden => 8,
        Attr::Strikethrough => 9,
    };
    format!("\x1b[{code}m")
}

/// The sequence that clears all SGR state back to the terminal default.
pub fn reset() -> &'static str {
    "\x1b[0m"
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn standard_fg_matches_classic_sgr_codes() {
        assert_eq!(fg(&AnsiColor::Standard(Color::Red, false)), "\x1b[31m");
        assert_eq!(fg(&AnsiColor::Standard(Color::Green, true)), "\x1b[92m");
    }

    #[test]
    fn rgb_fg_emits_truecolor_sequence() {
        assert_eq!(fg(&AnsiColor::Rgb(176, 38, 255)), "\x1b[38;2;176;38;255m");
    }

    #[test]
    fn bg_uses_40_offset() {
        assert_eq!(bg(&AnsiColor::Standard(Color::Cyan, false)), "\x1b[46m");
    }
}
