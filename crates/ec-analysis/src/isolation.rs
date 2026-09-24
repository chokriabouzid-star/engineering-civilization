#![forbid(unsafe_code)]

//! عزل التحليل في عملية فرعية (F1).
//!
//! تجاوز المكدس داخل `syn` يُجهض العملية بإشارة `SIGABRT`؛ لا يلتقطه
//! `catch_unwind`، ولا ينحصر في الخيط المُخطئ. الوسيلة الوحيدة لإبقاء خادم
//! طويل العمر (أو الـCLI) حيًّا هي تنفيذ التحليل في **عملية منفصلة** وترجمة
//! موت الابن إلى خطأ مُهيكل.

use std::io::{Read, Write};
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::sync::atomic::{AtomicBool, Ordering};
use std::time::{Duration, Instant};

use crate::report::AnalysisReport;

/// متغيّر البيئة الذي يحوّل ثنائيًا مؤهَّلًا إلى وضع العامل.
pub const WORKER_ENV: &str = "EC_ANALYZE_WORKER";

/// مسار صريح اختياري لثنائي عامل.
pub const WORKER_BIN_ENV: &str = "EC_ANALYZE_WORKER_BIN";

/// السطر الأول الذي يطبعه العامل قبل حمولة JSON.
pub const WORKER_MAGIC: &str = "EC-ANALYZE-WORKER-V1";

/// ثنائيات هذا المستودع التي تُركِّب خطاف العامل في `main`.
pub const WORKER_CAPABLE_BINARIES: &[&str] = &["ec-analyze-worker", "ec-server", "ec"];

/// الميزانية الزمنية لتحليل معزول واحد.
pub const WORKER_TIMEOUT: Duration = Duration::from_secs(20);

static SELF_IS_WORKER_CAPABLE: AtomicBool = AtomicBool::new(false);

/// أسباب فشل التحليل المعزول.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum IsolationError {
    /// لا يوجد ثنائي عامل قابل للاكتشاف.
    WorkerNotFound,
    /// تعذّر إطلاق العامل أو التواصل معه.
    Spawn(String),
    /// تجاوز العامل الميزانية الزمنية فقُتل وحُصد.
    Timeout {
        /// الميزانية بالمللي ثانية.
        millis: u64,
    },
    /// انتهى العامل بحالة غير ناجحة (تجاوز مكدس، إشارة، خروج != 0).
    Crashed {
        /// وصف حالة الخروج كما يقدّمها نظام التشغيل.
        status: String,
        /// مقتطف من مخرَج الخطأ القياسي للعامل.
        stderr: String,
    },
    /// خالف العامل بروتوكول المخرَج المتفق عليه.
    Protocol(String),
}

impl std::fmt::Display for IsolationError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::WorkerNotFound => write!(f, "no analysis worker binary could be located"),
            Self::Spawn(e) => write!(f, "cannot start analysis worker: {e}"),
            Self::Timeout { millis } => {
                write!(
                    f,
                    "analysis worker exceeded {millis}ms budget and was killed"
                )
            }
            Self::Crashed { status, stderr } => {
                write!(f, "analysis worker died ({status}); stderr: {stderr}")
            }
            Self::Protocol(e) => write!(f, "analysis worker protocol violation: {e}"),
        }
    }
}

impl std::error::Error for IsolationError {}

/// يجب استدعاؤها كأول سطر في `main` لكل ثنائي يستهلك التحليل.
///
/// إن كان `EC_ANALYZE_WORKER=1` فالعملية تتحوّل إلى عامل ولا تعود أبدًا.
/// وإلا تُسجَّل العملية كمؤهَّلة لخدمة نفسها كعامل.
pub fn install_worker_hook() {
    if std::env::var(WORKER_ENV).as_deref() == Ok("1") {
        run_worker();
    }
    SELF_IS_WORKER_CAPABLE.store(true, Ordering::SeqCst);
}

/// حلقة العامل: يقرأ الكود من stdin، ويكتب البانر ثم JSON إلى stdout.
pub fn run_worker() -> ! {
    let mut code = String::new();
    if let Err(e) = std::io::stdin().read_to_string(&mut code) {
        eprintln!("ec analyze worker: cannot read stdin: {e}");
        std::process::exit(64);
    }

    let report = crate::analyze_code_full(&code);
    let json = match serde_json::to_string(&report) {
        Ok(j) => j,
        Err(e) => {
            eprintln!("ec analyze worker: cannot serialize report: {e}");
            std::process::exit(65);
        }
    };

    let mut out = std::io::stdout();
    if out
        .write_all(format!("{WORKER_MAGIC}\n{json}\n").as_bytes())
        .is_err()
    {
        std::process::exit(66);
    }
    let _ = out.flush();
    std::process::exit(0);
}

