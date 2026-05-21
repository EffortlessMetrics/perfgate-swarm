//! Profiler detection: probes the system for available profiling tools.

mod availability;
mod detected;
mod selection;

pub use detected::DetectedProfiler;
pub use selection::detect_profiler;

#[cfg(test)]
mod tests {
    use super::*;
    use crate::runtime::profile::detect::availability::is_command_available;

    #[test]
    fn detected_profiler_name_is_human_readable() {
        assert_eq!(DetectedProfiler::Perf.name(), "perf + inferno");
        assert_eq!(DetectedProfiler::Dtrace.name(), "dtrace + inferno");
        assert_eq!(DetectedProfiler::CargoFlamegraph.name(), "cargo-flamegraph");
    }

    #[test]
    fn is_command_available_returns_false_for_nonexistent() {
        assert!(!is_command_available("perfgate-nonexistent-tool-xyz-12345"));
    }

    #[test]
    fn detect_profiler_does_not_panic() {
        // Just verify detection completes without panic on any platform.
        let _result = detect_profiler();
    }

    #[test]
    fn detected_profiler_variants_are_distinct() {
        assert_ne!(DetectedProfiler::Perf, DetectedProfiler::Dtrace);
        assert_ne!(DetectedProfiler::Perf, DetectedProfiler::CargoFlamegraph);
        assert_ne!(DetectedProfiler::Dtrace, DetectedProfiler::CargoFlamegraph);
    }
}
