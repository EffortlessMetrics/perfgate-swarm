//! Integration tests for first-use performance review explanation.

use predicates::prelude::*;
use serde_json::Value;
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

fn write_config(dir: &Path) {
    let command = success_command()
        .iter()
        .map(|part| format!("\"{}\"", part))
        .collect::<Vec<_>>()
        .join(", ");
    fs::write(
        dir.join("perfgate.toml"),
        format!(
            r#"[defaults]
repeat = 7
warmup = 0
baseline_dir = "baselines"
out_dir = "artifacts/perfgate"

[[bench]]
name = "review-bench"
command = [{command}]
"#
        ),
    )
    .expect("write config");
}

fn write_imported_summary_receipt(path: &Path) {
    let started_at = chrono::Utc::now() - chrono::Duration::days(1);
    let receipt = serde_json::json!({
        "schema": "perfgate.run.v1",
        "tool": { "name": "perfgate-ingest", "version": "0.21.0" },
        "run": {
            "id": "review-bench-imported",
            "started_at": started_at.to_rfc3339(),
            "ended_at": (started_at + chrono::Duration::seconds(1)).to_rfc3339(),
            "host": {
                "os": std::env::consts::OS,
                "arch": std::env::consts::ARCH
            }
        },
        "bench": {
            "name": "review-bench",
            "command": [
                "(ingested k6 summary JSON)",
                "source_kind=k6_summary_json",
                "source_path=artifacts/k6-summary.json",
                "latency_metric=http_req_duration",
                "throughput_metric=http_reqs",
                "sample_model=summary_only"
            ],
            "repeat": 0,
            "warmup": 0
        },
        "samples": [],
        "stats": {
            "wall_ms": {
                "median": 120,
                "min": 100,
                "max": 150,
                "mean": 122.0,
                "stddev": 12.0
            },
            "throughput_per_s": {
                "median": 50.0,
                "min": 45.0,
                "max": 55.0,
                "mean": 50.0
            }
        }
    });
    fs::create_dir_all(path.parent().expect("receipt parent")).expect("create receipt parent");
    fs::write(
        path,
        serde_json::to_string_pretty(&receipt).expect("serialize receipt"),
    )
    .expect("write receipt");
}

#[test]
fn review_explain_keeps_missing_baseline_as_setup_not_regression() {
    let dir = tempdir().expect("tempdir");
    write_config(dir.path());

    perfgate_cmd()
        .current_dir(dir.path())
        .args([
            "review",
            "explain",
            "--config",
            "perfgate.toml",
            "--bench",
            "review-bench",
        ])
        .assert()
        .success()
        .stdout(predicate::str::contains("perfgate review explain"))
        .stdout(predicate::str::contains(
            "Gate verdict: setup_incomplete_missing_baseline",
        ))
        .stdout(predicate::str::contains("Baseline maturity: missing"))
        .stdout(predicate::str::contains(
            "Signal confidence: no_decision_yet",
        ))
        .stdout(predicate::str::contains(
            "Policy posture: current=smoke, recommended=advisory",
        ))
        .stdout(predicate::str::contains(
            "missing baseline is setup, not a regression",
        ))
        .stdout(predicate::str::contains(
            "require server ledger mode for local correctness",
        ))
        .stdout(predicate::str::contains(
            "Advisory only: no config, baseline, threshold, policy, or server setting was changed.",
        ));
}

#[test]
fn review_explain_json_reports_imported_evidence_limits() {
    let dir = tempdir().expect("tempdir");
    write_config(dir.path());
    write_imported_summary_receipt(&dir.path().join("artifacts/perfgate/review-bench/run.json"));

    let output = perfgate_cmd()
        .current_dir(dir.path())
        .args([
            "review",
            "explain",
            "--config",
            "perfgate.toml",
            "--bench",
            "review-bench",
            "--json",
        ])
        .output()
        .expect("run review explain");
    assert!(output.status.success(), "review explain should succeed");
    let value: Value = serde_json::from_slice(&output.stdout).expect("json output");

    assert_eq!(value["bench"], "review-bench");
    assert_eq!(
        value["evidence_source"]["kind"],
        "imported (k6_summary_json)"
    );
    assert_eq!(value["evidence_source"]["sample_model"], "summary_only");
    assert!(
        value["non_inferences"]
            .as_array()
            .expect("non_inferences array")
            .iter()
            .any(|item| item
                .as_str()
                .expect("string item")
                .contains("summary-only evidence has limited noise support"))
    );
    assert!(
        value["agent_guardrails"]["forbidden_by_default"]
            .as_array()
            .expect("forbidden array")
            .iter()
            .any(|item| item.as_str() == Some("loosen thresholds"))
    );
}
