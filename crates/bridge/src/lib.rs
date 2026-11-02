//! UI bindings in one library: `flutter_rust_bridge` for Dart, `UniFFI` for
//! Kotlin and Swift, so a mobile app process holds exactly one engine.
//! Spec: docs/specs/architecture.md (Components, Process model).

#[cfg(target_os = "android")]
#[allow(unsafe_code)]
mod android;
pub mod api;
// Generated FFI glue is the only `unsafe` in the workspace. The generator's
// style isn't ours, so lints are off for its output.
#[allow(unsafe_code, clippy::all, clippy::pedantic, clippy::unwrap_used)]
mod frb_generated;
// The daemon link (plain Rust). Task 4 exposes it through FRB; until then nothing outside the
// tests calls it.
#[allow(dead_code, reason = "consumed by the FRB surface in Phase 3b Task 4")]
mod link;
#[allow(unsafe_code)]
mod native;

use std::sync::atomic::{AtomicU64, Ordering};

uniffi::setup_scaffolding!("lemusync");

/// Calls through either binding. SP1 uses it to prove both bindings reach
/// the same instance.
static CALLS: AtomicU64 = AtomicU64::new(0);

/// Counts one call and returns the new total.
fn count_call() -> u64 {
    CALLS.fetch_add(1, Ordering::SeqCst) + 1
}
