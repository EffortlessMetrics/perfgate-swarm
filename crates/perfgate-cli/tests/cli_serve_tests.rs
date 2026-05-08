//! CLI tests for the local `perfgate serve` sandbox.

use predicates::prelude::*;
use std::net::TcpListener;
use tempfile::tempdir;

mod common;
use common::perfgate_cmd;

#[test]
fn serve_doctor_checks_local_database_and_port() {
    let temp_dir = tempdir().expect("temp dir");
    let db_path = temp_dir.path().join("nested").join("perfgate.db");

    perfgate_cmd()
        .args(["serve", "--doctor", "--port", "0", "--db"])
        .arg(&db_path)
        .assert()
        .success()
        .stdout(predicate::str::contains("perfgate serve doctor"))
        .stdout(predicate::str::contains("INFO database"))
        .stdout(predicate::str::contains("INFO api"))
        .stdout(predicate::str::contains("INFO health"))
        .stdout(predicate::str::contains("OK   database dir"))
        .stdout(predicate::str::contains("OK   sqlite storage"))
        .stdout(predicate::str::contains("OK   dashboard bind"))
        .stdout(predicate::str::contains("Summary: 0 failed checks"));

    assert!(
        db_path.exists(),
        "serve doctor should initialize the SQLite DB"
    );
}

#[test]
fn serve_doctor_fails_when_dashboard_port_is_unavailable() {
    let listener = TcpListener::bind("127.0.0.1:0").expect("bind test port");
    let port = listener.local_addr().expect("local addr").port();
    let temp_dir = tempdir().expect("temp dir");
    let db_path = temp_dir.path().join("perfgate.db");

    perfgate_cmd()
        .args(["serve", "--doctor", "--port", &port.to_string(), "--db"])
        .arg(&db_path)
        .assert()
        .failure()
        .stdout(predicate::str::contains("FAIL dashboard bind"))
        .stdout(predicate::str::contains("Summary: 1 failed check"))
        .stderr(predicate::str::contains(
            "serve doctor found 1 failed check",
        ));
}

#[test]
fn serve_help_shows_doctor_flag() {
    perfgate_cmd()
        .args(["serve", "--help"])
        .assert()
        .success()
        .stdout(predicate::str::contains("Start a local dashboard server"))
        .stdout(predicate::str::contains("--doctor"))
        .stdout(predicate::str::contains("--db"));
}
