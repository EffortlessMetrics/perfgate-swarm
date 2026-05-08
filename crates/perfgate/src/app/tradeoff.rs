use crate::domain::budget::{aggregate_verdict, reason_token};
use perfgate_types::{
    Delta, HostInfo, Metric, MetricStatus, RunMeta, ScenarioReceipt, TRADEOFF_SCHEMA_V1, ToolInfo,
    TradeoffDecision, TradeoffDecisionStatus, TradeoffDowngrade, TradeoffReceipt,
    TradeoffRequirement, TradeoffRequirementOutcome, TradeoffRule, TradeoffRuleOutcome,
    VERDICT_REASON_TRADEOFF_MISSING_REQUIRED_METRIC, VERDICT_REASON_TRADEOFF_RULE_NOT_SATISFIED,
    Verdict,
};
use std::collections::BTreeMap;
use time::OffsetDateTime;
use uuid::Uuid;

#[derive(Debug)]
pub struct TradeoffEvaluateRequest {
    pub scenario: ScenarioReceipt,
    pub rules: Vec<TradeoffRule>,
    pub tool: ToolInfo,
}

#[derive(Debug)]
pub struct TradeoffEvaluateOutcome {
    pub receipt: TradeoffReceipt,
}

pub struct TradeoffUseCase;

impl TradeoffUseCase {
    pub fn evaluate(req: TradeoffEvaluateRequest) -> anyhow::Result<TradeoffEvaluateOutcome> {
        if req.rules.is_empty() {
            anyhow::bail!("no tradeoff rules provided");
        }

        let mut weighted_deltas = req.scenario.weighted_deltas.clone();
        let mut rule_outcomes = Vec::new();
        let mut accepted_reasons = Vec::new();
        let mut rejected_reasons = Vec::new();

        for rule in &req.rules {
            let outcome = evaluate_rule(rule, &weighted_deltas);
            if outcome.accepted {
                if let Some(delta) = weighted_deltas.get_mut(rule.if_failed.as_str()) {
                    delta.status = downgrade_status(rule.downgrade_to);
                }
                accepted_reasons.push(format!("tradeoff_{}_applied", rule.name));
            } else if matches!(outcome.status, TradeoffDecisionStatus::Rejected) {
                if outcome
                    .requirements
                    .iter()
                    .any(|requirement| requirement.observed_change.is_none())
                {
                    rejected_reasons
                        .push(VERDICT_REASON_TRADEOFF_MISSING_REQUIRED_METRIC.to_string());
                } else {
                    rejected_reasons.push(VERDICT_REASON_TRADEOFF_RULE_NOT_SATISFIED.to_string());
                }
            }
            rule_outcomes.push(outcome);
        }

        let mut verdict = verdict_from_weighted_deltas(&weighted_deltas);
        verdict
            .reasons
            .extend(non_pass_reason_tokens(&weighted_deltas));
        for reason in accepted_reasons.iter().chain(rejected_reasons.iter()) {
            push_unique(&mut verdict.reasons, reason.clone());
        }

        let accepted = rule_outcomes.iter().any(|outcome| outcome.accepted);
        let decision = TradeoffDecision {
            accepted_tradeoff: accepted,
            status: metric_status_from_verdict(&verdict),
            reason: decision_reason(accepted, &rule_outcomes, &verdict),
        };

        let receipt = TradeoffReceipt {
            schema: TRADEOFF_SCHEMA_V1.to_string(),
            tool: req.tool,
            run: make_run_meta(),
            scenario: Some(req.scenario.scenario.name),
            baseline_ref: req.scenario.baseline_ref,
            current_ref: req.scenario.current_ref,
            configured_rules: req.rules,
            rules: rule_outcomes,
            probes: Vec::new(),
            weighted_deltas,
            decision,
            verdict,
            warnings: req.scenario.warnings,
        };

        Ok(TradeoffEvaluateOutcome { receipt })
    }
}

fn evaluate_rule(
    rule: &TradeoffRule,
    weighted_deltas: &BTreeMap<String, Delta>,
) -> TradeoffRuleOutcome {
    let Some(target) = weighted_deltas.get(rule.if_failed.as_str()) else {
        return TradeoffRuleOutcome {
            name: rule.name.clone(),
            status: TradeoffDecisionStatus::NotEvaluated,
            accepted: false,
            downgrade_to: None,
            reason: Some(format!(
                "failed metric '{}' is not present",
                rule.if_failed.as_str()
            )),
            requirements: evaluate_requirements(&rule.require, weighted_deltas),
        };
    };

    if target.status != MetricStatus::Fail {
        return TradeoffRuleOutcome {
            name: rule.name.clone(),
            status: TradeoffDecisionStatus::NotEvaluated,
            accepted: false,
            downgrade_to: None,
            reason: Some(format!(
                "metric '{}' is not failing",
                rule.if_failed.as_str()
            )),
            requirements: evaluate_requirements(&rule.require, weighted_deltas),
        };
    }

    let requirements = evaluate_requirements(&rule.require, weighted_deltas);
    let accepted =
        !requirements.is_empty() && requirements.iter().all(|requirement| requirement.satisfied);

    TradeoffRuleOutcome {
        name: rule.name.clone(),
        status: if accepted {
            TradeoffDecisionStatus::Accepted
        } else {
            TradeoffDecisionStatus::Rejected
        },
        accepted,
        downgrade_to: accepted.then_some(rule.downgrade_to),
        reason: Some(if accepted {
            "all required compensating improvements were satisfied".to_string()
        } else {
            "one or more required compensating improvements were not satisfied".to_string()
        }),
        requirements,
    }
}

