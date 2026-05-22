#![forbid(unsafe_code)]

//! Week 54 Gate — ec-cli integration tests

use assert_cmd::Command;
use std::fs;
use tempfile::TempDir;

fn ec() -> Command {
    Command::cargo_bin("ec").unwrap()
}

fn write_sample_code(dir: &TempDir) -> std::path::PathBuf {
    let path = dir.path().join("sample.rs");
    fs::write(&path, "fn add(a: i32, b: i32) -> i32 { a + b }\n").unwrap();
    path
}

fn write_unsafe_code(dir: &TempDir) -> std::path::PathBuf {
    let path = dir.path().join("unsafe.rs");
    fs::write(&path, "unsafe fn raw(ptr: *const i32) -> i32 { *ptr }\n").unwrap();
    path
}

fn write_test_code(dir: &TempDir) -> std::path::PathBuf {
    let path = dir.path().join("tested.rs");
    fs::write(
        &path,
        "#[test]\nfn it_works() { assert_eq!(2 + 2, 4); }\n",
    )
    .unwrap();
    path
}

// ─── Gate 1: analyze basic ──────────────────────────────────────────

#[test]
fn w54_analyze_basic() {
    let dir = TempDir::new().unwrap();
    let path = write_sample_code(&dir);

    ec().arg("analyze").arg(path).assert().success();
}

#[test]
fn w54_analyze_shows_dimensions() {
    let dir = TempDir::new().unwrap();
    let path = write_sample_code(&dir);

    let output = ec().arg("analyze").arg(path).output().unwrap();
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("Security"));
    assert!(stdout.contains("Test Coverage"));
    assert!(stdout.contains("Maintainability"));
    assert!(stdout.contains("Performance"));
    assert!(stdout.contains("Stability"));
    assert!(stdout.contains("Reversibility"));
}

#[test]
fn w54_analyze_json_mode() {
    let dir = TempDir::new().unwrap();
    let path = write_sample_code(&dir);

    let output = ec().arg("analyze").arg(path).arg("--json").output().unwrap();
    let stdout = String::from_utf8_lossy(&output.stdout);
    let v: serde_json::Value = serde_json::from_str(&stdout).unwrap();
    assert!(v["fitness"]["security"].as_f64().unwrap() >= 0.0);
    assert!(v["fitness"]["test_coverage"].as_f64().unwrap() >= 0.0);
    assert!(v["confidence"]["overall"].as_f64().unwrap() >= 0.0);
    assert!(v["parse_successful"].as_bool().unwrap());
}

#[test]
fn w54_analyze_verbose() {
    let dir = TempDir::new().unwrap();
    let path = write_sample_code(&dir);

    ec().arg("analyze").arg(path).arg("--verbose").assert().success();
}

// ─── Gate 2: analyze unsafe code ────────────────────────────────────

#[test]
fn w54_analyze_unsafe_lower_security() {
    let dir = TempDir::new().unwrap();
    let safe_path = write_sample_code(&dir);
    let unsafe_path = write_unsafe_code(&dir);

    let safe_out = ec().arg("analyze").arg(&safe_path).arg("--json").output().unwrap();
    let unsafe_out = ec().arg("analyze").arg(&unsafe_path).arg("--json").output().unwrap();

    let safe_json: serde_json::Value =
        serde_json::from_str(&String::from_utf8_lossy(&safe_out.stdout)).unwrap();
    let unsafe_json: serde_json::Value =
        serde_json::from_str(&String::from_utf8_lossy(&unsafe_out.stdout)).unwrap();

    let safe_sec = safe_json["fitness"]["security"].as_f64().unwrap();
    let unsafe_sec = unsafe_json["fitness"]["security"].as_f64().unwrap();
    assert!(safe_sec > unsafe_sec, "safe={} should be > unsafe={}", safe_sec, unsafe_sec);
}

// ─── Gate 3: analyze tested code ────────────────────────────────────

