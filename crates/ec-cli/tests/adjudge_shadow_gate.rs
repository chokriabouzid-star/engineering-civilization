#![forbid(unsafe_code)]

//! بوابة ec adjudge في shadow mode — PR2.
//!
//! كل اختبار يُجبر Docker غير متاح بمسار PATH فارغ، فيكون الحكم Unverified
//! حتميًا بلا Docker وبلا تفاوت بيئي.

use assert_cmd::Command;
use std::path::Path;

fn empty_bin_dir() -> tempfile::TempDir {
    let dir = tempfile::tempdir().expect("tempdir");
    std::fs::create_dir(dir.path().join("bin")).expect("empty bin dir");
    dir
}

fn run_adjudge(bin_dir: &Path, file: &Path, json: bool) -> (bool, String) {
    let mut cmd = Command::cargo_bin("ec").expect("ec binary");
    cmd.env("PATH", bin_dir.join("bin").as_os_str());
    cmd.arg("adjudge").arg(file);
    if json {
        cmd.arg("--json");
    }
    let output = cmd.output().expect("run ec adjudge");
    (
        output.status.success(),
        String::from_utf8_lossy(&output.stdout).into_owned(),
    )
}

#[test]
fn adjudge_exits_zero_on_clean_file() {
    let dir = empty_bin_dir();
    let file = dir.path().join("clean.rs");
    std::fs::write(&file, "pub fn add(a: i32, b: i32) -> i32 {\n    a + b\n}\n").unwrap();

    let (success, stdout) = run_adjudge(dir.path(), &file, false);
    assert!(success, "shadow mode must exit 0, got: {}", stdout);
    assert!(stdout.contains("Unverified"), "got: {}", stdout);
}

#[test]
fn adjudge_exits_zero_on_hostile_file() {
    let dir = empty_bin_dir();
    let file = dir.path().join("hostile.rs");
    let hostile = "(".repeat(5000);
    std::fs::write(&file, hostile).unwrap();

    let (success, stdout) = run_adjudge(dir.path(), &file, false);
    assert!(
        success,
        "hostile input must not abort the CLI, got: {}",
        stdout
    );
}

#[test]
fn adjudge_json_reports_unverified_without_evidence() {
    let dir = empty_bin_dir();
    let file = dir.path().join("clean.rs");
    std::fs::write(&file, "pub fn f() {}\n").unwrap();

    let (success, stdout) = run_adjudge(dir.path(), &file, true);
    assert!(success, "shadow mode must exit 0, got: {}", stdout);

    let v: serde_json::Value = serde_json::from_str(&stdout).expect("valid json output");
    assert_eq!(v["verdict"], "Unverified");
    let reason = v["reason"].as_str().expect("reason must be a string");
    assert!(!reason.trim().is_empty(), "reason must not be empty");
    assert!(v["evidence"].is_null(), "no evidence when execution failed");
}