fn evaluate_requirements(
    requirements: &[TradeoffRequirement],
    weighted_deltas: &BTreeMap<String, Delta>,
) -> Vec<TradeoffRequirementOutcome> {
    requirements
        .iter()
        .map(|requirement| {
            let metric_key = requirement.metric.as_str();
            let Some(delta) = weighted_deltas.get(metric_key) else {
                return TradeoffRequirementOutcome {
                    metric: metric_key.to_string(),
                    probe: None,
                    required_change: required_change(requirement),
                    observed_change: None,
                    satisfied: false,
                    status: MetricStatus::Fail,
                    reason: Some("required metric missing".to_string()),
                };
            };

            let observed_ratio = improvement_ratio(delta, requirement.metric);
            let satisfied = observed_ratio
                .map(|ratio| ratio >= requirement.min_improvement_ratio)
                .unwrap_or(false);

            TradeoffRequirementOutcome {
                metric: metric_key.to_string(),
                probe: None,
                required_change: required_change(requirement),
                observed_change: Some(delta.pct),
                satisfied,
                status: if satisfied {
                    MetricStatus::Pass
                } else {
                    MetricStatus::Fail
                },
                reason: (!satisfied).then_some(format!(
                    "requires improvement ratio >= {:.6}",
                    requirement.min_improvement_ratio
                )),
            }
        })
        .collect()
}

fn required_change(requirement: &TradeoffRequirement) -> f64 {
    match requirement.metric.default_direction() {
        perfgate_types::Direction::Higher => requirement.min_improvement_ratio - 1.0,
        perfgate_types::Direction::Lower => (1.0 / requirement.min_improvement_ratio) - 1.0,
    }
}

fn improvement_ratio(delta: &Delta, metric: Metric) -> Option<f64> {
    match metric.default_direction() {
        perfgate_types::Direction::Higher => Some(delta.ratio),
        perfgate_types::Direction::Lower => Some(if delta.current <= 0.0 {
            f64::INFINITY
        } else {
            delta.baseline / delta.current
        }),
    }
}

fn downgrade_status(downgrade: TradeoffDowngrade) -> MetricStatus {
    match downgrade {
        TradeoffDowngrade::Warn => MetricStatus::Warn,
        TradeoffDowngrade::Pass => MetricStatus::Pass,
    }
}

fn verdict_from_weighted_deltas(weighted_deltas: &BTreeMap<String, Delta>) -> Verdict {
    let statuses: Vec<_> = weighted_deltas.values().map(|delta| delta.status).collect();
    aggregate_verdict(&statuses)
}

fn non_pass_reason_tokens(weighted_deltas: &BTreeMap<String, Delta>) -> Vec<String> {
    weighted_deltas
        .iter()
        .filter_map(|(metric_key, delta)| {
            if matches!(delta.status, MetricStatus::Pass) {
                return None;
            }
            Metric::parse_key(metric_key).map(|metric| reason_token(metric, delta.status))
        })
        .collect()
}

fn metric_status_from_verdict(verdict: &Verdict) -> MetricStatus {
    match verdict.status {
        perfgate_types::VerdictStatus::Pass => MetricStatus::Pass,
        perfgate_types::VerdictStatus::Warn => MetricStatus::Warn,
        perfgate_types::VerdictStatus::Fail => MetricStatus::Fail,
        perfgate_types::VerdictStatus::Skip => MetricStatus::Skip,
    }
}

fn decision_reason(
    accepted: bool,
    rule_outcomes: &[TradeoffRuleOutcome],
    verdict: &Verdict,
) -> String {
    if accepted && let Some(rule) = rule_outcomes.iter().find(|outcome| outcome.accepted) {
        return format!("tradeoff '{}' accepted", rule.name);
    }

    if matches!(verdict.status, perfgate_types::VerdictStatus::Fail) {
        "no configured tradeoff accepted the failing metric".to_string()
    } else {
        "no failing metric required a tradeoff decision".to_string()
    }
}

fn push_unique(values: &mut Vec<String>, value: String) {
    if !values.contains(&value) {
        values.push(value);
    }
}

