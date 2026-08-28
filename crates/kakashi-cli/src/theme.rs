//! The "Copycat Ninja" presentation layer — `--format human` output only.
//!
//! Kakashi Hatake copies other shinobi's techniques and reports, plainly,
//! whether the copy actually matches. That's this whole project's job
//! description, so it gets his colors: Konoha-leaf green for a confirmed
//! copy, Sharingan red for one that doesn't match, hitai-ate steel for
//! plain information.
//!
//! Everything here is cosmetic. It changes zero bytes of `--format json`
//! output, zero field names, and zero enum values — see `commands.rs`,
//! where every call site keeps printing the real `Severity`/`Disposition`
//! value and only *additionally* colors it or appends a one-line tagline.
//! `kakashi diff`/`validate`/`converge` piped to a file or a script must
//! see exactly the same text they always did, modulo ANSI codes that only
//! ever appear on an interactive terminal (see [`color_support`]).

use form3::anim::{Spinner, BRAILLE};
use form3::compat::{Colorize, StyledText};
use form3::term::ColorSupport;
use kakashi_core::Severity;
use std::io::IsTerminal;
use std::sync::mpsc;
use std::time::Duration;

/// Sharingan red: the left eye, and anything that doesn't match.
const SHARINGAN_RED: (u8, u8, u8) = (196, 30, 30);
/// Konoha leaf green: a confirmed copy.
const KONOHA_GREEN: (u8, u8, u8) = (76, 145, 65);
/// Hitai-ate steel: the headband plate, and plain information.
const HITAIATE_STEEL: (u8, u8, u8) = (150, 165, 175);
/// Chidori blue: work in progress.
const CHIDORI_BLUE: (u8, u8, u8) = (110, 190, 255);
/// Icha Icha orange: worth a second look, not yet a failure.
const ICHA_ICHA_ORANGE: (u8, u8, u8) = (222, 135, 40);

/// `form3` is deliberately ignorant of *which* stream is in play (see its
/// own `term` module doc comment) — this is the per-stream TTY check it
/// asks callers to layer on top, combined with its own `NO_COLOR`/`TERM`
/// detection. Piping `kakashi validate ... | tee report.txt` must see
/// plain text, exactly like every other command in this workspace.
pub fn color_support() -> ColorSupport {
    if std::io::stdout().is_terminal() {
        form3::term::TermInfo::detect().color_support
    } else {
        ColorSupport::NoColor
    }
}

fn styled(text: impl Into<String>) -> StyledText {
    StyledText::new(text).with_color_support(color_support())
}

fn rgb(text: &str, (r, g, b): (u8, u8, u8)) -> StyledText {
    styled(text).rgb(r, g, b)
}

/// Colors `severity`'s Debug label (`Critical`, `High`, `Low`,
/// `Informational` — the same strings every human-format printer in
/// `commands.rs` already used) without changing what the string says.
pub fn severity_label(severity: Severity) -> StyledText {
    let label = format!("{severity:?}");
    match severity {
        Severity::Critical => rgb(&label, SHARINGAN_RED).bold(),
        Severity::High => rgb(&label, ICHA_ICHA_ORANGE),
        Severity::Low => rgb(&label, HITAIATE_STEEL),
        Severity::Informational => styled(label).dimmed(),
    }
}

/// Colors a disposition string exactly as `Disposition::to_string()`
/// already renders it (`"EQUIVALENT"`, `"NON_EQUIVALENT"`, ...) — this
/// takes the string, not the enum, so it also colors the `report`
/// command's hardcoded `"INCONCLUSIVE"` the same way `validate` does.
pub fn disposition_label(disposition: &str) -> StyledText {
    match disposition {
        "EQUIVALENT" => rgb(disposition, KONOHA_GREEN).bold(),
        "CONDITIONALLY_EQUIVALENT" => rgb(disposition, CHIDORI_BLUE),
        "PARTIALLY_EQUIVALENT" => rgb(disposition, ICHA_ICHA_ORANGE),
        "NON_EQUIVALENT" => rgb(disposition, SHARINGAN_RED).bold(),
        "BLOCKED" => rgb(disposition, SHARINGAN_RED).bold().underline(),
        _ => styled(disposition).dimmed(), // INCONCLUSIVE and anything future
    }
}

/// A one-line, in-character gloss on a disposition string (the same
/// strings `Disposition::to_string()` / `disposition_label` use). Never a
/// substitute for the real value above it — see the module doc comment.
pub fn disposition_tagline(disposition: &str) -> &'static str {
    match disposition {
        "EQUIVALENT" => "Copied to the last hand seal. Sensei approves.",
        "CONDITIONALLY_EQUIVALENT" => "Close enough to pass — under the conditions observed.",
        "PARTIALLY_EQUIVALENT" => "Copied the shape of the jutsu, not the substance.",
        "NON_EQUIVALENT" => "That's not the same technique at all.",
        "BLOCKED" => "Even the Sharingan can't see through this one.",
        _ => "Not enough hand seals observed yet to call it.", // INCONCLUSIVE and anything future
    }
}

pub fn gate_label(passed: bool) -> StyledText {
    if passed {
        rgb("PASS", KONOHA_GREEN).bold()
    } else {
        rgb("FAIL", SHARINGAN_RED).bold()
    }
}

/// Colors `text` (kept verbatim — e.g. `kakashi-converge`'s `"Accept"`/
/// `"Rollback"` `Decision` debug label) green when `good`, red otherwise.
pub fn decision_label(text: &str, good: bool) -> StyledText {
    if good {
        rgb(text, KONOHA_GREEN)
    } else {
        rgb(text, SHARINGAN_RED)
    }
}

