use clap::ValueEnum;
use serde::Serialize;

#[derive(Debug, Clone, Copy, PartialEq, Eq, ValueEnum)]
pub enum OutputFormat {
    Human,
    Json,
}

/// Print `value` as pretty JSON, or hand it to `human` for a
/// human-readable rendering — every command supports both (§33).
pub fn emit<T: Serialize>(format: OutputFormat, value: &T, human: impl FnOnce(&T)) {
    match format {
        OutputFormat::Json => {
            println!(
                "{}",
                serde_json::to_string_pretty(value).expect("serializable")
            );
        }
        OutputFormat::Human => human(value),
    }
}
