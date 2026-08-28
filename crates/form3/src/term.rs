//! Whether to emit color at all. This module is deliberately ignorant of
//! *which* stream is in play (stdout vs stderr) — callers with more than
//! one output stream should check `std::io::IsTerminal` themselves per
//! stream and downgrade to [`ColorSupport::NoColor`] when redirected,
//! then feed the result in via `StyledText::with_color_support`.
//! [`TermInfo::detect`] only covers the environment-wide overrides that
//! apply regardless of stream: the `NO_COLOR` convention
//! (<https://no-color.org>) and `TERM=dumb`.

/// How much color a caller has decided is safe to emit.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum ColorSupport {
    #[default]
    NoColor,
    Standard16,
}

/// Environment-derived color defaults.
#[derive(Debug, Clone, Copy)]
pub struct TermInfo {
    pub color_support: ColorSupport,
}

impl TermInfo {
    pub fn detect() -> Self {
        let no_color = std::env::var_os("NO_COLOR").is_some();
        let dumb = std::env::var("TERM")
            .map(|term| term == "dumb")
            .unwrap_or(false);
        let color_support = if no_color || dumb {
            ColorSupport::NoColor
        } else {
            ColorSupport::Standard16
        };
        Self { color_support }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn no_color_env_forces_no_color() {
        std::env::set_var("NO_COLOR", "1");
        assert_eq!(TermInfo::detect().color_support, ColorSupport::NoColor);
        std::env::remove_var("NO_COLOR");
    }
}
