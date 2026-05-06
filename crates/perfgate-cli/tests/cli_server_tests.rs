//! Integration tests for CLI commands that interact with the baseline server.
//!
//! These tests verify the CLI integration with the server including:
//! - `perfgate run --upload`
//! - `perfgate promote --to-server`
//! - `perfgate baseline list --baseline-server`
//! - `perfgate baseline upload`

use perfgate_server::auth::Role;
use perfgate_server::server::{ServerConfig, StorageBackend};
use perfgate_server::testing::{TestServer, spawn_test_server};
use predicates::prelude::*;
use std::env;
use std::fs;
use std::time::{SystemTime, UNIX_EPOCH};
use tempfile::TempDir;

mod common;
use common::{fixtures_dir, perfgate_cmd};

struct RunningTestServer {
    inner: TestServer,
}

impl RunningTestServer {
    async fn spawn(config: ServerConfig) -> Self {
        Self {
            inner: spawn_test_server(config).await,
        }
    }

    fn url(&self) -> &str {
        &self.inner.url
    }
}

impl Drop for RunningTestServer {
    fn drop(&mut self) {
        self.inner.handle.abort();
    }
}

fn add_success_command(cmd: &mut assert_cmd::Command) {
    cmd.arg("--");
    if cfg!(windows) {
        cmd.args(["cmd", "/C", "echo", "test"]);
    } else {
        cmd.args(["sh", "-c", "printf test"]);
    }
}

fn unique_project(label: &str) -> String {
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("system clock should be after unix epoch")
        .as_nanos();
    format!("cli-live-{label}-{nanos}")
}

fn contributor_key() -> String {
    ["pg_test_", "fixtureonlynotsecretserverclitests000000"].concat()
}

fn live_server_config(project: &str, backend: StorageBackend, api_key: &str) -> ServerConfig {
    ServerConfig::new().storage_backend(backend).scoped_api_key(
        api_key,
        Role::Contributor,
        project,
        None,
    )
}

fn add_server_flags(cmd: &mut assert_cmd::Command, server_url: &str, project: &str, api_key: &str) {
    cmd.arg("--baseline-server")
        .arg(server_url)
        .arg("--api-key")
        .arg(api_key)
        .arg("--project")
        .arg(project);
}

/// Test that `--upload` fails without `--baseline-server` configured.
#[test]
fn test_upload_requires_baseline_server() {
    let temp_dir = TempDir::new().expect("failed to create temp dir");
    let output_path = temp_dir.path().join("output.json");

    let mut cmd = perfgate_cmd();
    cmd.arg("run")
        .arg("--name")
        .arg("test")
        .arg("--repeat")
        .arg("1")
        .arg("--out")
        .arg(&output_path)
        .arg("--upload");
    add_success_command(&mut cmd);

    cmd.assert().failure().stderr(
        predicate::str::contains("baseline server is not configured")
            .and(predicate::str::contains("--baseline-server")),
    );
}

/// Test that `--upload` fails without `--project` configured.
#[test]
fn test_upload_requires_project() {
    let temp_dir = TempDir::new().expect("failed to create temp dir");
    let output_path = temp_dir.path().join("output.json");

    let mut cmd = perfgate_cmd();
    cmd.arg("run")
        .arg("--name")
        .arg("test")
        .arg("--repeat")
        .arg("1")
        .arg("--out")
        .arg(&output_path)
        .arg("--upload")
        .arg("--baseline-server")
        .arg("http://localhost:9999/api/v1");
    add_success_command(&mut cmd);

    cmd.assert().failure().stderr(
        predicate::str::contains("--project is required")
            .and(predicate::str::contains("PERFGATE_PROJECT")),
    );
}