/// يحدّد ثنائي العامل المستخدَم، أو `None` إن تعذّر.
pub fn locate_worker() -> Option<PathBuf> {
    if let Ok(explicit) = std::env::var(WORKER_BIN_ENV) {
        let path = PathBuf::from(explicit);
        return if path.is_file() { Some(path) } else { None };
    }

    let current = std::env::current_exe().ok()?;
    if SELF_IS_WORKER_CAPABLE.load(Ordering::SeqCst) {
        return Some(current);
    }

    // سياق الاختبارات داخل العملية: المُنفِّذ هو حزمة اختبار في `target/*/deps`.
    let mut roots: Vec<PathBuf> = Vec::new();
    if let Some(dir) = current.parent() {
        roots.push(dir.to_path_buf());
        if let Some(up) = dir.parent() {
            roots.push(up.to_path_buf());
        }
    }
    for dir in roots {
        for name in WORKER_CAPABLE_BINARIES {
            let candidate = dir.join(format!("{name}{}", std::env::consts::EXE_SUFFIX));
            if candidate.is_file() {
                return Some(candidate);
            }
        }
    }
    None
}

/// تحليل معزول بميزانية زمنية افتراضية.
pub fn analyze_code_full_isolated(code: &str) -> Result<AnalysisReport, IsolationError> {
    analyze_code_full_isolated_with_timeout(code, WORKER_TIMEOUT)
}

/// تحليل معزول بميزانية زمنية صريحة.
pub fn analyze_code_full_isolated_with_timeout(
    code: &str,
    timeout: Duration,
) -> Result<AnalysisReport, IsolationError> {
    // حاجز ضد التعاود: العامل نفسه لا يُفرِّخ عاملًا آخر أبدًا.
    if std::env::var(WORKER_ENV).as_deref() == Ok("1") {
        return Ok(crate::analyze_code_full(code));
    }
    let worker = locate_worker().ok_or(IsolationError::WorkerNotFound)?;
    run_command(worker_command(&worker), code, timeout)
}

/// يبني أمر العامل بأقل صلاحية: بيئة فارغة تمامًا عدا مفتاح وضع العامل.
/// العامل يعالج مدخلات عدائية، فلا يرث `EC_API_KEY` ولا `EC_DB` ولا غيرهما.
fn worker_command(worker: &Path) -> Command {
    let mut command = Command::new(worker);
    command.env_clear().env(WORKER_ENV, "1");
    command
}

fn run_command(
    mut command: Command,
    code: &str,
    timeout: Duration,
) -> Result<AnalysisReport, IsolationError> {
    let mut child = command
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .map_err(|e| IsolationError::Spawn(e.to_string()))?;

    let stdin = child
        .stdin
        .take()
        .ok_or_else(|| IsolationError::Spawn("worker stdin unavailable".to_string()))?;
    let stdout = child
        .stdout
        .take()
        .ok_or_else(|| IsolationError::Spawn("worker stdout unavailable".to_string()))?;
    let stderr = child
        .stderr
        .take()
        .ok_or_else(|| IsolationError::Spawn("worker stderr unavailable".to_string()))?;

    let payload = code.to_owned();
    let writer = std::thread::spawn(move || {
        let mut stdin = stdin;
        let _ = stdin.write_all(payload.as_bytes());
        let _ = stdin.flush();
        // إسقاط المقبض يُغلق الأنبوب فينتهي قارئ العامل.
    });
    let out_reader = std::thread::spawn(move || {
        let mut stdout = stdout;
        let mut buf = String::new();
        let _ = stdout.read_to_string(&mut buf);
        buf
    });
    let err_reader = std::thread::spawn(move || {
        let mut stderr = stderr;
        let mut buf = String::new();
        let _ = stderr.read_to_string(&mut buf);
        buf
    });

    let deadline = Instant::now() + timeout;
    let mut timed_out = false;
    let finished = loop {
        match child.try_wait() {
            Ok(Some(status)) => break Some(status),
            Ok(None) => {}
            Err(e) => {
                let _ = child.kill();
                let _ = child.wait();
                return Err(IsolationError::Spawn(e.to_string()));
            }
        }
        if Instant::now() >= deadline {
            timed_out = true;
            let _ = child.kill();
            let _ = child.wait();
            break None;
        }
        std::thread::sleep(Duration::from_millis(5));
    };

    let _ = writer.join();
    let stdout_text = out_reader.join().unwrap_or_default();
    let stderr_text = err_reader.join().unwrap_or_default();

    if timed_out {
        return Err(IsolationError::Timeout {
            millis: u64::try_from(timeout.as_millis()).unwrap_or(u64::MAX),
        });
    }

    let status = finished
        .ok_or_else(|| IsolationError::Protocol("worker status unavailable".to_string()))?;
    if !status.success() {
        return Err(IsolationError::Crashed {
            status: status.to_string(),
            stderr: truncate(&stderr_text),
        });
    }

    parse_worker_output(&stdout_text)
}

