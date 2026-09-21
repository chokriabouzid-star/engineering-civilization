//! T1.3 / ADR-026 C3 — static least-privilege contract for the EC seccomp profile.
//!
//! This gate encodes EC's OWN policy decision. It does not fetch or diff against
//! upstream Moby at test time (no network, no moving branch).
//!
//! Decision under test:
//!   * clone  -> ALLOW only when no namespace-creation bit is set
//!               (flags & 0x7e020000) == 0, checked on arg0 (x86 ABI).
//!   * clone3 -> ERRNO(38 = ENOSYS), so glibc falls back to the filterable
//!               clone path. EPERM would NOT trigger that fallback.
//!   * mount/ptrace/namespace syscalls stay denied.
//!   * statfs/fstatfs stay allowed (regression guard for the ADR-026 C3 breakage).
//!
//! Profile arches are X86_64/X86/X32 only, where clone's `flags` is arg0.
//! Do NOT copy this rule to s390x (arg0/arg1 are swapped there).

use serde_json::Value;
use std::collections::HashSet;

const PROFILE: &str = include_str!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/profiles/rust-sandbox.json"
));

// CLONE_NEWNS|NEWCGROUP|NEWUTS|NEWIPC|NEWUSER|NEWPID|NEWNET
const CLONE_NS_MASK: u64 = 0x7e02_0000; // 2114060288
const ENOSYS: u64 = 38;

const MUST_NOT_BE_ALLOWED: &[&str] = &[
    "unshare", "setns", "mount", "umount2", "pivot_root",
    "fsopen", "fsconfig", "fsmount", "move_mount", "open_tree", "mount_setattr",
    "ptrace", "process_vm_readv", "process_vm_writev",
    "keyctl", "add_key", "request_key", "userfaultfd", "bpf", "perf_event_open",
];

const MUST_STAY_ALLOWED: &[&str] = &["statfs", "fstatfs"];