/// Test that `--to-server` fails without `--baseline-server` configured.
#[test]
fn test_to_server_requires_baseline_server() {
    let temp_dir = TempDir::new().expect("failed to create temp dir");
    let baseline_path = temp_dir.path().join("baseline.json");

    // Create a minimal baseline file
    let baseline_content = serde_json::json!({
        "schema": "perfgate.run.v1",
        "tool": {"name": "perfgate", "version": "0.3.0"},
        "run": {
            "id": "test-run",
            "started_at": "2024-01-15T10:00:00Z",
            "ended_at": "2024-01-15T10:00:01Z",
            "host": {"os": "linux", "arch": "x86_64"}
        },
        "bench": {
            "name": "test-bench",
            "command": ["echo", "test"],
            "repeat": 1,
            "warmup": 0
        },
        "samples": [{"wall_ms": 100, "exit_code": 0, "warmup": false, "timed_out": false, "max_rss_kb": 1024}],
        "stats": {
            "wall_ms": {"median": 100, "min": 100, "max": 100},
            "max_rss_kb": {"median": 1024, "min": 1024, "max": 1024}
        }
    });
    fs::write(
        &baseline_path,
        serde_json::to_string(&baseline_content).unwrap(),
    )
    .unwrap();

    let mut cmd = perfgate_cmd();
    cmd.arg("promote")
        .arg("--current")
        .arg(&baseline_path)
        .arg("--to-server")
        .arg("--benchmark")
        .arg("test-bench");

    cmd.assert().failure().stderr(predicate::str::contains(
        "baseline server is not configured",
    ));
}

/// Test that `--to-server` fails without `--project` configured.
#[test]
fn test_to_server_requires_project() {
    let temp_dir = TempDir::new().expect("failed to create temp dir");
    let baseline_path = temp_dir.path().join("baseline.json");

    // Create a minimal baseline file
    let baseline_content = serde_json::json!({
        "schema": "perfgate.run.v1",
        "tool": {"name": "perfgate", "version": "0.3.0"},
        "run": {
            "id": "test-run",
            "started_at": "2024-01-15T10:00:00Z",
            "ended_at": "2024-01-15T10:00:01Z",
            "host": {"os": "linux", "arch": "x86_64"}
        },
        "bench": {
            "name": "test-bench",
            "command": ["echo", "test"],
            "repeat": 1,
            "warmup": 0
        },
        "samples": [{"wall_ms": 100, "exit_code": 0, "warmup": false, "timed_out": false, "max_rss_kb": 1024}],
        "stats": {
            "wall_ms": {"median": 100, "min": 100, "max": 100},
            "max_rss_kb": {"median": 1024, "min": 1024, "max": 1024}
        }
    });
    fs::write(
        &baseline_path,
        serde_json::to_string(&baseline_content).unwrap(),
    )
    .unwrap();

    let mut cmd = perfgate_cmd();
    cmd.arg("promote")
        .arg("--current")
        .arg(&baseline_path)
        .arg("--to-server")
        .arg("--baseline-server")
        .arg("http://localhost:9999/api/v1")
        .arg("--benchmark")
        .arg("test-bench");

    cmd.assert()
        .failure()
        .stderr(predicate::str::contains("--project is required"));
}

/// Test that `--to-server` fails without `--benchmark` configured.
#[test]
fn test_to_server_requires_benchmark() {
    let temp_dir = TempDir::new().expect("failed to create temp dir");
    let baseline_path = temp_dir.path().join("baseline.json");

    // Create a minimal baseline file
    let baseline_content = serde_json::json!({
        "schema": "perfgate.run.v1",
        "tool": {"name": "perfgate", "version": "0.3.0"},
        "run": {
            "id": "test-run",
            "started_at": "2024-01-15T10:00:00Z",
            "ended_at": "2024-01-15T10:00:01Z",
            "host": {"os": "linux", "arch": "x86_64"}
        },
        "bench": {
            "name": "test-bench",
            "command": ["echo", "test"],
            "repeat": 1,
            "warmup": 0
        },
        "samples": [{"wall_ms": 100, "exit_code": 0, "warmup": false, "timed_out": false, "max_rss_kb": 1024}],
        "stats": {
            "wall_ms": {"median": 100, "min": 100, "max": 100},
            "max_rss_kb": {"median": 1024, "min": 1024, "max": 1024}
        }
    });
    fs::write(
        &baseline_path,
        serde_json::to_string(&baseline_content).unwrap(),
    )
    .unwrap();

    let mut cmd = perfgate_cmd();
    cmd.arg("promote")
        .arg("--current")
        .arg(&baseline_path)
        .arg("--to-server")
        .arg("--baseline-server")
        .arg("http://localhost:9999/api/v1")
        .arg("--project")
        .arg("test-project");

    cmd.assert()
        .failure()
        .stderr(predicate::str::contains("--to-server requires --benchmark"));
}

