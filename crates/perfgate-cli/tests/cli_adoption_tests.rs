//! Integration tests for adoption-pack catalog surfaces.

use predicates::prelude::*;

mod common;
use common::perfgate_cmd;

#[test]
fn adoption_packs_lists_reviewable_catalog() {
    perfgate_cmd()
        .args(["adoption", "packs"])
        .assert()
        .success()
        .stdout(predicate::str::contains(
            "Adoption packs are reviewable starting points",
        ))
        .stdout(predicate::str::contains(
            "They do not detect benchmarks magically",
        ))
        .stdout(predicate::str::contains("Pack: rust-cli"))
        .stdout(predicate::str::contains("Pack: rust-workspace"))
        .stdout(predicate::str::contains("Pack: python-service"))
        .stdout(predicate::str::contains("Pack: node-tool-action"))
        .stdout(predicate::str::contains("Pack: http-local-smoke"))
        .stdout(predicate::str::contains("Pack: generic-command"))
        .stdout(predicate::str::contains("Local reproduction:"))
        .stdout(predicate::str::contains("Do not infer:"));
}

#[test]
fn adoption_packs_can_show_one_pack() {
    perfgate_cmd()
        .args(["adoption", "packs", "--pack", "http-local-smoke"])
        .assert()
        .success()
        .stdout(predicate::str::contains("Pack: http-local-smoke"))
        .stdout(predicate::str::contains("k6 summary JSON"))
        .stdout(predicate::str::contains("production capacity proof"))
        .stdout(predicate::str::contains("Pack: rust-cli").not());
}

#[test]
fn adoption_packs_rejects_unknown_pack() {
    perfgate_cmd()
        .args(["adoption", "packs", "--pack", "mobile-app"])
        .assert()
        .failure()
        .stderr(predicate::str::contains("invalid value"));
}