fn validate(json: &str) -> Result<(), Vec<String>> {
    let p: Value = match serde_json::from_str(json) {
        Ok(v) => v,
        Err(e) => return Err(vec![format!("unparsable profile: {e}")]),
    };

    let mut errs: Vec<String> = Vec::new();

    if p.get("defaultAction").and_then(Value::as_str) != Some("SCMP_ACT_ERRNO") {
        errs.push("defaultAction must be SCMP_ACT_ERRNO".to_string());
    }
    if p.get("defaultErrnoRet").and_then(Value::as_u64) != Some(1) {
        errs.push("defaultErrnoRet must be 1".to_string());
    }

    let groups = match p.get("syscalls").and_then(Value::as_array) {
        Some(g) => g,
        None => {
            errs.push("missing `syscalls` array".to_string());
            return Err(errs);
        }
    };

    let mut allow_plain: HashSet<&str> = HashSet::new();
    let mut allow_args: Vec<(&str, &Value)> = Vec::new();
    let mut errno_rules: Vec<(&str, u64)> = Vec::new();

    for (i, g) in groups.iter().enumerate() {
        // Unsupported conditioning must fail loudly, never be ignored.
        if g.get("includes").is_some() || g.get("excludes").is_some() {
            errs.push(format!(
                "group {i}: includes/excludes not covered by this contract"
            ));
        }

        let action = g.get("action").and_then(Value::as_str).unwrap_or("");
        let names = match g.get("names").and_then(Value::as_array) {
            Some(n) => n,
            None => {
                errs.push(format!("group {i}: missing `names`"));
                continue;
            }
        };
        let args = g.get("args");
        let errno = g.get("errnoRet").and_then(Value::as_u64);

        for n in names {
            let Some(name) = n.as_str() else {
                errs.push(format!("group {i}: non-string syscall name"));
                continue;
            };
            match action {
                "SCMP_ACT_ALLOW" => match args {
                    None => {
                        allow_plain.insert(name);
                    }
                    Some(a) => allow_args.push((name, a)),
                },
                "SCMP_ACT_ERRNO" => errno_rules.push((name, errno.unwrap_or(1))),
                other => errs.push(format!("group {i}: unsupported action `{other}`")),
            }
        }
    }

    // ---- clone ----
    if allow_plain.contains("clone") {
        errs.push("clone is unconditionally ALLOWed (no args filter)".to_string());
    }
    let clone_rules: Vec<&(&str, &Value)> =
        allow_args.iter().filter(|(n, _)| *n == "clone").collect();
    match clone_rules.len() {
        0 => {
            if !allow_plain.contains("clone") {
                errs.push("clone has no conditional ALLOW rule".to_string());
            }
        }
        1 => {
            let args = clone_rules[0].1.as_array();
            match args {
                Some(a) if a.len() == 1 => {
                    let arg = &a[0];
                    if arg.get("index").and_then(Value::as_u64) != Some(0) {
                        errs.push("clone args[0].index must be 0".to_string());
                    }
                    if arg.get("op").and_then(Value::as_str) != Some("SCMP_CMP_MASKED_EQ") {
                        errs.push("clone args[0].op must be SCMP_CMP_MASKED_EQ".to_string());
                    }
                    if arg.get("value").and_then(Value::as_u64) != Some(CLONE_NS_MASK) {
                        errs.push(format!(
                            "clone args[0].value must be {CLONE_NS_MASK} (0x7e020000)"
                        ));
                    }
                    if arg.get("valueTwo").and_then(Value::as_u64) != Some(0) {
                        errs.push("clone args[0].valueTwo must be explicitly 0".to_string());
                    }
                }
                _ => errs.push("clone rule must have exactly one arg condition".to_string()),
            }
        }
        n => errs.push(format!("clone must appear in exactly 1 ALLOW rule, found {n}")),
    }
    if errno_rules.iter().any(|(n, _)| *n == "clone") {
        errs.push("clone must not also appear in an ERRNO rule".to_string());
    }

    // ---- clone3 ----
    if allow_plain.contains("clone3") || allow_args.iter().any(|(n, _)| *n == "clone3") {
        errs.push("clone3 must not be ALLOWed".to_string());
    }
    match errno_rules.iter().find(|(n, _)| *n == "clone3") {
        None => errs.push("clone3 must have an explicit ERRNO rule".to_string()),
        Some((_, code)) if *code != ENOSYS => errs.push(format!(
            "clone3 must return ENOSYS({ENOSYS}) for glibc fallback, found {code}"
        )),
        Some(_) => {}
    }

    // ---- denied set ----
    for s in MUST_NOT_BE_ALLOWED {
        if allow_plain.contains(s) || allow_args.iter().any(|(n, _)| n == s) {
            errs.push(format!("{s} must never be ALLOWed"));
        }
    }

    // ---- ADR-026 C3 regression guard ----
    for s in MUST_STAY_ALLOWED {
        let present = allow_plain.contains(s) || allow_args.iter().any(|(n, _)| n == s);
        if !present {
            errs.push(format!("{s} must stay ALLOWed (ADR-026 C3)"));
        }
    }

    if errs.is_empty() {
        Ok(())
    } else {
        Err(errs)
    }
}

// ---------------------------------------------------------------- real profile

#[test]
fn ec_profile_satisfies_least_privilege_contract() {
    if let Err(errs) = validate(PROFILE) {
        panic!("profile violates T1.3 contract:\n  - {}", errs.join("\n  - "));
    }
}

// ------------------------------------------------- in-memory compliant fixture

// Builds an in-memory profile that satisfies the T1.3 contract. The committed
// profile is intentionally NOT compliant before the T1.3 policy patch, so
// negative-control tests must start from a known-good fixture, then weaken
// exactly one property.
fn compliant_profile() -> Value {
    let mut profile: Value =
        serde_json::from_str(PROFILE).expect("committed base profile parses");

    let groups = profile["syscalls"]
        .as_array_mut()
        .expect("profile has syscalls array");

    for group in groups.iter_mut() {
        if let Some(names) = group["names"].as_array_mut() {
            names.retain(|name| name != "clone" && name != "clone3");
        }
    }

    groups.push(serde_json::json!({
        "names": ["clone"],
        "action": "SCMP_ACT_ALLOW",
        "args": [{
            "index": 0,
            "value": CLONE_NS_MASK,
            "valueTwo": 0,
            "op": "SCMP_CMP_MASKED_EQ"
        }]
    }));

    groups.push(serde_json::json!({
        "names": ["clone3"],
        "action": "SCMP_ACT_ERRNO",
        "errnoRet": ENOSYS
    }));

    profile
}