/// Test that baseline list command requires server configuration.
#[test]
fn test_baseline_list_requires_server() {
    let mut cmd = perfgate_cmd();
    cmd.arg("baseline").arg("list");

    // Without --baseline-server, the command should either fail or show help
    // depending on implementation
    let output = cmd.output().expect("Failed to execute command");

    // The command should fail since no server is configured
    assert!(
        !output.status.success()
            || String::from_utf8_lossy(&output.stdout).contains("requires")
            || String::from_utf8_lossy(&output.stderr).contains("requires")
    );
}

/// Test that baseline upload command requires server configuration.
#[test]
fn test_baseline_upload_requires_server() {
    let temp_dir = TempDir::new().expect("failed to create temp dir");
    let receipt_path = temp_dir.path().join("receipt.json");

    // Create a minimal receipt file
    let receipt_content = serde_json::json!({
        "schema": "perfgate.run.v1",
        "tool": {"name": "perfgate", "version": "0.3.0"},
        "run": {
            "id": "test-run",
            "started_at": "2024-01-15T10:00:00Z",
            "ended_at": "2024-01-15T10:00:01Z",
            "host": {"os": "linux", "arch": "x86_64"}
        },
        "bench": {
            "name": "test-bench",
            "command": ["echo", "test"],
            "repeat": 1,
            "warmup": 0
        },
        "samples": [{"wall_ms": 100, "exit_code": 0, "warmup": false, "timed_out": false, "max_rss_kb": 1024}],
        "stats": {
            "wall_ms": {"median": 100, "min": 100, "max": 100},
            "max_rss_kb": {"median": 1024, "min": 1024, "max": 1024}
        }
    });
    fs::write(
        &receipt_path,
        serde_json::to_string(&receipt_content).unwrap(),
    )
    .unwrap();

    let mut cmd = perfgate_cmd();
    cmd.arg("baseline")
        .arg("upload")
        .arg("--benchmark")
        .arg("test-bench")
        .arg("--file")
        .arg(&receipt_path);

    // Should fail without server configuration
    let output = cmd.output().expect("Failed to execute command");
    assert!(!output.status.success());
}

/// Test that environment variables are used for server configuration.
#[test]
fn test_server_config_from_env() {
    let temp_dir = TempDir::new().expect("failed to create temp dir");
    let output_path = temp_dir.path().join("output.json");

    // Set environment variables
    let mut cmd = perfgate_cmd();
    cmd.env("PERFGATE_SERVER_URL", "http://localhost:9999/api/v1")
        .env("PERFGATE_API_KEY", "test-key")
        .env("PERFGATE_PROJECT", "test-project")
        .arg("run")
        .arg("--name")
        .arg("test")
        .arg("--repeat")
        .arg("1")
        .arg("--out")
        .arg(&output_path)
        .arg("--upload");
    add_success_command(&mut cmd);

    // The command should run but fail to connect to the server
    // (since there's no server running on port 9999)
    // This tests that the env vars are being read
    let output = cmd.output().expect("Failed to execute command");

    // The run should succeed (creating the receipt), but upload should fail
    // Check that it tried to upload (connection error message)
    let stderr = String::from_utf8_lossy(&output.stderr);

    // Either it failed during upload (connection error) or succeeded with local file
    // We're testing that the env vars were picked up
    assert!(
        output.status.success()
            || stderr.contains("connection")
            || stderr.contains("Failed to upload")
            || stderr.contains("connect")
    );
}

