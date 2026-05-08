//! Integration tests for `perfgate tradeoff`.

mod common;

use common::perfgate_cmd;
use predicates::prelude::*;
use serde_json::{Value, json};
use std::fs;
use std::path::Path;
use tempfile::tempdir;

#[test]
fn test_tradeoff_evaluate_writes_tradeoff_receipt() {
    let temp_dir = tempdir().expect("failed to create temp dir");
    let config_path = temp_dir.path().join("perfgate.toml");
    let scenario_path = temp_dir.path().join("scenario.json");
    let output_path = temp_dir.path().join("tradeoff.json");

    write_tradeoff_config(&config_path, 1.10, "warn");
    write_scenario_receipt(&scenario_path, 80.0, "fail");

    perfgate_cmd()
        .arg("tradeoff")
        .arg("evaluate")
        .arg("--config")
        .arg(&config_path)
        .arg("--scenario")
        .arg(&scenario_path)
        .arg("--out")
        .arg(&output_path)
        .assert()
        .success()
        .stderr(predicate::str::contains("Tradeoff receipt written"));

    let receipt: Value = serde_json::from_str(
        &fs::read_to_string(&output_path).expect("failed to read tradeoff receipt"),
    )
    .expect("tradeoff receipt should be JSON");
    let typed: perfgate_types::TradeoffReceipt =
        serde_json::from_value(receipt.clone()).expect("tradeoff receipt should deserialize");

    assert_eq!(typed.schema, "perfgate.tradeoff.v1");
    assert_eq!(typed.scenario.as_deref(), Some("release_workload"));
    assert!(typed.decision.accepted_tradeoff);
    assert_eq!(typed.verdict.status, perfgate_types::VerdictStatus::Warn);
    assert_eq!(
        receipt["weighted_deltas"]["max_rss_kb"]["status"].as_str(),
        Some("warn")
    );
    assert_eq!(receipt["rules"][0]["status"].as_str(), Some("accepted"));
    assert_eq!(receipt["rules"][0]["requirements"][0]["satisfied"], true);
}

#[test]
fn test_tradeoff_evaluate_fails_when_rule_not_satisfied() {
    let temp_dir = tempdir().expect("failed to create temp dir");
    let config_path = temp_dir.path().join("perfgate.toml");
    let scenario_path = temp_dir.path().join("scenario.json");
    let output_path = temp_dir.path().join("tradeoff.json");

    write_tradeoff_config(&config_path, 1.20, "pass");
    write_scenario_receipt(&scenario_path, 96.0, "fail");

    perfgate_cmd()
        .arg("tradeoff")
        .arg("evaluate")
        .arg("--config")
        .arg(&config_path)
        .arg("--scenario")
        .arg(&scenario_path)
        .arg("--out")
        .arg(&output_path)
        .assert()
        .code(2);

    let receipt: Value = serde_json::from_str(
        &fs::read_to_string(&output_path).expect("failed to read tradeoff receipt"),
    )
    .expect("tradeoff receipt should be JSON");

    assert_eq!(receipt["schema"], "perfgate.tradeoff.v1");
    assert_eq!(receipt["decision"]["accepted_tradeoff"], false);
    assert_eq!(receipt["rules"][0]["status"], "rejected");
    assert!(
        receipt["verdict"]["reasons"]
            .as_array()
            .expect("reasons array")
            .iter()
            .any(|reason| reason == "tradeoff_rule_not_satisfied")
    );
}

#[test]
fn test_tradeoff_evaluate_rejects_config_without_tradeoffs() {
    let temp_dir = tempdir().expect("failed to create temp dir");
    let config_path = temp_dir.path().join("perfgate.toml");
    let scenario_path = temp_dir.path().join("scenario.json");

    fs::write(&config_path, "[defaults]\nthreshold = 0.20\n").expect("failed to write config");
    write_scenario_receipt(&scenario_path, 80.0, "fail");

    perfgate_cmd()
        .arg("tradeoff")
        .arg("evaluate")
        .arg("--config")
        .arg(&config_path)
        .arg("--scenario")
        .arg(&scenario_path)
        .arg("--out")
        .arg(temp_dir.path().join("tradeoff.json"))
        .assert()
        .failure()
        .stderr(predicate::str::contains(
            "no [[tradeoff]] entries configured",
        ));
}

fn write_tradeoff_config(path: &Path, min_improvement_ratio: f64, downgrade_to: &str) {
    fs::write(
        path,
        format!(
            r#"[[tradeoff]]
name = "memory_for_speed"
if_failed = "max_rss_kb"
downgrade_to = "{downgrade_to}"

[[tradeoff.require]]
metric = "wall_ms"
min_improvement_ratio = {min_improvement_ratio}
"#
        ),
    )
    .expect("failed to write config");
}

fn write_scenario_receipt(path: &Path, wall_current: f64, memory_status: &str) {
    let memory_fail = u32::from(memory_status == "fail");
    let memory_warn = u32::from(memory_status == "warn");
    let memory_pass = u32::from(memory_status == "pass");
    let reasons: Vec<&str> = if memory_status == "fail" {
        vec!["max_rss_kb_fail"]
    } else if memory_status == "warn" {
        vec!["max_rss_kb_warn"]
    } else {
        Vec::new()
    };
    let receipt = json!({
        "schema": "perfgate.scenario.v1",
        "tool": {"name": "perfgate", "version": "0.16.0"},
        "run": {
            "id": "scenario-run",
            "started_at": "2026-05-08T00:00:00Z",
            "ended_at": "2026-05-08T00:00:01Z",
            "host": {"os": "linux", "arch": "x86_64"}
        },
        "scenario": {
            "name": "release_workload",
            "weight": 1.0
        },
        "components": [],
        "weighted_deltas": {
            "wall_ms": {
                "baseline": 100.0,
                "current": wall_current,
                "ratio": wall_current / 100.0,
                "pct": (wall_current - 100.0) / 100.0,
                "regression": 0.0,
                "status": "pass"
            },
            "max_rss_kb": {
                "baseline": 100.0,
                "current": 120.0,
                "ratio": 1.20,
                "pct": 0.20,
                "regression": 0.20,
                "status": memory_status
            }
        },
        "verdict": {
            "status": memory_status,
            "counts": {
                "pass": 1 + memory_pass,
                "warn": memory_warn,
                "fail": memory_fail,
                "skip": 0
            },
            "reasons": reasons
        }
    });

    fs::write(
        path,
        serde_json::to_string_pretty(&receipt).expect("serialize scenario fixture"),
    )
    .expect("failed to write scenario fixture");
}
