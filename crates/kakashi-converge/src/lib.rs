//! The convergence ledger and monotonic-progress gate (§38, §39).
//! Deliberately small: this crate decides *whether an iteration's result
//! should be accepted*, given before/after scores and regression counts.
//! It does not decide *what change to try next* — that's Cambrian's job
//! per the doctrine, wired in through `kakashi-elci` once available.

use kakashi_core::EquivalenceScores;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Decision {
    Accept,
    Rollback,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LedgerEntry {
    pub iteration: u64,
    pub timestamp: chrono::DateTime<chrono::Utc>,
    pub before_score: Option<f64>,
    pub after_score: Option<f64>,
    pub delta: Option<f64>,
    pub changes: Vec<String>,
    pub tests_run: usize,
    pub regressions: usize,
    pub decision: Decision,
}

/// Apply the monotonic progress requirement (§39): accept only if the
/// composite score did not regress and no new regressions were introduced.
/// A `None` after-score (nothing measured) is never accepted — silence is
/// not progress.
pub fn evaluate_iteration(
    iteration: u64,
    before: &EquivalenceScores,
    after: &EquivalenceScores,
    changes: Vec<String>,
    tests_run: usize,
    regressions: usize,
) -> LedgerEntry {
    let before_score = before.composite();
    let after_score = after.composite();

    let decision = match (before_score, after_score) {
        (_, None) => Decision::Rollback,
        (None, Some(_)) if regressions == 0 => Decision::Accept,
        (Some(b), Some(a)) if a >= b && regressions == 0 => Decision::Accept,
        _ => Decision::Rollback,
    };

    LedgerEntry {
        iteration,
        timestamp: kakashi_core::now(),
        before_score,
        after_score,
        delta: match (before_score, after_score) {
            (Some(b), Some(a)) => Some(a - b),
            _ => None,
        },
        changes,
        tests_run,
        regressions,
        decision,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use kakashi_core::EquivalenceDimension;

    fn scores(v: f64) -> EquivalenceScores {
        let mut s = EquivalenceScores::default();
        s.set(EquivalenceDimension::Structural, v);
        s
    }

    #[test]
    fn accepts_improving_iteration_with_no_regressions() {
        let entry = evaluate_iteration(1, &scores(0.5), &scores(0.8), vec![], 3, 0);
        assert_eq!(entry.decision, Decision::Accept);
        assert!((entry.delta.unwrap() - 0.3).abs() < 1e-9);
    }

    #[test]
    fn rolls_back_when_score_regresses() {
        let entry = evaluate_iteration(1, &scores(0.8), &scores(0.5), vec![], 3, 0);
        assert_eq!(entry.decision, Decision::Rollback);
    }

    #[test]
    fn rolls_back_on_any_regression_even_if_score_improves() {
        let entry = evaluate_iteration(1, &scores(0.5), &scores(0.9), vec![], 3, 1);
        assert_eq!(entry.decision, Decision::Rollback);
    }

    #[test]
    fn rolls_back_when_nothing_was_measured_after() {
        let entry =
            evaluate_iteration(1, &scores(0.5), &EquivalenceScores::default(), vec![], 0, 0);
        assert_eq!(entry.decision, Decision::Rollback);
        assert_eq!(entry.after_score, None);
    }
}
