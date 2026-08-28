use kakashi_sandbox::{ExecutionPolicy, ExecutionReport};
use serde::{Deserialize, Serialize};

/// A replayable experiment record (§13, §32): a program invocation plus
/// the exact policy it ran under and what was observed. Persisted to
/// `.kakashi/experiments/<id>.json` so `kakashi replay <id>` can re-run it
/// from the manifest alone.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Experiment {
    pub id: String,
    pub label: String,
    pub program: String,
    pub args: Vec<String>,
    pub policy: ExecutionPolicy,
    pub report: ExecutionReport,
}
