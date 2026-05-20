use crate::app::{format_metric, format_pct};
use perfgate_types::{
    CHECK_ID_BASELINE, CHECK_ID_BUDGET, CompareReceipt, FINDING_CODE_BASELINE_MISSING,
    FINDING_CODE_METRIC_FAIL, FINDING_CODE_METRIC_WARN, FindingData, MetricStatus, PerfgateReport,
    REPORT_SCHEMA_V1, ReportFinding, ReportSummary, RunReceipt, Severity, VERDICT_REASON_NO_BASELINE, Verdict, VerdictCounts, VerdictStatus,
};

pub(super) fn build_report(compare: &CompareReceipt) -> PerfgateReport {
    let mut findings = Vec::new();

    for (metric, delta) in &compare.deltas {
        let severity = match delta.status {
            MetricStatus::Pass | MetricStatus::Skip => continue,
            MetricStatus::Warn => Severity::Warn,
            MetricStatus::Fail => Severity::Fail,
        };

        let budget = compare.budgets.get(metric);
        let (threshold, direction) = budget
            .map(|b| (b.threshold, b.direction))
            .unwrap_or((0.20, metric.default_direction()));

        let code = match delta.status {
            MetricStatus::Warn => FINDING_CODE_METRIC_WARN.to_string(),
            MetricStatus::Fail => FINDING_CODE_METRIC_FAIL.to_string(),
            MetricStatus::Pass | MetricStatus::Skip => unreachable!(),
        };

        let metric_name = format_metric(*metric).to_string();
        let regression_pct = delta.regression * 100.0;
        let message = format!(
            "{} regression: {:.2}% (change: {}, threshold: {:.1}%)",
            metric_name,
            regression_pct,
            format_pct(delta.pct),
            threshold * 100.0
        );

        findings.push(ReportFinding {
            check_id: CHECK_ID_BUDGET.to_string(),
            code,
            severity,
            message,
            data: Some(FindingData {
                metric_name,
                baseline: delta.baseline,
                current: delta.current,
                regression_pct,
                threshold,
                direction,
            }),
        });
    }

    let summary = ReportSummary {
        pass_count: compare.verdict.counts.pass,
        warn_count: compare.verdict.counts.warn,
        fail_count: compare.verdict.counts.fail,
        skip_count: compare.verdict.counts.skip,
        total_count: compare.verdict.counts.pass
            + compare.verdict.counts.warn
            + compare.verdict.counts.fail
            + compare.verdict.counts.skip,
    };

    PerfgateReport {
        report_type: REPORT_SCHEMA_V1.to_string(),
        verdict: compare.verdict.clone(),
        compare: Some(compare.clone()),
        findings,
        summary,
        complexity: None,
        profile_path: None,
    }
}

/// Build a PerfgateReport for the case when there is no baseline.
///
/// Returns a report with Warn status (not Pass) to indicate that while
/// the check is non-blocking by default, no actual performance evaluation
/// occurred. The cockpit can highlight this as "baseline missing" rather
/// than incorrectly showing "pass".
pub(super) fn build_no_baseline_report(run: &RunReceipt) -> PerfgateReport {
    // Warn verdict: the sensor ran but no comparison was possible
    let verdict = Verdict {
        status: VerdictStatus::Warn,
        counts: VerdictCounts {
            pass: 0,
            warn: 1,
            fail: 0,
            skip: 0,
        },
        reasons: vec![VERDICT_REASON_NO_BASELINE.to_string()],
    };

    // Single finding for the baseline-missing condition
    let finding = ReportFinding {
        check_id: CHECK_ID_BASELINE.to_string(),
        code: FINDING_CODE_BASELINE_MISSING.to_string(),
        severity: Severity::Warn,
        message: format!(
            "No baseline found for bench '{}'; comparison skipped",
            run.bench.name
        ),
        data: None, // No metric data for structural findings
    };

    PerfgateReport {
        report_type: REPORT_SCHEMA_V1.to_string(),
        verdict,
        compare: None, // No synthetic compare receipt
        findings: vec![finding],
        summary: ReportSummary {
            pass_count: 0,
            warn_count: 1,
            fail_count: 0,
            skip_count: 0,
            total_count: 1,
        },
        complexity: None,
        profile_path: None,
    }
}

/// Detect high coefficient of variation in run receipt wall_ms.
///
/// Returns true if the wall_ms CV exceeds 0.30 (30%), which indicates
/// the benchmark environment is noisy and paired mode would give better results.
pub(super) fn detect_high_cv(run: &RunReceipt) -> bool {
    run.stats.wall_ms.cv().map(|cv| cv > 0.30).unwrap_or(false)
}

/// Render markdown for the case when there is no baseline.
pub(super) fn render_no_baseline_markdown(run: &RunReceipt, warnings: &[String]) -> String {
    let mut out = String::new();

    out.push_str("## perfgate: no baseline\n\n");
    out.push_str(&format!("**Bench:** `{}`\n\n", run.bench.name));
    out.push_str("No baseline found for comparison. This run will establish a new baseline.\n\n");

    out.push_str("### Current Results\n\n");
    out.push_str("| metric | value |\n");
    out.push_str("|---|---:|\n");
    out.push_str(&format!(
        "| `wall_ms` | {} ms |\n",
        run.stats.wall_ms.median
    ));

    if let Some(cpu) = &run.stats.cpu_ms {
        out.push_str(&format!("| `cpu_ms` | {} ms |\n", cpu.median));
    }

    if let Some(page_faults) = &run.stats.page_faults {
        out.push_str(&format!(
            "| `page_faults` | {} count |\n",
            page_faults.median
        ));
    }

    if let Some(ctx_switches) = &run.stats.ctx_switches {
        out.push_str(&format!(
            "| `ctx_switches` | {} count |\n",
            ctx_switches.median
        ));
    }

    if let Some(rss) = &run.stats.max_rss_kb {
        out.push_str(&format!("| `max_rss_kb` | {} KB |\n", rss.median));
    }

    if let Some(binary_bytes) = &run.stats.binary_bytes {
        out.push_str(&format!(
            "| `binary_bytes` | {} bytes |\n",
            binary_bytes.median
        ));
    }

    if let Some(throughput) = &run.stats.throughput_per_s {
        out.push_str(&format!(
            "| `throughput_per_s` | {:.3} /s |\n",
            throughput.median
        ));
    }

    if !warnings.is_empty() {
        out.push_str("\n**Warnings:**\n");
        for w in warnings {
            out.push_str(&format!("- {}\n", w));
        }
    }

    out
}