fn make_run_meta() -> RunMeta {
    let now = OffsetDateTime::now_utc();
    let timestamp = now
        .format(&time::format_description::well_known::Rfc3339)
        .unwrap_or_else(|_| "1970-01-01T00:00:00Z".to_string());

    RunMeta {
        id: Uuid::new_v4().to_string(),
        started_at: timestamp.clone(),
        ended_at: timestamp,
        host: HostInfo {
            os: std::env::consts::OS.to_string(),
            arch: std::env::consts::ARCH.to_string(),
            cpu_count: None,
            memory_bytes: None,
            hostname_hash: None,
        },
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use perfgate_types::{SCENARIO_SCHEMA_V1, ScenarioMeta, VerdictCounts, VerdictStatus};

    fn scenario_receipt(wall_current: f64, memory_status: MetricStatus) -> ScenarioReceipt {
        ScenarioReceipt {
            schema: SCENARIO_SCHEMA_V1.to_string(),
            tool: ToolInfo {
                name: "perfgate".to_string(),
                version: "0.16.0".to_string(),
            },
            run: make_run_meta(),
            scenario: ScenarioMeta {
                name: "release_workload".to_string(),
                weight: 1.0,
                description: None,
                command: None,
            },
            baseline_ref: None,
            current_ref: None,
            components: Vec::new(),
            weighted_deltas: BTreeMap::from([
                (
                    "wall_ms".to_string(),
                    delta(100.0, wall_current, MetricStatus::Pass),
                ),
                ("max_rss_kb".to_string(), delta(100.0, 120.0, memory_status)),
            ]),
            verdict: Verdict {
                status: VerdictStatus::Fail,
                counts: VerdictCounts {
                    pass: 1,
                    warn: 0,
                    fail: 1,
                    skip: 0,
                },
                reasons: vec!["max_rss_kb_fail".to_string()],
            },
            warnings: Vec::new(),
        }
    }

    fn delta(baseline: f64, current: f64, status: MetricStatus) -> Delta {
        Delta {
            baseline,
            current,
            ratio: current / baseline,
            pct: (current - baseline) / baseline,
            regression: if current > baseline {
                (current - baseline) / baseline
            } else {
                0.0
            },
            cv: None,
            noise_threshold: None,
            statistic: perfgate_types::MetricStatistic::Median,
            significance: None,
            status,
        }
    }

    fn memory_for_speed_rule(downgrade_to: TradeoffDowngrade) -> TradeoffRule {
        TradeoffRule {
            name: "memory_for_speed".to_string(),
            if_failed: Metric::MaxRssKb,
            require: vec![TradeoffRequirement {
                metric: Metric::WallMs,
                min_improvement_ratio: 1.10,
            }],
            downgrade_to,
        }
    }

    #[test]
    fn tradeoff_evaluate_accepts_satisfied_rule() {
        let outcome = TradeoffUseCase::evaluate(TradeoffEvaluateRequest {
            scenario: scenario_receipt(80.0, MetricStatus::Fail),
            rules: vec![memory_for_speed_rule(TradeoffDowngrade::Warn)],
            tool: ToolInfo {
                name: "perfgate".to_string(),
                version: "0.16.0".to_string(),
            },
        })
        .expect("evaluate tradeoff");

        let receipt = outcome.receipt;
        assert_eq!(receipt.schema, TRADEOFF_SCHEMA_V1);
        assert!(receipt.decision.accepted_tradeoff);
        assert_eq!(
            receipt.weighted_deltas["max_rss_kb"].status,
            MetricStatus::Warn
        );
        assert_eq!(receipt.verdict.status, VerdictStatus::Warn);
        assert_eq!(receipt.rules[0].status, TradeoffDecisionStatus::Accepted);
        assert_eq!(
            receipt.rules[0].requirements[0].observed_change,
            Some(-0.20)
        );
    }

    #[test]
    fn tradeoff_evaluate_rejects_unsatisfied_rule() {
        let outcome = TradeoffUseCase::evaluate(TradeoffEvaluateRequest {
            scenario: scenario_receipt(96.0, MetricStatus::Fail),
            rules: vec![memory_for_speed_rule(TradeoffDowngrade::Pass)],
            tool: ToolInfo {
                name: "perfgate".to_string(),
                version: "0.16.0".to_string(),
            },
        })
        .expect("evaluate tradeoff");

        let receipt = outcome.receipt;
        assert!(!receipt.decision.accepted_tradeoff);
        assert_eq!(
            receipt.weighted_deltas["max_rss_kb"].status,
            MetricStatus::Fail
        );
        assert_eq!(receipt.verdict.status, VerdictStatus::Fail);
        assert_eq!(receipt.rules[0].status, TradeoffDecisionStatus::Rejected);
        assert!(
            receipt
                .verdict
                .reasons
                .contains(&VERDICT_REASON_TRADEOFF_RULE_NOT_SATISFIED.to_string())
        );
    }
}
