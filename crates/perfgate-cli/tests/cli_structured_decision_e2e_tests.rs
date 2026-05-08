//! End-to-end fixture for the structured decision evidence path.

use predicates::prelude::*;
use std::fs;
use std::path::Path;
use tempfile::tempdir;

mod common;
use common::perfgate_cmd;

#[cfg(unix)]
fn success_command() -> Vec<&'static str> {
    vec!["true"]
}

#[cfg(windows)]
fn success_command() -> Vec<&'static str> {
    vec!["cmd", "/c", "exit", "0"]
}

fn command_toml_array(command: &[&str]) -> String {
    command
        .iter()
        .map(|part| format!("\"{part}\""))
        .collect::<Vec<_>>()
        .join(", ")
}

fn write_config(path: &Path) {
    fs::write(
        path,
        format!(
            r#"[defaults]
repeat = 1
warmup = 0
threshold = 10.0
warn_factor = 0.50
noise_threshold = 1.0
noise_policy = "warn"
baseline_dir = "baselines"
out_dir = "artifacts/perfgate"

[[bench]]
name = "parser"
command = [{}]

[[scenario]]
name = "release_workload"
weight = 1.0
bench = "parser"

[[tradeoff]]
name = "wall-clock-safety"
if_failed = "wall_ms"
downgrade_to = "warn"

[[tradeoff.require]]
metric = "wall_ms"
min_improvement_ratio = 1.01
"#,
            command_toml_array(&success_command())
        ),
    )
    .expect("write config");
}

#[test]
fn structured_decision_path_produces_scenario_and_tradeoff_receipts() {
    let temp_dir = tempdir().expect("create temp dir");
    let root = temp_dir.path();
    let config_path = root.join("perfgate.toml");
    write_config(&config_path);

    perfgate_cmd()
        .current_dir(root)
        .args(["check", "--config", "perfgate.toml", "--all"])
        .assert()
        .success()
        .stderr(predicate::str::contains("parser"));

    perfgate_cmd()
        .current_dir(root)
        .args(["baseline", "promote", "--config", "perfgate.toml", "--all"])
        .assert()
        .success()
        .stderr(predicate::str::contains("Promoted baseline for parser"));

    perfgate_cmd()
        .current_dir(root)
        .args([
            "check",
            "--config",
            "perfgate.toml",
            "--all",
            "--require-baseline",
        ])
        .assert()
        .success();

    let compare_path = root.join("artifacts/perfgate/parser/compare.json");
    assert!(compare_path.exists(), "check should write compare receipt");

    let scenario_path = root.join("artifacts/perfgate/scenario.json");
    perfgate_cmd()
        .current_dir(root)
        .args([
            "scenario",
            "evaluate",
            "--config",
            "perfgate.toml",
            "--out",
            "artifacts/perfgate/scenario.json",
        ])
        .assert()
        .success()
        .stderr(predicate::str::contains("Scenario receipt written"));

    let scenario: perfgate_types::ScenarioReceipt =
        serde_json::from_str(&fs::read_to_string(&scenario_path).expect("read scenario receipt"))
            .expect("scenario receipt should deserialize");
    assert_eq!(scenario.schema, "perfgate.scenario.v1");
    assert_eq!(scenario.scenario.name, "release_workload");
    assert_eq!(scenario.components.len(), 1);
    assert_eq!(scenario.components[0].benchmark.as_deref(), Some("parser"));
    assert!(scenario.weighted_deltas.contains_key("wall_ms"));

    let tradeoff_path = root.join("artifacts/perfgate/tradeoff.json");
    perfgate_cmd()
        .current_dir(root)
        .args([
            "tradeoff",
            "evaluate",
            "--config",
            "perfgate.toml",
            "--scenario",
            "artifacts/perfgate/scenario.json",
            "--out",
            "artifacts/perfgate/tradeoff.json",
        ])
        .assert()
        .success()
        .stderr(predicate::str::contains("Tradeoff receipt written"));

    let tradeoff: perfgate_types::TradeoffReceipt =
        serde_json::from_str(&fs::read_to_string(&tradeoff_path).expect("read tradeoff receipt"))
            .expect("tradeoff receipt should deserialize");
    assert_eq!(tradeoff.schema, "perfgate.tradeoff.v1");
    assert_eq!(tradeoff.scenario.as_deref(), Some("release_workload"));
    assert_eq!(tradeoff.configured_rules.len(), 1);
    assert_eq!(tradeoff.rules.len(), 1);
    assert_eq!(tradeoff.rules[0].name, "wall-clock-safety");
    assert_eq!(
        tradeoff.verdict.status,
        perfgate_types::VerdictStatus::Pass,
        "tradeoff command exit code should match the final pass verdict"
    );

    perfgate_cmd()
        .current_dir(root)
        .args([
            "md",
            "--tradeoff",
            "artifacts/perfgate/tradeoff.json",
            "--out",
            "artifacts/perfgate/decision.md",
        ])
        .assert()
        .success();
    let decision =
        fs::read_to_string(root.join("artifacts/perfgate/decision.md")).expect("read decision md");
    assert!(decision.contains("perfgate tradeoff: pass"));
    assert!(decision.contains("wall-clock-safety"));
}
