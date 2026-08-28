use kakashi_core::EvidenceEnvelope;
use serde_json::Value;

pub const INVOKE_UNAVAILABLE: &str = "CAPABILITY_UNAVAILABLE";

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AdapterKind {
    /// A sibling ELCI project that exposes a CLI binary Kakashi can shell
    /// out to.
    Binary,
    /// A sibling ELCI project that is a library/workspace with no CLI
    /// entry point Kakashi currently invokes. Detected as present, but
    /// `invoke()` always reports `CAPABILITY_UNAVAILABLE`.
    Library,
}

#[derive(Debug, Clone)]
pub struct AdapterHealth {
    pub available: bool,
    pub detail: String,
}

/// The common adapter contract every ELCI integration implements (§36).
pub trait ElciAdapter {
    fn name(&self) -> &str;
    fn kind(&self) -> AdapterKind;
    fn detect(&self) -> bool;
    fn capabilities(&self) -> Vec<&'static str>;
    fn version(&self) -> Option<String>;
    fn health(&self) -> AdapterHealth;

    /// Invoke the adapter's underlying tool. Returns `Err` describing
    /// `CAPABILITY_UNAVAILABLE` when the tool isn't a directly invokable
    /// binary, or isn't detected at all.
    fn invoke(&self, args: &[String]) -> anyhow::Result<Value>;

    /// Translate a raw invocation result into the shared evidence
    /// envelope shape so downstream consumers never see adapter-specific
    /// formats (§21, §36).
    fn translate_evidence(&self, raw: &Value) -> EvidenceEnvelope {
        EvidenceEnvelope::new(
            self.name().to_string(),
            self.name().to_string(),
            format!("kakashi-elci::{}", self.name()),
        )
        .with_evidence(raw.clone())
    }
}
