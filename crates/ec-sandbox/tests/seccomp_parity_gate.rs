//! T1.2 GATE (ADR-026 C4 / ADR-028) — seccomp parity.
//!
//! الأطروحة: بروفايل EC المخصص **مُحمَّل ونافذ** في مسار الإنتاج، وهو **أشد**
//! من Docker default profile على ثلاثة متجهات تمييزية. الاكتمال / least-privilege
//! ليس موضوع هذه البوابة — ذاك T1.3.
//!
//! الذراعان (مقيسان 2026-09-20، Docker 29.1.3، x86_64، DEFAULT_IMAGE):
//!   ARM A = HardenedConfig::without_seccomp() => لا يُمرَّر --security-opt seccomp
//!           => Docker يطبّق بروفايله المدمج. **ليس unconfined.**
//!           ptrace=ESRCH(3) · pidfd_open=ESRCH(3) · process_vm_readv=EINVAL(22)
//!           unshare/add_key/keyctl/perf_event_open = EPERM(1)
//!   ARM B = HardenedConfig::default() => بروفايلنا (allow=203, defaultAction=ERRNO)
//!           السبعة كلها = EPERM(1)
//!
//! المؤشّر = errno، لا is_err(). defaultAction=SCMP_ACT_ERRNO يُرجع EPERM ولا يقتل
//! شيئاً؛ exit_code=0 في الذراعين. أي بوابة مبنية على is_err() باطلة هنا.
//! الوسائط زائفة عمداً كي يكون errno عند السماح مميَّزاً (ESRCH/EINVAL) لا EPERM.
//!
//! تحكم سلبي **بنيوي** (مجاني، بلا اختبار منفصل): لو انكسر تمرير البروفايل
//! (خطأ مسار، refactor يسقط --security-opt)، تُظهر ARM B قيم ARM A (3/3/22)
//! وتفشل توكيدات EPERM فوراً. تحقّق يدوي: بدّل ARM B إلى without_seccomp()
//! => يجب أن تفشل بـ got 3 / got 3 / got 22.
//!
//! متجهات غير تمييزية (مقيسة، لا مفترضة): unshare(CLONE_NEWUSER) و add_key و
//! keyctl و perf_event_open = EPERM في الذراعين (Docker default + --cap-drop ALL).
//! تُوكَّد كانحدار خط أساس فقط، وليست دليلاً على بروفايلنا.
//!
//! المتجهات الخمسة القديمة (week16_gate) غير مذكورة هنا: تحتويها cap-drop /
//! read-only / pids-limit في الذراعين => vacuous بالنسبة لـseccomp. تبقى كما هي.
//!
//! أرقام syscall خاصة بـx86_64 (101/434/310/272/248/250/298). CI والصورة x86_64.
//! `clone(CLONE_NEWUSER)` مستبعد عمداً: clone خام بلا stack مُدار = UB في Rust؛
//! السؤال يُحسم ستاتيكياً بمقارنة args-filter في T1.3.
//!
//! Run:
//!   cargo test -p ec-sandbox --locked --features docker_tests \
//!     --test seccomp_parity_gate -- --test-threads=1 --nocapture

use std::collections::HashMap;
use std::process::Command;
use std::time::Duration;

use ec_sandbox::hardened::{HardenedConfig, HardenedDockerRunner};

const EPERM: i32 = 1;
const ESRCH: i32 = 3;
const EINVAL: i32 = 22;

/// مسموحة في Docker default (تفشل منطقياً بوسائط زائفة)، محجوبة في بروفايلنا.
const DISCRIMINATORS: [(&str, i32); 3] = [
    ("ptrace_attach_bogus", ESRCH),
    ("pidfd_open_bogus", ESRCH),
    ("process_vm_readv", EINVAL),
];

/// EPERM في الذراعين — انحدار خط أساس، غير تمييزية.
const DEFENSE_IN_DEPTH: [&str; 4] = [
    "unshare_newuser",
    "add_key_null",
    "keyctl_null",
    "perf_event_open_null",
];