#[test]
fn w54_analyze_tested_higher_coverage() {
    let dir = TempDir::new().unwrap();
    let plain_path = write_sample_code(&dir);
    let tested_path = write_test_code(&dir);

    let plain_out = ec().arg("analyze").arg(&plain_path).arg("--json").output().unwrap();
    let tested_out = ec().arg("analyze").arg(&tested_path).arg("--json").output().unwrap();

    let plain_json: serde_json::Value =
        serde_json::from_str(&String::from_utf8_lossy(&plain_out.stdout)).unwrap();
    let tested_json: serde_json::Value =
        serde_json::from_str(&String::from_utf8_lossy(&tested_out.stdout)).unwrap();

    let plain_cov = plain_json["fitness"]["test_coverage"].as_f64().unwrap();
    let tested_cov = tested_json["fitness"]["test_coverage"].as_f64().unwrap();
    assert!(tested_cov > plain_cov, "tested={} should be > plain={}", tested_cov, plain_cov);
}

// ─── Gate 4: analyze nonexistent file ───────────────────────────────

#[test]
fn w54_analyze_nonexistent_file_fails() {
    let result = ec()
        .arg("analyze")
        .arg("/nonexistent/file.rs")
        .output()
        .unwrap();
    assert!(!result.status.success());
    let stderr = String::from_utf8_lossy(&result.stderr);
    assert!(stderr.contains("❌") || stderr.contains("No such file"));
}

// ─── Gate 5: health ─────────────────────────────────────────────────

#[test]
fn w54_health() {
    let output = ec().arg("health").output().unwrap();
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("OK"));
    assert!(stdout.contains("Version"));
}

// ─── Gate 6: drift (no db) ──────────────────────────────────────────

#[test]
fn w54_drift_no_db() {
    let dir = TempDir::new().unwrap();
    let db = dir.path().join("nonexistent.db");
    let output = ec()
        .arg("drift")
        .arg("--db")
        .arg(db.to_str().unwrap())
        .output()
        .unwrap();
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("not found") || stdout.contains("Insufficient"));
}

// ─── Gate 7: propose subcommands ────────────────────────────────────

#[test]
fn w54_propose_list() {
    let output = ec().arg("propose").arg("list").output().unwrap();
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("REST API"));
}

#[test]
fn w54_propose_submit() {
    let output = ec()
        .arg("propose")
        .arg("submit")
        .arg("--dimension")
        .arg("security")
        .arg("--current")
        .arg("0.70")
        .arg("--proposed")
        .arg("0.75")
        .arg("--justification")
        .arg("test")
        .output()
        .unwrap();
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("POST"));
}

#[test]
fn w54_propose_approve() {
    let output = ec()
        .arg("propose")
        .arg("approve")
        .arg("test-id")
        .arg("--by")
        .arg("lead")
        .output()
        .unwrap();
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("PATCH") || stdout.contains("Approve"));
}

// ─── Gate 8: audit ──────────────────────────────────────────────────

#[test]
fn w54_audit() {
    let output = ec().arg("audit").output().unwrap();
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("audit") || stdout.contains("API"));
}

// ─── Gate 9: --version ──────────────────────────────────────────────

#[test]
fn w54_version() {
    let output = ec().arg("--version").output().unwrap();
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("ec"));
}

// ─── Gate 10: --help ────────────────────────────────────────────────

#[test]
fn w54_help() {
    let output = ec().arg("--help").output().unwrap();
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("analyze"));
    assert!(stdout.contains("drift"));
    assert!(stdout.contains("propose"));
    assert!(stdout.contains("audit"));
    assert!(stdout.contains("health"));
}

// ─── Final Gate ─────────────────────────────────────────────────────

#[test]
fn w54_gate_complete() {
    let dir = TempDir::new().unwrap();
    let path = write_sample_code(&dir);

    let output = ec().arg("analyze").arg(&path).output().unwrap();
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("Security"));
    assert!(stdout.contains("✅ OK"));

    println!("═══════════════════════════════════════════════");
    println!("  Week 54 Gate — ec-cli");
    println!("═══════════════════════════════════════════════");
    println!("  Commands: analyze, drift, propose, audit, health");
    println!("  JSON mode: ✅");
    println!("  Table mode: ✅");
    println!("  Error handling: ✅");
    println!("  --version / --help: ✅");
    println!("═══════════════════════════════════════════════");
    println!("  ✅ Week 54 Gate: PASSED");
}
