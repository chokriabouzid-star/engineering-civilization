#![forbid(unsafe_code)]

//! F1: hostile source must not abort the CLI parent process.

use std::fs;
use std::process::Command;
use tempfile::TempDir;

fn hostile_file(dir: &TempDir) -> std::path::PathBuf {
    let path = dir.path().join("hostile.rs");
    let code = format!(
        "fn f() {{ let x: {}i32{} = todo!(); }}",
        "Vec<".repeat(600),
        ">".repeat(600)
    );
    fs::write(&path, code).expect("write hostile source");
    path
}

#[test]
fn analyze_worker_crash_is_reported_without_aborting_cli() {
    let dir = TempDir::new().expect("temporary directory");
    let path = hostile_file(&dir);
    let output = Command::new(env!("CARGO_BIN_EXE_ec"))
        .args(["analyze", "--json"])
        .arg(path)
        .current_dir(dir.path())
        .output()
        .expect("run ec analyze");
    let stderr = String::from_utf8_lossy(&output.stderr);

    assert_eq!(
        output.status.code(),
        Some(2),
        "expected clean exit 2, not a signal or false success: status={:?}, \
         stdout={}, stderr={stderr}",
        output.status,
        String::from_utf8_lossy(&output.stdout)
    );
    assert!(
        stderr.contains("analysis worker died"),
        "stderr must name the worker failure: {stderr}"
    );
}

#[test]
fn check_worker_crash_fails_closed_without_aborting_cli() {
    let dir = TempDir::new().expect("temporary directory");
    hostile_file(&dir);
    let output = Command::new(env!("CARGO_BIN_EXE_ec"))
        .args(["check", "--json"])
        .arg(dir.path())
        .current_dir(dir.path())
        .output()
        .expect("run ec check");

    assert_eq!(
        output.status.code(),
        Some(1),
        "expected clean exit 1, not a signal or false success: status={:?}, \
         stdout={}, stderr={}",
        output.status,
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );

    let report: serde_json::Value =
        serde_json::from_slice(&output.stdout).expect("check must return JSON");
    assert_eq!(report["files_scanned"], 1);
    assert_eq!(report["files_failed"], 1);
    assert_eq!(report["violations"][0]["dimension"], "analysis_failed");
}
