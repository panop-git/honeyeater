//! Live scipy cross-validation via subprocess.
//!
//! For kernels where committing a `.npy` reference vector is overkill (e.g.
//! a one-off check that a 32-tap window matches scipy's output up to
//! floating-point slop), this helper shells out to a Python interpreter
//! with scipy installed and returns the result as a parseable string or
//! native array.
//!
//! # Phase 0 status
//!
//! Stub. The signature below defines the contract; implementation is
//! deferred to the first kernel that needs live cross-validation rather
//! than a committed vector.
//!
//! When implemented, the helper should:
//!
//! - Look up the Python interpreter via `PYTHON` env var, falling back to
//!   `python3` on PATH.
//! - Pass the user's script via stdin (not as a temp file — avoids cleanup
//!   races).
//! - Pin scipy/numpy versions in `tools/oracle-gen/requirements.txt` and
//!   verify the installed versions match before running. Mismatches are a
//!   test failure, not a silent run.
//! - Use a JSON output convention so the helper can deserialise without
//!   parsing free-form text.
//! - Time out after some reasonable bound (60 s default; configurable).
//! - Skip — not fail — when no interpreter is available, so contributors
//!   without Python installed can still run the bulk of the suite. The
//!   scipy-using tests should be marked clearly so the skip is visible in
//!   the test report.

/// Run a Python script via the scipy-enabled interpreter and return its
/// stdout.
///
/// The script is responsible for emitting parseable output (typically a
/// JSON object) that the caller deserialises.
///
/// # Panics
///
/// Stub: always panics.
#[must_use]
pub fn run(_script: &str) -> String {
    unimplemented!(
        "honeyeater_test::scipy::run is a Phase 0 stub; \
         implementation deferred to first kernel that needs live scipy cross-validation"
    );
}
