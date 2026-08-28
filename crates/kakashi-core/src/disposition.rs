use serde::{Deserialize, Serialize};

/// Final disposition of a replication attempt (§41). `Equivalent` may only
/// be reported when every mandatory dimension's gate passed — never as a
/// stand-in for "the composite score looked good".
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum Disposition {
    Equivalent,
    ConditionallyEquivalent,
    PartiallyEquivalent,
    NonEquivalent,
    Inconclusive,
    Blocked,
}

impl std::fmt::Display for Disposition {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let s = match self {
            Self::Equivalent => "EQUIVALENT",
            Self::ConditionallyEquivalent => "CONDITIONALLY_EQUIVALENT",
            Self::PartiallyEquivalent => "PARTIALLY_EQUIVALENT",
            Self::NonEquivalent => "NON_EQUIVALENT",
            Self::Inconclusive => "INCONCLUSIVE",
            Self::Blocked => "BLOCKED",
        };
        f.write_str(s)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn display_matches_serde_wire_format() {
        for d in [
            Disposition::Equivalent,
            Disposition::ConditionallyEquivalent,
            Disposition::PartiallyEquivalent,
            Disposition::NonEquivalent,
            Disposition::Inconclusive,
            Disposition::Blocked,
        ] {
            let json = serde_json::to_string(&d).unwrap();
            let wire_value: String = serde_json::from_str(&json).unwrap();
            assert_eq!(d.to_string(), wire_value);
        }
    }
}