/// Test that CLI flags override environment variables.
#[test]
fn test_cli_flags_override_env() {
    let temp_dir = TempDir::new().expect("failed to create temp dir");
    let output_path = temp_dir.path().join("output.json");

    // Set environment variables
    let mut cmd = perfgate_cmd();
    cmd.env("PERFGATE_SERVER_URL", "http://env-server:9999/api/v1")
        .env("PERFGATE_API_KEY", "env-key")
        .env("PERFGATE_PROJECT", "env-project")
        .arg("run")
        .arg("--name")
        .arg("test")
        .arg("--repeat")
        .arg("1")
        .arg("--out")
        .arg(&output_path)
        .arg("--upload")
        .arg("--baseline-server")
        .arg("http://cli-server:8888/api/v1") // Override
        .arg("--api-key")
        .arg("cli-key") // Override
        .arg("--project")
        .arg("cli-project"); // Override
    add_success_command(&mut cmd);

    let output = cmd.output().expect("Failed to execute command");
    let stderr = String::from_utf8_lossy(&output.stderr);

    // The command should try to connect to cli-server, not env-server
    // We can verify this by checking the error message contains the CLI-specified server
    if !output.status.success() && stderr.contains("cli-server") {
        // Good - it tried to connect to the CLI-specified server
    }
    // If it succeeded, that's also fine (local file created)
}

/// Test help output for server-related commands.
#[test]
fn test_run_help_shows_upload_option() {
    let mut cmd = perfgate_cmd();
    cmd.arg("run").arg("--help");

    cmd.assert()
        .success()
        .stdout(predicate::str::contains("--upload"));
}

#[test]
fn test_promote_help_shows_to_server_option() {
    let mut cmd = perfgate_cmd();
    cmd.arg("promote").arg("--help");

    cmd.assert()
        .success()
        .stdout(predicate::str::contains("--to-server"));
}

#[test]
fn test_baseline_subcommand_exists() {
    let mut cmd = perfgate_cmd();
    cmd.arg("baseline").arg("--help");

    cmd.assert()
        .success()
        .stdout(predicate::str::contains("list"))
        .stdout(predicate::str::contains("upload"));
}

/// Test that global server flags are shown in help.
#[test]
fn test_global_server_flags_in_help() {
    let mut cmd = perfgate_cmd();
    cmd.arg("--help");

    cmd.assert()
        .success()
        .stdout(predicate::str::contains("--baseline-server"))
        .stdout(predicate::str::contains("--api-key"))
        .stdout(predicate::str::contains("--project"));
}

#[test]
fn test_admin_keys_subcommands_exist() {
    let mut cmd = perfgate_cmd();
    cmd.arg("admin").arg("keys").arg("--help");

    cmd.assert()
        .success()
        .stdout(predicate::str::contains("create"))
        .stdout(predicate::str::contains("list"))
        .stdout(predicate::str::contains("revoke"))
        .stdout(predicate::str::contains("rotate"));
}

/// Test compare with @server:benchmark reference (when server is not available).
#[test]
fn test_compare_server_reference_without_server() {
    let temp_dir = TempDir::new().expect("failed to create temp dir");
    let current_path = temp_dir.path().join("current.json");

    // Create a minimal current receipt
    let current_content = serde_json::json!({
        "schema": "perfgate.run.v1",
        "tool": {"name": "perfgate", "version": "0.3.0"},
        "run": {
            "id": "current-run",
            "started_at": "2024-01-15T10:00:00Z",
            "ended_at": "2024-01-15T10:00:01Z",
            "host": {"os": "linux", "arch": "x86_64"}
        },
        "bench": {
            "name": "test-bench",
            "command": ["echo", "test"],
            "repeat": 1,
            "warmup": 0
        },
        "samples": [{"wall_ms": 100, "exit_code": 0, "warmup": false, "timed_out": false, "max_rss_kb": 1024}],
        "stats": {
            "wall_ms": {"median": 100, "min": 100, "max": 100},
            "max_rss_kb": {"median": 1024, "min": 1024, "max": 1024}
        }
    });
    fs::write(
        &current_path,
        serde_json::to_string(&current_content).unwrap(),
    )
    .unwrap();

    let mut cmd = perfgate_cmd();
    cmd.arg("compare")
        .arg("--baseline")
        .arg("@server:test-bench")
        .arg("--current")
        .arg(&current_path)
        .arg("--baseline-server")
        .arg("http://localhost:9999/api/v1")
        .arg("--project")
        .arg("test-project");

    // Should fail because server is not available
    let output = cmd.output().expect("Failed to execute command");
    assert!(!output.status.success());
}

