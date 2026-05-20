use perfgate_types::{
    CHECK_ID_COMPLEXITY, CompareReceipt, ComplexityGateResult, ComplexityGateStatus,
    FINDING_CODE_COMPLEXITY_FAIL, FINDING_CODE_COMPLEXITY_INCONCLUSIVE, PerfgateReport,
    ReportFinding, Severity, VerdictStatus,
};

pub(super) fn apply_complexity_gate(
    mut compare: Option<CompareReceipt>,
    mut report: PerfgateReport,
    complexity: Option<ComplexityGateResult>,
) -> (Option<CompareReceipt>, PerfgateReport) {
    let Some(complexity) = complexity else {
        return (compare, report);
    };

    let token = complexity.reason.clone();
    match complexity.status {
        ComplexityGateStatus::Pass => {
            report.summary.pass_count += 1;
        }
        ComplexityGateStatus::Inconclusive => {
            report.summary.warn_count += 1;
            if let Some(token) = &token {
                report.verdict.reasons.push(token.clone());
            }
            report.findings.push(ReportFinding {
                check_id: CHECK_ID_COMPLEXITY.to_string(),
                code: FINDING_CODE_COMPLEXITY_INCONCLUSIVE.to_string(),
                severity: Severity::Warn,
                message: complexity.message.clone(),
                data: None,
            });
        }
        ComplexityGateStatus::Fail => {
            report.summary.fail_count += 1;
            if let Some(token) = &token {
                report.verdict.reasons.push(token.clone());
            }
            report.findings.push(ReportFinding {
                check_id: CHECK_ID_COMPLEXITY.to_string(),
                code: FINDING_CODE_COMPLEXITY_FAIL.to_string(),
                severity: Severity::Fail,
                message: complexity.message.clone(),
                data: None,
            });
        }
    }
    report.summary.total_count =
        report.summary.pass_count + report.summary.warn_count + report.summary.fail_count;
    report.verdict.counts.pass = report.summary.pass_count;
    report.verdict.counts.warn = report.summary.warn_count;
    report.verdict.counts.fail = report.summary.fail_count;
    report.verdict.status = if report.summary.fail_count > 0 {
        VerdictStatus::Fail
    } else if report.summary.warn_count > 0 {
        VerdictStatus::Warn
    } else {
        VerdictStatus::Pass
    };
    report.complexity = Some(complexity.clone());

    if let Some(compare_receipt) = compare.as_mut() {
        match complexity.status {
            ComplexityGateStatus::Pass => compare_receipt.verdict.counts.pass += 1,
            ComplexityGateStatus::Inconclusive => compare_receipt.verdict.counts.warn += 1,
            ComplexityGateStatus::Fail => compare_receipt.verdict.counts.fail += 1,
        }
        if let Some(token) = token {
            compare_receipt.verdict.reasons.push(token);
        }
        compare_receipt.verdict.status = if compare_receipt.verdict.counts.fail > 0 {
            VerdictStatus::Fail
        } else if compare_receipt.verdict.counts.warn > 0 {
            VerdictStatus::Warn
        } else {
            VerdictStatus::Pass
        };
    }

    (compare, report)
}

pub(super) fn median(values: &mut [f64]) -> Option<f64> {
    if values.is_empty() {
        return None;
    }
    values.sort_by(f64::total_cmp);
    let mid = values.len() / 2;
    if values.len().is_multiple_of(2) {
        Some((values[mid - 1] + values[mid]) / 2.0)
    } else {
        Some(values[mid])
    }
}