/// Colors an `Option<i32>` exit code exactly as its `{:?}` form already
/// printed (`Some(0)`, `Some(3)`, `None`) — green for a clean exit, red
/// for anything else, dim for `None` (the process never returned one).
pub fn exit_code_label(exit_code: Option<i32>) -> StyledText {
    let label = format!("{exit_code:?}");
    match exit_code {
        Some(0) => rgb(&label, KONOHA_GREEN),
        Some(_) => rgb(&label, ICHA_ICHA_ORANGE),
        None => styled(label).dimmed(),
    }
}

/// Colors a `bool` as its plain `Display` text (`"true"`/`"false"`) — used
/// for flags where `true` means something went wrong (e.g. `timed_out`),
/// so `true` is Sharingan red and `false` is Konoha green.
pub fn bool_label(value: bool) -> StyledText {
    if value {
        rgb("true", SHARINGAN_RED)
    } else {
        rgb("false", KONOHA_GREEN)
    }
}

pub fn adapter_status_label(available: bool) -> StyledText {
    if available {
        rgb("AVAILABLE", KONOHA_GREEN)
    } else {
        styled("CAPABILITY_UNAVAILABLE").dimmed()
    }
}

/// The banner shown once, at the top of `kakashi status`'s human output.
/// Two eyes, deliberately mismatched: the left one is the borrowed
/// Sharingan (red); the right is Kakashi's own, plain (steel grey).
pub fn banner() -> String {
    let left_eye = rgb("◉", SHARINGAN_RED);
    let right_eye = rgb("◈", HITAIATE_STEEL);
    let plate = rgb("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━", HITAIATE_STEEL);
    let title = styled("K A K A S H I").bold();
    let subtitle =
        styled("the copycat ninja — deterministic replication & equivalence engine").dimmed();
    format!("{plate}\n  {left_eye}  {title}  {right_eye}\n  {subtitle}\n{plate}",)
}

/// Runs `work` (a sandboxed-execution call that blocks on a subprocess)
/// while animating a braille spinner next to `label` — but only when
/// stdout is an interactive terminal; a redirected/piped stdout gets no
/// spinner frames mixed into its output, ever.
///
/// `form3::anim::Spinner` has no timing loop of its own by design (see its
/// module doc comment) — this is that loop, ticking every 80ms on the
/// main thread while `work` runs on a background one.
pub fn spin_while<T: Send + 'static>(label: &str, work: impl FnOnce() -> T + Send + 'static) -> T {
    if color_support() == ColorSupport::NoColor {
        return work();
    }

    let (tx, rx) = mpsc::channel();
    let handle = std::thread::spawn(move || {
        let result = work();
        let _ = tx.send(());
        result
    });

    let spinner = Spinner::new(&BRAILLE);
    let mut tick = 0u64;
    loop {
        match rx.recv_timeout(Duration::from_millis(80)) {
            Ok(()) => break,
            Err(mpsc::RecvTimeoutError::Timeout) => {
                eprint!("\r{} {label}", rgb(spinner.frame(tick), CHIDORI_BLUE));
                tick += 1;
            }
            Err(mpsc::RecvTimeoutError::Disconnected) => break,
        }
    }
    eprint!("\r{}\r", " ".repeat(label.len() + 2));

    handle.join().unwrap_or_else(|panic| {
        std::panic::resume_unwind(panic);
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use kakashi_core::Disposition;

    #[test]
    fn severity_label_never_changes_the_underlying_text() {
        for severity in [
            Severity::Critical,
            Severity::High,
            Severity::Low,
            Severity::Informational,
        ] {
            let expected = format!("{severity:?}");
            assert_eq!(
                severity_label(severity)
                    .with_color_support(ColorSupport::NoColor)
                    .to_string(),
                expected
            );
        }
    }

    #[test]
    fn disposition_label_never_changes_the_underlying_text() {
        for disposition in [
            Disposition::Equivalent,
            Disposition::ConditionallyEquivalent,
            Disposition::PartiallyEquivalent,
            Disposition::NonEquivalent,
            Disposition::Inconclusive,
            Disposition::Blocked,
        ] {
            let text = disposition.to_string();
            assert_eq!(
                disposition_label(&text)
                    .with_color_support(ColorSupport::NoColor)
                    .to_string(),
                text
            );
        }
    }

    #[test]
    fn every_disposition_has_a_distinct_tagline() {
        let taglines: std::collections::BTreeSet<&str> = [
            Disposition::Equivalent,
            Disposition::ConditionallyEquivalent,
            Disposition::PartiallyEquivalent,
            Disposition::NonEquivalent,
            Disposition::Inconclusive,
            Disposition::Blocked,
        ]
        .iter()
        .map(|d| disposition_tagline(&d.to_string()))
        .collect();
        assert_eq!(taglines.len(), 6);
    }

    #[test]
    fn gate_label_text_is_plain_pass_or_fail() {
        assert_eq!(
            gate_label(true)
                .with_color_support(ColorSupport::NoColor)
                .to_string(),
            "PASS"
        );
        assert_eq!(
            gate_label(false)
                .with_color_support(ColorSupport::NoColor)
                .to_string(),
            "FAIL"
        );
    }

    #[test]
    fn spin_while_returns_the_closure_result_without_a_terminal() {
        // color_support() is NoColor under `cargo test` (stdout isn't a
        // TTY), so this exercises the no-spinner fast path.
        let result = spin_while("working", || 2 + 2);
        assert_eq!(result, 4);
    }

    #[test]
    fn banner_contains_no_ansi_codes_without_color_support() {
        // banner() always calls styled()/rgb(), which already resolve
        // color_support() once per call — under `cargo test` that's
        // NoColor, so the banner must come back as plain text.
        assert!(!banner().contains('\x1b'));
        assert!(banner().contains("KAKASHI") || banner().contains("K A K A S H I"));
    }
}
