use super::availability::{has_cargo_flamegraph_subcommand, has_inferno, is_command_available};
use super::detected::DetectedProfiler;

/// Detect the best available profiler on the current system.
///
/// Priority order:
/// 1. `perf` + `inferno-flamegraph` (Linux)
/// 2. `dtrace` + `inferno-flamegraph` (macOS)
/// 3. `cargo flamegraph` (any platform)
/// 4. `None` if nothing is available
pub fn detect_profiler() -> Option<DetectedProfiler> {
    if cfg!(target_os = "linux") && is_command_available("perf") && has_inferno() {
        return Some(DetectedProfiler::Perf);
    }

    if cfg!(target_os = "macos") && is_command_available("dtrace") && has_inferno() {
        return Some(DetectedProfiler::Dtrace);
    }

    if is_command_available("cargo-flamegraph") || has_cargo_flamegraph_subcommand() {
        return Some(DetectedProfiler::CargoFlamegraph);
    }

    None
}
