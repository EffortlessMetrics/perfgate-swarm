use std::collections::BTreeMap;

use perfgate_client::DecisionRecord;

#[derive(Default)]
struct DecisionDebtAreaSummary {
    accepted_count: u32,
    review_required_count: u32,
    rule_counts: BTreeMap<String, u32>,
    max_cap_used: Option<f64>,
    max_accepted_delta: Option<DecisionDebtMetricDelta>,
}

#[derive(Clone)]
struct DecisionDebtMetricDelta {
    metric: String,
    regression: f64,
}

pub fn print_decision_debt_summary(project: &str, days: u32, records: &[&DecisionRecord]) {
    let mut areas: BTreeMap<String, DecisionDebtAreaSummary> = BTreeMap::new();
    let mut accepted_total = 0_u32;
    let mut review_required_total = 0_u32;

    for record in records {
        if record.accepted_rules.is_empty() {
            continue;
        }

        let area_name = record
            .scenario
            .clone()
            .unwrap_or_else(|| "unspecified".to_string());
        let area = areas.entry(area_name).or_default();
        area.accepted_count += 1;
        accepted_total += 1;

        if record.review_required {
            area.review_required_count += 1;
            review_required_total += 1;
        }

        for rule in &record.accepted_rules {
            *area.rule_counts.entry(rule.clone()).or_insert(0) += 1;
        }

        if let Some(cap_used) = decision_record_max_cap_used(record) {
            area.max_cap_used = Some(area.max_cap_used.map_or(cap_used, |current| {
                if cap_used > current {
                    cap_used
                } else {
                    current
                }
            }));
        }

        if let Some(delta) = decision_record_max_accepted_delta(record) {
            area.max_accepted_delta = Some(area.max_accepted_delta.as_ref().map_or_else(
                || delta.clone(),
                |current| max_metric_delta(current, &delta),
            ));
        }
    }

    let window = if days == 0 {
        "all fetched records".to_string()
    } else {
        format!("last {days} days")
    };

    println!(
        "Decision debt for {} ({}, {} records scanned):",
        project,
        window,
        records.len()
    );
    println!("Accepted tradeoff records: {accepted_total}");
    println!("Review-required accepted records: {review_required_total}");

    if areas.is_empty() {
        println!("\nNo accepted tradeoffs found.");
        return;
    }

    println!();
    println!(
        "{:<24} {:>5} {:>6} {:>8} {:>14} {:>11}  common rule",
        "area", "count", "review", "cap used", "accepted delta", "budget used"
    );

    for (area, summary) in areas {
        println!(
            "{:<24} {:>5} {:>6} {:>8} {:>14} {:>11}  {}",
            area,
            summary.accepted_count,
            summary.review_required_count,
            format_cap_used(summary.max_cap_used),
            format_accepted_delta(summary.max_accepted_delta.as_ref()),
            format_budget_headroom_used(None),
            most_common_rule(&summary.rule_counts)
        );
    }
}

pub fn print_decision_record(label: &str, record: &DecisionRecord) {
    let scenario = record.scenario.as_deref().unwrap_or("unspecified");
    let git_ref = record.git_ref.as_deref().unwrap_or("unknown");
    let review = if record.review_required {
        "review-required"
    } else {
        "no-review"
    };
    let accepted_rules = if record.accepted_rules.is_empty() {
        "none".to_string()
    } else {
        record.accepted_rules.join(",")
    };

    println!(
        "{} {} scenario={} status={} verdict={} {} git_ref={} accepted_rules={} created_at={}",
        label,
        record.id,
        scenario,
        record.status.as_str(),
        record.verdict.as_str(),
        review,
        git_ref,
        accepted_rules,
        record.created_at
    );
}

fn decision_record_max_cap_used(record: &DecisionRecord) -> Option<f64> {
    record
        .tradeoff_receipt
        .rules
        .iter()
        .filter(|rule| rule.accepted)
        .flat_map(|rule| rule.allowances.iter())
        .filter_map(|allowance| {
            if allowance.max_regression <= 0.0 {
                return None;
            }
            let observed = allowance.observed_regression?;
            Some((observed.max(0.0) / allowance.max_regression).max(0.0))
        })
        .reduce(f64::max)
}

fn decision_record_max_accepted_delta(record: &DecisionRecord) -> Option<DecisionDebtMetricDelta> {
    record
        .tradeoff_receipt
        .rules
        .iter()
        .filter(|rule| rule.accepted)
        .filter_map(|rule| {
            let configured = record
                .tradeoff_receipt
                .configured_rules
                .iter()
                .find(|configured| configured.name == rule.name)?;
            let metric = configured.if_failed.as_str();
            let delta = record.tradeoff_receipt.weighted_deltas.get(metric)?;
            if delta.regression <= 0.0 {
                return None;
            }
            Some(DecisionDebtMetricDelta {
                metric: metric.to_string(),
                regression: delta.regression,
            })
        })
        .reduce(|current, next| max_metric_delta(&current, &next))
}

fn max_metric_delta(
    left: &DecisionDebtMetricDelta,
    right: &DecisionDebtMetricDelta,
) -> DecisionDebtMetricDelta {
    if right.regression > left.regression {
        right.clone()
    } else {
        left.clone()
    }
}

fn format_cap_used(value: Option<f64>) -> String {
    value
        .map(|value| format!("{:.0}%", value * 100.0))
        .unwrap_or_else(|| "n/a".to_string())
}

fn format_accepted_delta(value: Option<&DecisionDebtMetricDelta>) -> String {
    value
        .map(|delta| format!("{} +{:.1}%", delta.metric, delta.regression * 100.0))
        .unwrap_or_else(|| "n/a".to_string())
}

fn format_budget_headroom_used(value: Option<f64>) -> String {
    value
        .map(|value| format!("{:.0}%", value * 100.0))
        .unwrap_or_else(|| "n/a".to_string())
}

fn most_common_rule(rule_counts: &BTreeMap<String, u32>) -> String {
    rule_counts
        .iter()
        .max_by(|(left_rule, left_count), (right_rule, right_count)| {
            left_count
                .cmp(right_count)
                .then_with(|| right_rule.cmp(left_rule))
        })
        .map(|(rule, count)| format!("{rule} ({count})"))
        .unwrap_or_else(|| "none".to_string())
}
