//! Helpers for invoking Python/SciPy as a live numerical oracle.
//!
//! Normal Honeyeater tests should prefer committed reference vectors generated
//! by `tools/oracle-gen`. This module provides a lightweight subprocess helper
//! for development-time cross-validation where generating and committing a
//! reference vector is unnecessary.

use std::io::Write;
use std::process::{Command, Stdio};

/// Run a Python script and return its stdout.
///
/// The Python interpreter may be overridden with the `PYTHON` environment
/// variable. Otherwise, `python` is used from `PATH`.
///
/// The script is passed to Python through stdin. If Python exits with a
/// non-zero status, its stderr is returned as an error.
///
/// # Errors
///
/// Returns an error if Python cannot be started, the script cannot be written
/// to stdin, the process cannot be waited on, Python exits unsuccessfully, or
/// its stdout is not valid UTF-8.
pub fn run(script: &str) -> Result<String, Box<dyn std::error::Error>> {
    let python = std::env::var("PYTHON").unwrap_or_else(|_| "python".to_string());

    let mut child = Command::new(python)
        .arg("-")
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()?;

    child
        .stdin
        .as_mut()
        .ok_or("failed to open Python stdin")?
        .write_all(script.as_bytes())?;

    let output = child.wait_with_output()?;

    if !output.status.success() {
        return Err(format!(
            "Python oracle failed:\n{}",
            String::from_utf8_lossy(&output.stderr)
        )
        .into());
    }

    Ok(String::from_utf8(output.stdout)?)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn runs_python_script() {
        let output = run("print(2 + 2)").unwrap();
        assert_eq!(output.trim(), "4");
    }

    #[test]
    fn reports_python_failure() {
        let result = run("raise RuntimeError('oracle failure')");
        assert!(result.is_err());
    }
}
