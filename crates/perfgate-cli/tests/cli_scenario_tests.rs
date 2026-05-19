//! Integration tests for `perfgate scenario`.

mod common;

use common::perfgate_cmd;
use predicates::prelude::*;
use serde_json::{Value, json};
use std::fs;
use std::path::Path;
use tempfile::tempdir;

#[test]
fn test_scenario_evaluate_writes_weighted_scenario_receipt() {
    let temp_dir = tempdir().expect("failed to create temp dir");
    let out_dir = temp_dir.path().join("artifacts").join("perfgate");
    write_compare_receipt(
        &out_dir.join("large-file").join("compare.json"),
        "large-file",
        100.0,
        90.0,
        "pass",
    );
    write_probe_compare_receipt(&out_dir.join("large-file").join("probe-compare.json"));
    write_compare_receipt(
        &out_dir.join("small-edit").join("compare.json"),
        "small-edit",
        100.0,
        110.0,
        "fail",
    );

    let config_path = temp_dir.path().join("perfgate.toml");
    fs::write(
        &config_path,
        format!(
            r#"[defaults]
threshold = 0.20
warn_factor = 0.50
out_dir = "{}"

[[bench]]
name = "large-file"
command = ["echo", "large"]

[[bench]]
name = "small-edit"
command = ["echo", "small"]

[[scenario]]
name = "large_file_parse"
weight = 0.75
bench = "large-file"
probe_compare = "{}"

[[scenario]]
name = "small_edit"
weight = 0.25
bench = "small-edit"
"#,
            toml_path(&out_dir),
            toml_path(&out_dir.join("large-file").join("probe-compare.json"))
        ),
    )
    .expect("failed to write config");

    let output_path = temp_dir.path().join("scenario.json");
    perfgate_cmd()
        .arg("scenario")
        .arg("evaluate")
        .arg("--config")
        .arg(&config_path)
        .arg("--out")
        .arg(&output_path)
        .assert()
        .success()
        .stderr(predicate::str::contains("Scenario receipt written"));

    let receipt: Value = serde_json::from_str(
        &fs::read_to_string(&output_path).expect("failed to read scenario receipt"),
    )
    .expect("scenario receipt should be JSON");
    let typed: perfgate_types::ScenarioReceipt =
        serde_json::from_value(receipt.clone()).expect("scenario receipt should deserialize");

    assert_eq!(typed.schema, "perfgate.scenario.v1");
    assert_eq!(typed.scenario.name, "configured_workload");
    assert_eq!(typed.components.len(), 2);
    assert_eq!(typed.verdict.status, perfgate_types::VerdictStatus::Pass);
    assert_eq!(receipt["weighted_deltas"]["wall_ms"]["baseline"], 100.0);
    assert_eq!(receipt["weighted_deltas"]["wall_ms"]["current"], 95.0);
    assert_eq!(receipt["weighted_deltas"]["wall_ms"]["status"], "pass");
    let expected_compare_path = out_dir
        .join("large-file")
        .join("compare.json")
        .display()
        .to_string();
    assert_eq!(
        receipt["components"][0]["compare_ref"]["path"]
            .as_str()
            .map(|path| path.replace('\\', "/")),
        Some(expected_compare_path.replace('\\', "/"))
    );
    assert_eq!(receipt["components"][0]["probes"][0], "parser.tokenize");
    assert_eq!(
        receipt["components"][0]["probe_compare_ref"]["path"]
            .as_str()
            .map(|path| path.replace('\\', "/")),
        Some(
            out_dir
                .join("large-file")
                .join("probe-compare.json")
                .display()
                .to_string()
                .replace('\\', "/")
        )
    );
}

#[test]
fn test_scenario_evaluate_rejects_config_without_scenarios() {
    let temp_dir = tempdir().expect("failed to create temp dir");
    let config_path = temp_dir.path().join("perfgate.toml");
    fs::write(
        &config_path,
        r#"[[bench]]
name = "large-file"
command = ["echo", "large"]
"#,
    )
    .expect("failed to write config");

    perfgate_cmd()
        .arg("scenario")
        .arg("evaluate")
        .arg("--config")
        .arg(&config_path)
        .arg("--out")
        .arg(temp_dir.path().join("scenario.json"))
        .assert()
        .failure()
        .stderr(predicate::str::contains(
            "no [[scenario]] entries configured",
        ));
}