const EXPECTED_PROBE_LINES: usize = DISCRIMINATORS.len() + DEFENSE_IN_DEPTH.len();

/// مطابق حرفياً للـprobe المُثبت تجريبياً — لا تُغيَّر أرقام syscall ولا ترتيب الوسائط.
const PROBE_SRC: &str = r#"
extern "C" { fn syscall(num: i64, ...) -> i64; }
fn errno() -> i32 { std::io::Error::last_os_error().raw_os_error().unwrap_or(-1) }
fn probe(name: &str, r: i64) {
    let e = if r < 0 { errno() } else { 0 };
    println!("PROBE {} ret={} errno={}", name, r, e);
}
fn main() {
    const BOGUS: i64 = 0x7FFF_FFFF;
    unsafe {
        probe("ptrace_attach_bogus",  syscall(101, 16, BOGUS, 0, 0));
        probe("pidfd_open_bogus",     syscall(434, BOGUS, 0));
        probe("process_vm_readv",     syscall(310, BOGUS, 0, 0, 0, 0, 0));
        probe("unshare_newuser",      syscall(272, 0x10000000));
        probe("add_key_null",         syscall(248, 0, 0, 0, 0, 0));
        probe("keyctl_null",          syscall(250, 0, 0, 0, 0, 0));
        probe("perf_event_open_null", syscall(298, 0, 0, -1, -1, 0));
    }
}
"#;

/// يُبنى من الـAPI العام فقط (القاعدة 8) — لا مساس بـ`clone_with_timeout` الخاصة.
fn make_runner(timeout_secs: u64, hardened: HardenedConfig) -> HardenedDockerRunner {
    let base = ec_sandbox::docker::DockerRunner::new(
        ec_sandbox::docker::DEFAULT_IMAGE,
        512,
        0.5,
        Duration::from_secs(timeout_secs),
    );
    HardenedDockerRunner::new(base, hardened).expect("hardened runner construction failed")
}

/// يشغّل ذراعاً ويُرجع probe_name -> errno. يفشل بصوت عالٍ عند أي غموض:
/// toolchain مكسور يجب ألا يبدو كاحتواء.
fn run_arm(label: &str, hardened: HardenedConfig) -> HashMap<String, i32> {
    let out = make_runner(120, hardened)
        .compile_and_run_hardened(PROBE_SRC)
        .unwrap_or_else(|e| {
            panic!(
                "ARM {label}: execution failed: {e:?}\n\
                 البوابة لا تستطيع التمييز بين 'seccomp حجب' و'toolchain مكسور' — FAIL CLOSED."
            )
        });

    assert_eq!(
        out.exit_code, 0,
        "ARM {label}: probe must exit 0 (تفشل الـsyscalls لا العملية).\n\
         stdout:\n{}\nstderr:\n{}",
        out.stdout, out.stderr
    );

    let mut map = HashMap::new();
    for line in out.stdout.lines().filter(|l| l.starts_with("PROBE ")) {
        let mut parts = line.split_whitespace();
        let _tag = parts.next();
        let name = parts
            .next()
            .unwrap_or_else(|| panic!("ARM {label}: nameless probe line: {line}"))
            .to_string();
        let errno = parts
            .find_map(|p| p.strip_prefix("errno=").and_then(|v| v.parse::<i32>().ok()))
            .unwrap_or_else(|| panic!("ARM {label}: unparseable probe line: {line}"));
        println!("[{label}] {name} errno={errno}");
        map.insert(name, errno);
    }

    assert_eq!(
        map.len(),
        EXPECTED_PROBE_LINES,
        "ARM {label}: expected {EXPECTED_PROBE_LINES} probe lines, got {}.\n\
         raw stdout:\n{}",
        map.len(),
        out.stdout
    );
    map
}

