use std::process::{Command, Stdio};

/// Check whether a binary is available on PATH.
pub(super) fn is_command_available(name: &str) -> bool {
    Command::new(name)
        .arg("--version")
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status()
        .is_ok()
}

/// Check whether `inferno-flamegraph` is available (part of the `inferno` crate).
pub(super) fn has_inferno() -> bool {
    is_command_available("inferno-flamegraph")
}

/// Check whether `cargo flamegraph` subcommand is available.
pub(super) fn has_cargo_flamegraph_subcommand() -> bool {
    Command::new("cargo")
        .args(["flamegraph", "--help"])
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status()
        .is_ok_and(|s| s.success())
}