#[test]
fn test_scenario_evaluate_records_missing_probe_compare_warning() {
    let temp_dir = tempdir().expect("failed to create temp dir");
    let out_dir = temp_dir.path().join("artifacts").join("perfgate");
    write_compare_receipt(
        &out_dir.join("large-file").join("compare.json"),
        "large-file",
        100.0,
        90.0,
        "pass",
    );

    let missing_probe_compare = out_dir
        .join("large-file")
        .join("missing-probe-compare.json");
    let config_path = temp_dir.path().join("perfgate.toml");
    fs::write(
        &config_path,
        format!(
            r#"[defaults]
out_dir = "{}"

[[bench]]
name = "large-file"
command = ["echo", "large"]

[[scenario]]
name = "large_file_parse"
weight = 1.0
bench = "large-file"
probe_compare = "{}"
"#,
            toml_path(&out_dir),
            toml_path(&missing_probe_compare)
        ),
    )
    .expect("failed to write config");

    let output_path = temp_dir.path().join("scenario.json");
    perfgate_cmd()
        .arg("scenario")
        .arg("evaluate")
        .arg("--config")
        .arg(&config_path)
        .arg("--out")
        .arg(&output_path)
        .assert()
        .success();

    let receipt: Value = serde_json::from_str(
        &fs::read_to_string(&output_path).expect("failed to read scenario receipt"),
    )
    .expect("scenario receipt should be JSON");
    assert!(
        receipt["warnings"]
            .as_array()
            .expect("scenario warnings should be array")
            .iter()
            .any(|warning| warning
                .as_str()
                .is_some_and(|warning| warning.contains("probe evidence missing")))
    );
}

fn write_compare_receipt(path: &Path, bench: &str, baseline: f64, current: f64, status: &str) {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).expect("failed to create compare fixture directory");
    }

    let fail = u32::from(status == "fail");
    let warn = u32::from(status == "warn");
    let pass = u32::from(status == "pass");
    let skip = u32::from(status == "skip");
    let regression = if current > baseline {
        (current - baseline) / baseline
    } else {
        0.0
    };
    let receipt = json!({
        "schema": "perfgate.compare.v1",
        "tool": {"name": "perfgate", "version": "0.16.0"},
        "bench": {
            "name": bench,
            "command": ["echo", bench],
            "repeat": 1,
            "warmup": 0
        },
        "baseline_ref": {
            "path": format!("baselines/{bench}.json"),
            "run_id": format!("{bench}-baseline")
        },
        "current_ref": {
            "path": format!("artifacts/perfgate/{bench}/run.json"),
            "run_id": format!("{bench}-current")
        },
        "budgets": {},
        "deltas": {
            "wall_ms": {
                "baseline": baseline,
                "current": current,
                "ratio": current / baseline,
                "pct": (current - baseline) / baseline,
                "regression": regression,
                "status": status
            }
        },
        "verdict": {
            "status": status,
            "counts": {
                "pass": pass,
                "warn": warn,
                "fail": fail,
                "skip": skip
            },
            "reasons": []
        }
    });

    fs::write(
        path,
        serde_json::to_string_pretty(&receipt).expect("serialize compare fixture"),
    )
    .expect("failed to write compare fixture");
}

fn write_probe_compare_receipt(path: &Path) {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).expect("failed to create probe compare fixture directory");
    }

    let receipt = json!({
        "schema": "perfgate.probe_compare.v1",
        "tool": {"name": "perfgate", "version": "0.16.0"},
        "run": {
            "id": "probe-compare-run",
            "started_at": "2026-05-08T00:00:00Z",
            "ended_at": "2026-05-08T00:00:01Z",
            "host": {"os": "linux", "arch": "x86_64"}
        },
        "scenario": "large_file_parse",
        "probes": [{
            "name": "parser.tokenize",
            "scope": "local",
            "baseline_count": 1,
            "current_count": 1,
            "deltas": {},
            "status": "pass"
        }],
        "verdict": {
            "status": "pass",
            "counts": {"pass": 1, "warn": 0, "fail": 0, "skip": 0},
            "reasons": []
        }
    });

    fs::write(
        path,
        serde_json::to_string_pretty(&receipt).expect("serialize probe compare fixture"),
    )
    .expect("failed to write probe compare fixture");
}

fn toml_path(path: &Path) -> String {
    path.display().to_string().replace('\\', "/")
}
