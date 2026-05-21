use crate::runtime::profile::profiler::{
    CargoFlamegraphProfiler, DtraceProfiler, PerfProfiler, ProfileError, ProfileRequest,
    ProfileResult, Profiler,
};

/// A profiler that was detected on the current system.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DetectedProfiler {
    /// Linux `perf` with `inferno-flamegraph` for SVG rendering.
    Perf,
    /// macOS `dtrace` with `inferno-flamegraph` for SVG rendering.
    Dtrace,
    /// Cross-platform `cargo flamegraph` wrapper.
    CargoFlamegraph,
}

impl DetectedProfiler {
    /// Human-readable name of the profiler.
    pub fn name(&self) -> &'static str {
        match self {
            DetectedProfiler::Perf => "perf + inferno",
            DetectedProfiler::Dtrace => "dtrace + inferno",
            DetectedProfiler::CargoFlamegraph => "cargo-flamegraph",
        }
    }

    /// Capture a flamegraph using this profiler.
    pub fn capture(&self, request: &ProfileRequest) -> Result<ProfileResult, ProfileError> {
        match self {
            DetectedProfiler::Perf => PerfProfiler.capture(request),
            DetectedProfiler::Dtrace => DtraceProfiler.capture(request),
            DetectedProfiler::CargoFlamegraph => CargoFlamegraphProfiler.capture(request),
        }
    }
}
