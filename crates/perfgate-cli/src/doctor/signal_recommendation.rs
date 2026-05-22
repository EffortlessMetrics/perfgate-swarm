use super::SignalRecommendation;

pub(super) struct SignalRecommendationInput {
    pub(super) baseline_found: bool,
    pub(super) baseline_remote: bool,
    pub(super) compare_found: bool,
    pub(super) samples: usize,
    pub(super) cv: Option<f64>,
    pub(super) host_mismatch: bool,
    pub(super) baseline_age_days: Option<i64>,
}

pub(super) fn decide_signal_recommendation(
    input: SignalRecommendationInput,
    stale_baseline_days: i64,
    high_noise_cv: f64,
    mature_sample_limit: usize,
) -> SignalRecommendation {
    if missing_baseline(&input) {
        return SignalRecommendation::NoDecisionYet;
    }
    if host_mismatch_detected(&input) {
        return SignalRecommendation::CheckHostMismatch;
    }
    if baseline_is_stale(&input, stale_baseline_days) {
        return SignalRecommendation::RefreshBaseline;
    }
    if is_high_noise(&input, high_noise_cv) {
        return SignalRecommendation::UsePairedMode;
    }
    if needs_more_samples(&input, mature_sample_limit) {
        return SignalRecommendation::IncreaseSamples;
    }
    if advisory_only(&input) {
        return SignalRecommendation::AdvisoryOnly;
    }
    SignalRecommendation::SafeToGate
}

fn missing_baseline(input: &SignalRecommendationInput) -> bool {
    !input.baseline_found && !input.baseline_remote
}

fn host_mismatch_detected(input: &SignalRecommendationInput) -> bool {
    input.host_mismatch
}

fn baseline_is_stale(input: &SignalRecommendationInput, stale_baseline_days: i64) -> bool {
    input
        .baseline_age_days
        .is_some_and(|days| days > stale_baseline_days)
}

fn is_high_noise(input: &SignalRecommendationInput, high_noise_cv: f64) -> bool {
    input.cv.is_some_and(|cv| cv > high_noise_cv)
}

fn needs_more_samples(input: &SignalRecommendationInput, mature_sample_limit: usize) -> bool {
    input.samples < mature_sample_limit
}

fn advisory_only(input: &SignalRecommendationInput) -> bool {
    !input.compare_found || input.baseline_remote
}
