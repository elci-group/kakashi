//! ELCI adapters (§3, §36). Kakashi never recreates functionality that an
//! ELCI sibling tool already owns; it detects the sibling and either uses
//! it or reports `CAPABILITY_UNAVAILABLE` — it must never silently change
//! its own behaviour when a sibling is missing.

pub mod adapter;
pub mod registry;

pub use adapter::{AdapterHealth, AdapterKind, ElciAdapter, INVOKE_UNAVAILABLE};
pub use registry::{component_registry, ComponentDescriptor};
