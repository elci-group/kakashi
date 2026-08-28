//! Frame-based animation primitives. `form3` has no runtime of its own —
//! a [`Spinner`] just maps a tick counter to a frame; the caller owns the
//! timing loop (a `tokio::time::interval`, a plain `std::thread::sleep`,
//! whatever fits) and decides when/where to print.

/// A braille-dot spin cycle. Renders cleanly in any UTF-8 terminal without
/// needing a wide font fallback.
pub const BRAILLE: [&str; 10] = ["⠋", "⠙", "⠹", "⠸", "⠼", "⠴", "⠦", "⠧", "⠇", "⠏"];

/// A plain ASCII spin cycle for terminals/logs that can't render braille.
pub const ASCII: [&str; 4] = ["|", "/", "-", "\\"];

#[derive(Debug, Clone, Copy)]
pub struct Spinner {
    frames: &'static [&'static str],
}

impl Default for Spinner {
    fn default() -> Self {
        Self { frames: &BRAILLE }
    }
}

impl Spinner {
    pub fn new(frames: &'static [&'static str]) -> Self {
        assert!(!frames.is_empty(), "Spinner needs at least one frame");
        Self { frames }
    }

    /// The frame for `tick`. Wraps around, so any monotonically
    /// increasing counter works.
    pub fn frame(&self, tick: u64) -> &'static str {
        self.frames[(tick as usize) % self.frames.len()]
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn frames_wrap_around() {
        let spinner = Spinner::default();
        assert_eq!(spinner.frame(0), BRAILLE[0]);
        assert_eq!(spinner.frame(BRAILLE.len() as u64), BRAILLE[0]);
    }

    #[test]
    fn custom_frame_set() {
        let spinner = Spinner::new(&ASCII);
        assert_eq!(spinner.frame(2), "-");
    }
}
