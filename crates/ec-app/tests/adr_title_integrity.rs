#![forbid(unsafe_code)]

//! ADR title integrity gate: every docs/adr/ADR-NNN-*.md must start with
//! a heading of the form `# ADR-NNN:` matching its own file name.
//! Legacy files 001-003 (no ADR- prefix) must still carry `# ADR-NNN:`.

use std::fs;
use std::path::Path;

fn expected_number(name: &str) -> Option<String> {
    let stem = name.strip_suffix(".md")?;
    let rest = stem.strip_prefix("ADR-").unwrap_or(stem);
    let digits: String = rest.chars().take_while(|c| c.is_ascii_digit()).collect();
    if digits.len() == 3 {
        Some(digits)
    } else {
        None
    }
}

#[test]
fn every_adr_heading_matches_its_file_number() {
    let dir = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../docs/adr");
    let mut violations = Vec::new();
    let mut checked = 0usize;

    for entry in fs::read_dir(&dir).expect("docs/adr must exist") {
        let path = entry.expect("dir entry").path();
        let name = path.file_name().unwrap().to_string_lossy().to_string();
        if !name.ends_with(".md") {
            continue;
        }
        let Some(num) = expected_number(&name) else {
            violations.push(format!("{name}: cannot derive ADR number from file name"));
            continue;
        };
        checked += 1;
        let text = fs::read_to_string(&path).expect("readable ADR");
        let first_nonempty = text.lines().find(|l| !l.trim().is_empty()).unwrap_or("");
        let expected = format!("# ADR-{num}:");
        if !first_nonempty.starts_with(&expected) {
            violations.push(format!(
                "{name}: first heading is {first_nonempty:?}, expected to start with {expected:?}"
            ));
        }
    }

    assert!(
        checked >= 26,
        "expected at least 26 ADR files, found {checked}"
    );
    assert!(
        violations.is_empty(),
        "ADR title violations:\n{}",
        violations.join("\n")
    );
}