fn orphan_count() -> usize {
    let out = Command::new("docker")
        .args(["ps", "-aq", "--filter", "name=ec-sbx-"])
        .output()
        .expect("`docker ps` failed — is the daemon running?");
    String::from_utf8_lossy(&out.stdout)
        .lines()
        .filter(|l| !l.trim().is_empty())
        .count()
}

/// تسلسلي عمداً: اختبار واحد كي لا يلوّث أي توازٍ داخل الملف عدّ `ec-sbx-`
/// (نفس نمط timeout_gate).
#[test]
#[cfg_attr(
    not(feature = "docker_tests"),
    ignore = "requires --features docker_tests + a running docker daemon"
)]
fn seccomp_custom_profile_is_strictly_stronger_than_docker_default() {
    let pre_orphans = orphan_count();

    let arm_a = run_arm("A_docker_default", HardenedConfig::without_seccomp());
    let arm_b = run_arm("B_custom_profile", HardenedConfig::default());

    // ── 1) ARM B: بروفايلنا يحجب السبعة كلها بـEPERM ─────────────────────
    for name in DISCRIMINATORS
        .iter()
        .map(|(n, _)| *n)
        .chain(DEFENSE_IN_DEPTH)
    {
        assert_eq!(
            arm_b[name], EPERM,
            "ARM B: `{name}` must be EPERM({EPERM}) under the custom profile, got {}.\n\
             لو ظهرت هنا قيم ARM A (3/3/22) فالبروفايل المخصص لم يُمرَّر أصلاً \
             (كسر في --security-opt seccomp=…) — هذا هو التحكم السلبي البنيوي.",
            arm_b[name]
        );
    }
    println!("[B_custom_profile] OK: 7/7 blocked (EPERM)");

    // ── 2) الخاصية التمييزية: أشد من Docker default ──────────────────────
    for (name, expected_a) in DISCRIMINATORS {
        assert_ne!(
            arm_a[name], EPERM,
            "ARM A: `{name}` returned EPERM under Docker default — \
             المتجه فقد قدرته التمييزية. أعد تشغيل الـprobe وحدّث ADR-028."
        );
        assert_eq!(
            arm_a[name], expected_a,
            "ARM A: `{name}` expected errno={expected_a}, got {}. \
             سلوك Docker default تغيّر (ترقية؟) — حدّث ADR-028 بالقياس الجديد، \
             لا تُرخِ التوكيد صامتاً.",
            arm_a[name]
        );
    }
    println!("[A_docker_default] OK: 3/3 discriminating (non-EPERM, exact)");

    // ── 3) دفاع-بعمق: EPERM في الذراعين (انحدار خط أساس) ─────────────────
    for name in DEFENSE_IN_DEPTH {
        assert_eq!(
            arm_a[name], EPERM,
            "ARM A: `{name}` expected EPERM from Docker default baseline, got {}.",
            arm_a[name]
        );
    }
    println!("[A_docker_default] OK: 4/4 defense-in-depth baseline");

    // ── 4) البروفايل لا يكسر كوداً بريئاً (rustc يعمل) ───────────────────
    let hello = make_runner(120, HardenedConfig::default())
        .compile_and_run_hardened(r#"fn main() { println!("hello_gate"); }"#)
        .expect("hello-world must run under the custom profile");
    assert_eq!(hello.exit_code, 0, "hello-world exit_code != 0");
    assert!(
        hello.stdout.contains("hello_gate"),
        "hello-world stdout missing marker:\n{}",
        hello.stdout
    );
    println!("[hello] OK: rustc works under custom profile");

    // ── 5) نظافة (C1): لا تسريب حاويات ───────────────────────────────────
    let post_orphans = orphan_count();
    assert!(
        post_orphans <= pre_orphans,
        "orphan containers leaked: pre={pre_orphans} post={post_orphans}"
    );
    println!("[POST] OK: orphans pre={pre_orphans} post={post_orphans}");
}