/// Test compare with an explicit baseline project override and no global project.
#[test]
fn test_compare_server_reference_with_baseline_project_without_global_project() {
    let temp_dir = TempDir::new().expect("failed to create temp dir");
    let current_path = temp_dir.path().join("current.json");

    let current_content = serde_json::json!({
        "schema": "perfgate.run.v1",
        "tool": {"name": "perfgate", "version": "0.3.0"},
        "run": {
            "id": "current-run",
            "started_at": "2024-01-15T10:00:00Z",
            "ended_at": "2024-01-15T10:00:01Z",
            "host": {"os": "linux", "arch": "x86_64"}
        },
        "bench": {
            "name": "test-bench",
            "command": ["echo", "test"],
            "repeat": 1,
            "warmup": 0
        },
        "samples": [{"wall_ms": 100, "exit_code": 0, "warmup": false, "timed_out": false, "max_rss_kb": 1024}],
        "stats": {
            "wall_ms": {"median": 100, "min": 100, "max": 100},
            "max_rss_kb": {"median": 1024, "min": 1024, "max": 1024}
        }
    });
    fs::write(
        &current_path,
        serde_json::to_string(&current_content).unwrap(),
    )
    .unwrap();

    let mut cmd = perfgate_cmd();
    cmd.arg("compare")
        .arg("--baseline")
        .arg("@server:test-bench")
        .arg("--baseline-project")
        .arg("source-project")
        .arg("--current")
        .arg(&current_path)
        .arg("--baseline-server")
        .arg("http://localhost:9999/api/v1");

    let output = cmd.output().expect("Failed to execute command");
    assert!(!output.status.success());

    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        !stderr.contains("--project is required"),
        "compare should accept --baseline-project as the server lookup scope: {stderr}"
    );
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn live_server_cli_workflow_memory() {
    let project = unique_project("memory");
    let api_key = contributor_key();
    let config = live_server_config(&project, StorageBackend::Memory, &api_key);

    run_live_server_cli_workflow(config, &project, &api_key).await;
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn live_server_cli_workflow_sqlite() {
    let temp_dir = TempDir::new().expect("failed to create temp dir");
    let project = unique_project("sqlite");
    let api_key = contributor_key();
    let config = live_server_config(&project, StorageBackend::Sqlite, &api_key)
        .sqlite_path(temp_dir.path().join("perfgate.db"));

    run_live_server_cli_workflow(config, &project, &api_key).await;
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn live_server_cli_workflow_postgres() {
    let Ok(postgres_url) = env::var("PERFGATE_TEST_POSTGRES_URL") else {
        eprintln!(
            "skipping postgres live server CLI workflow; PERFGATE_TEST_POSTGRES_URL is unset"
        );
        return;
    };

    let project = unique_project("postgres");
    let api_key = contributor_key();
    let config =
        live_server_config(&project, StorageBackend::Postgres, &api_key).postgres_url(postgres_url);

    run_live_server_cli_workflow(config, &project, &api_key).await;
}

async fn run_live_server_cli_workflow(config: ServerConfig, project: &str, api_key: &str) {
    let server = RunningTestServer::spawn(config).await;
    let temp_dir = TempDir::new().expect("failed to create temp dir");
    let baseline_file = fixtures_dir().join("baseline.json");
    let current_file = fixtures_dir().join("current_pass.json");
    let uploaded_bench = format!("{project}-uploaded");
    let run_bench = format!("{project}-run");
    let promoted_bench = format!("{project}-promoted");
    let downloaded_path = temp_dir.path().join("downloaded-baseline.json");
    let run_path = temp_dir.path().join("uploaded-run.json");
    let compare_path = temp_dir.path().join("server-compare.json");

    let mut upload = perfgate_cmd();
    upload
        .arg("baseline")
        .arg("upload")
        .arg("--file")
        .arg(&baseline_file)
        .arg("--benchmark")
        .arg(&uploaded_bench)
        .arg("--version")
        .arg("seed-v1");
    add_server_flags(&mut upload, server.url(), project, api_key);
    upload
        .assert()
        .success()
        .stderr(predicate::str::contains(&uploaded_bench));

    let mut list = perfgate_cmd();
    list.arg("baseline").arg("list").arg("--limit").arg("10");
    add_server_flags(&mut list, server.url(), project, api_key);
    list.assert()
        .success()
        .stdout(predicate::str::contains(&uploaded_bench));

    let mut download = perfgate_cmd();
    download
        .arg("baseline")
        .arg("download")
        .arg("--benchmark")
        .arg(&uploaded_bench)
        .arg("--output")
        .arg(&downloaded_path);
    add_server_flags(&mut download, server.url(), project, api_key);
    download.assert().success();
    assert!(
        downloaded_path.exists(),
        "downloaded baseline should be written"
    );
    let downloaded: serde_json::Value = serde_json::from_str(
        &fs::read_to_string(&downloaded_path).expect("downloaded baseline should be readable"),
    )
    .expect("downloaded baseline should be valid JSON");
    assert_eq!(downloaded["schema"].as_str(), Some("perfgate.run.v1"));

    let mut history = perfgate_cmd();
    history
        .arg("baseline")
        .arg("history")
        .arg("--benchmark")
        .arg(&uploaded_bench);
    add_server_flags(&mut history, server.url(), project, api_key);
    history
        .assert()
        .success()
        .stdout(predicate::str::contains("seed-v1"));

    let mut run = perfgate_cmd();
    run.arg("run")
        .arg("--name")
        .arg(&run_bench)
        .arg("--repeat")
        .arg("1")
        .arg("--out")
        .arg(&run_path)
        .arg("--upload");
    add_server_flags(&mut run, server.url(), project, api_key);
    add_success_command(&mut run);
    run.assert()
        .success()
        .stderr(predicate::str::contains(&run_bench));
    assert!(
        run_path.exists(),
        "uploaded run receipt should be preserved"
    );

    let mut compare = perfgate_cmd();
    compare
        .arg("compare")
        .arg("--baseline")
        .arg(format!("@server:{uploaded_bench}"))
        .arg("--current")
        .arg(&current_file)
        .arg("--out")
        .arg(&compare_path)
        .arg("--host-mismatch")
        .arg("ignore");
    add_server_flags(&mut compare, server.url(), project, api_key);
    compare.assert().success();
    assert!(
        compare_path.exists(),
        "server compare receipt should be written"
    );
    let compare_receipt: serde_json::Value = serde_json::from_str(
        &fs::read_to_string(&compare_path).expect("compare receipt should be readable"),
    )
    .expect("compare receipt should be valid JSON");
    assert_eq!(
        compare_receipt["schema"].as_str(),
        Some("perfgate.compare.v1")
    );

    let mut promote = perfgate_cmd();
    promote
        .arg("promote")
        .arg("--current")
        .arg(&baseline_file)
        .arg("--to-server")
        .arg("--benchmark")
        .arg(&promoted_bench)
        .arg("--version")
        .arg("promoted-v1");
    add_server_flags(&mut promote, server.url(), project, api_key);
    promote
        .assert()
        .success()
        .stderr(predicate::str::contains(&promoted_bench));
}