fn parse_worker_output(raw: &str) -> Result<AnalysisReport, IsolationError> {
    let mut lines = raw.lines();
    let banner = lines.next().unwrap_or_default();
    if banner.trim() != WORKER_MAGIC {
        return Err(IsolationError::Protocol(format!(
            "unexpected worker banner: {}",
            truncate(banner)
        )));
    }
    let json = lines.next().unwrap_or_default();
    serde_json::from_str(json).map_err(|e| IsolationError::Protocol(e.to_string()))
}

fn truncate(text: &str) -> String {
    const MAX: usize = 512;
    if text.len() <= MAX {
        return text.to_string();
    }
    let mut end = MAX;
    while end > 0 && !text.is_char_boundary(end) {
        end -= 1;
    }
    format!("{}...", &text[..end])
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn worker_output_roundtrips_through_json() {
        let report = crate::analyze_code_full("fn f() {}");
        let json = serde_json::to_string(&report).expect("report must serialize");
        let raw = format!("{WORKER_MAGIC}\n{json}\n");
        let parsed = parse_worker_output(&raw).expect("banner + json must parse");
        assert!(parsed.parse_successful);
    }

    #[test]
    fn foreign_stdout_is_rejected_not_silently_accepted() {
        let err = parse_worker_output("running 3 tests\ntest x ... ok\n")
            .expect_err("libtest output must not be accepted as a report");
        assert!(matches!(err, IsolationError::Protocol(_)), "{err}");
    }

    #[test]
    fn missing_json_line_is_a_protocol_error() {
        let raw = format!("{WORKER_MAGIC}\n");
        let err = parse_worker_output(&raw).expect_err("missing payload must fail");
        assert!(matches!(err, IsolationError::Protocol(_)), "{err}");
    }

    #[test]
    fn missing_worker_binary_is_a_spawn_error() {
        let command = worker_command(Path::new("/nonexistent/ec-analyze-worker"));
        let err = run_command(command, "fn f() {}", Duration::from_secs(5))
            .expect_err("a missing binary cannot succeed");
        assert!(matches!(err, IsolationError::Spawn(_)), "{err}");
    }

    #[cfg(unix)]
    fn shell(script: &str) -> Command {
        let mut command = Command::new("/bin/sh");
        command.arg("-c").arg(script);
        command
    }

    #[cfg(unix)]
    #[test]
    fn hung_worker_is_killed_at_its_deadline() {
        let started = Instant::now();
        // `exec` يجعل sleep هو الابن نفسه، فالقتل يطاله مباشرة.
        let err = run_command(
            shell("exec sleep 30"),
            "fn f() {}",
            Duration::from_millis(300),
        )
        .expect_err("a hung worker must not be awaited forever");
        assert_eq!(err, IsolationError::Timeout { millis: 300 }, "{err}");
        assert!(
            started.elapsed() < Duration::from_secs(10),
            "deadline not enforced: {:?}",
            started.elapsed()
        );
    }

    #[cfg(unix)]
    #[test]
    fn nonzero_worker_exit_is_a_crash() {
        let err = run_command(
            shell("cat >/dev/null; echo boom >&2; exit 7"),
            "fn f() {}",
            Duration::from_secs(5),
        )
        .expect_err("exit 7 must not be accepted");
        match err {
            IsolationError::Crashed { status, stderr } => {
                assert!(status.contains('7'), "status: {status}");
                assert!(stderr.contains("boom"), "stderr: {stderr}");
            }
            other => panic!("expected Crashed, got {other}"),
        }
    }

    #[cfg(unix)]
    #[test]
    fn signal_death_is_a_crash_not_a_result() {
        // SIGKILL يحاكي موت العامل بإشارة (كتجاوز المكدس) دون ترك ملف core.
        let err = run_command(
            shell("cat >/dev/null; kill -KILL $$"),
            "fn f() {}",
            Duration::from_secs(5),
        )
        .expect_err("a signalled worker must not be accepted");
        assert!(matches!(err, IsolationError::Crashed { .. }), "{err}");
    }

    #[cfg(unix)]
    #[test]
    fn worker_starts_with_a_cleared_environment() {
        // cargo test يمرّر CARGO_MANIFEST_DIR وHOME؛ لو ورثهما العامل فالخروج 9.
        let mut command = worker_command(Path::new("/bin/sh"));
        command.arg("-c").arg(
            "[ -z \"$CARGO_MANIFEST_DIR\" ] && [ -z \"$HOME\" ] \
             && [ \"$EC_ANALYZE_WORKER\" = 1 ] && exit 0; exit 9",
        );
        let err = run_command(command, "", Duration::from_secs(5))
            .expect_err("empty stdout is never a valid report");
        assert!(
            matches!(err, IsolationError::Protocol(_)),
            "worker environment was not cleared: {err}"
        );
    }
}