fn mutated(f: impl FnOnce(&mut Value)) -> String {
    let mut profile = compliant_profile();
    f(&mut profile);
    profile.to_string()
}

fn rejects(json: &str, needle: &str) {
    match validate(json) {
        Ok(()) => panic!("gate accepted a weakened profile (expected `{needle}`)"),
        Err(errs) => assert!(
            errs.iter().any(|e| e.contains(needle)),
            "expected an error containing `{needle}`, got: {errs:?}"
        ),
    }
}

#[test]
fn in_memory_compliant_fixture_satisfies_contract() {
    let fixture = compliant_profile();
    validate(&fixture.to_string())
        .expect("the compliant in-memory fixture must satisfy the T1.3 contract");
}

// --------------------------------------------------------- negative controls

#[test]
fn rejects_unconditional_clone() {
    let bad = mutated(|v| {
        v["syscalls"].as_array_mut().unwrap().push(serde_json::json!({
            "names": ["clone"],
            "action": "SCMP_ACT_ALLOW"
        }));
    });
    rejects(&bad, "unconditionally");
}

#[test]
fn rejects_wrong_clone_mask() {
    let bad = mutated(|v| {
        for g in v["syscalls"].as_array_mut().unwrap() {
            let is_clone_with_args = g.get("args").is_some()
                && g["names"]
                    .as_array()
                    .is_some_and(|n| n.iter().any(|x| x == "clone"));
            if is_clone_with_args {
                g["args"][0]["value"] = serde_json::json!(0);
            }
        }
    });
    rejects(&bad, "value must be");
}

#[test]
fn rejects_clone_mask_on_wrong_arg_index() {
    let bad = mutated(|v| {
        for g in v["syscalls"].as_array_mut().unwrap() {
            let is_clone_with_args = g.get("args").is_some()
                && g["names"]
                    .as_array()
                    .is_some_and(|n| n.iter().any(|x| x == "clone"));
            if is_clone_with_args {
                g["args"][0]["index"] = serde_json::json!(1);
            }
        }
    });
    rejects(&bad, "index must be 0");
}

#[test]
fn rejects_clone3_allow() {
    let bad = mutated(|v| {
        v["syscalls"].as_array_mut().unwrap().push(serde_json::json!({
            "names": ["clone3"],
            "action": "SCMP_ACT_ALLOW"
        }));
    });
    rejects(&bad, "clone3 must not be ALLOWed");
}

// EPERM instead of ENOSYS — the change that silently breaks pthread_create.
#[test]
fn rejects_clone3_eperm_instead_of_enosys() {
    let bad = mutated(|v| {
        for g in v["syscalls"].as_array_mut().unwrap() {
            let is_clone3_errno = g["action"] == "SCMP_ACT_ERRNO"
                && g["names"]
                    .as_array()
                    .is_some_and(|n| n.iter().any(|x| x == "clone3"));
            if is_clone3_errno {
                g["errnoRet"] = serde_json::json!(1);
            }
        }
    });
    rejects(&bad, "ENOSYS");
}

#[test]
fn rejects_allowing_a_mount_api_syscall() {
    let bad = mutated(|v| {
        v["syscalls"].as_array_mut().unwrap().push(serde_json::json!({
            "names": ["move_mount"],
            "action": "SCMP_ACT_ALLOW"
        }));
    });
    rejects(&bad, "move_mount must never be ALLOWed");
}

#[test]
fn rejects_default_action_allow() {
    let bad = mutated(|v| v["defaultAction"] = serde_json::json!("SCMP_ACT_ALLOW"));
    rejects(&bad, "defaultAction");
}

#[test]
fn rejects_dropping_statfs() {
    let bad = mutated(|v| {
        for g in v["syscalls"].as_array_mut().unwrap() {
            if g["action"] == "SCMP_ACT_ALLOW" {
                if let Some(names) = g["names"].as_array_mut() {
                    names.retain(|n| n != "statfs");
                }
            }
        }
    });
    rejects(&bad, "statfs must stay ALLOWed");
}

#[test]
fn rejects_unparsable_profile() {
    rejects("{ not json", "unparsable");
}
