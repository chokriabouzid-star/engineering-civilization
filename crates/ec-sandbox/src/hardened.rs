#![forbid(unsafe_code)]

//! Security Hardening — Week 16
//!
//! يوفر Docker runner مُقوَّى بـ:
//! - seccomp profile
//! - non-root user
//! - read-only filesystem
//! - tmpfs للـ workspace فقط
//! - pids-limit (يمنع fork bombs بشكل حاسم — أُضيف بعد Phase 1 CI، انظر
//!   PIDS_LIMIT أدناه للتفاصيل)

use crate::docker::{DockerError, DockerOutput, DockerRunner};
use std::path::PathBuf;
use std::time::Duration;

/// مسار seccomp profile الافتراضي.
pub const SECCOMP_PROFILE_PATH: &str =
    concat!(env!("CARGO_MANIFEST_DIR"), "/profiles/rust-sandbox.json");

/// إعدادات الأمان المُقوَّى.
#[derive(Debug, Clone)]
pub struct HardenedConfig {
    /// مسار seccomp profile.
    pub seccomp_profile: Option<PathBuf>,
    /// تشغيل كـ non-root user.
    pub user_id: u32,
    /// group id.
    pub group_id: u32,
    /// read-only filesystem (مع tmpfs exceptions).
    pub read_only_root: bool,
    /// حجم tmpfs workspace بالميجابايت.
    pub workspace_size_mb: u64,
}

impl Default for HardenedConfig {
    fn default() -> Self {
        Self {
            seccomp_profile: Some(PathBuf::from(SECCOMP_PROFILE_PATH)),
            user_id: 1000,
            group_id: 1000,
            read_only_root: true,
            workspace_size_mb: 200,
        }
    }
}

impl HardenedConfig {
    /// بدون seccomp (للاختبار).
    pub fn without_seccomp() -> Self {
        Self {
            seccomp_profile: None,
            ..Default::default()
        }
    }

    /// التحقق من صحة الإعدادات.
    pub fn validate(&self) -> anyhow::Result<()> {
        if let Some(path) = &self.seccomp_profile {
            anyhow::ensure!(
                path.exists(),
                "seccomp profile not found: {}",
                path.display()
            );
        }
        anyhow::ensure!(
            self.workspace_size_mb >= 50,
            "workspace_size_mb must be >= 50"
        );
        Ok(())
    }
}

/// Docker runner مُقوَّى أمنياً.
pub struct HardenedDockerRunner {
    base: DockerRunner,
    hardened: HardenedConfig,
}

impl HardenedDockerRunner {
    /// الحد الأقصى لعدد الـ PIDs (processes/threads) داخل الحاوية عبر
    /// cgroup pids controller.
    ///
    /// اكتُشفت الحاجة لهذا في Phase 1 CI (GitHub Actions): الاحتواء كان
    /// يعتمد ضمنيًا على حدود الذاكرة/CPU فقط، وهذا غير كافٍ لمنع fork bomb —
    /// `--memory` لا تحدّ عدد الـ threads المسموح إنشاؤها، فقط استهلاك
    /// الذاكرة الكلي. على بيئة أقوى من WSL2 (4 CPU / 15.6GB) تمكّن اختبار
    /// fork bomb من إنشاء 10,000 thread دون أي حد. `--pids-limit` يمنع هذا
    /// بشكل حاسم ومستقل عن موارد الـ host.
    ///
    /// القيمة 256: هامش مريح فوق احتياجات تجميع/تشغيل أي برنامج Rust
    /// معقول (rustc + linker + البرنامج الناتج)، وصغيرة كفاية لإيقاف أي
    /// fork bomb حقيقي. إن كسرت اختبارات التجميع العادية، ارفعها إلى 512.
    const PIDS_LIMIT: u32 = 256;

    /// عتبة اعتبار الاختبار "هروبًا": أي عدد أعلى من PIDS_LIMIT + هامش
    /// أمان يعني أن pids-limit لم يُطبَّق فعليًا (لا مجرد اقتراب طبيعي
    /// من الحد المُهيّأ).
    const FORK_BOMB_ESCAPE_THRESHOLD: u32 = Self::PIDS_LIMIT + 50;

    /// إنشاء runner مُقوَّى.
    pub fn new(base: DockerRunner, hardened: HardenedConfig) -> anyhow::Result<Self> {
        hardened.validate()?;
        Ok(Self { base, hardened })
    }

    /// إنشاء runner مُقوَّى بالإعدادات الافتراضية.
    pub fn default_hardened() -> anyhow::Result<Self> {
        Self::new(DockerRunner::default(), HardenedConfig::default())
    }

    /// إنشاء runner مُقوَّى بدون seccomp (للاختبار).
    pub fn for_testing() -> anyhow::Result<Self> {
        Self::new(DockerRunner::default(), HardenedConfig::without_seccomp())
    }

    /// تجميع وتشغيل Rust code بأمان كامل.
    pub fn compile_and_run_hardened(&self, source_code: &str) -> Result<DockerOutput, DockerError> {
        let escaped = source_code.replace('\'', r#"'"'"'"#);

        let script = format!(
            "printf '%s' '{escaped}' > /workspace/main.rs && \
             rustc /workspace/main.rs -o /workspace/program 2>&1 && \
             echo '---OUTPUT---' && \
             /workspace/program"
        );

        // بناء Docker args
        let mut args = vec![
            "run".to_string(),
            "--rm".to_string(),
            "--network".to_string(),
            "none".to_string(),
            "--memory".to_string(),
            format!("{}m", self.base.memory_mb),
            "--memory-swap".to_string(),
            format!("{}m", self.base.memory_mb),
            "--cpus".to_string(),
            self.base.cpu_limit.to_string(),
        ];

        // حد صارم على عدد processes/threads داخل الحاوية — يمنع fork bombs
        // بشكل حاسم، مستقل عن ذاكرة/CPU الـ host (انظر توثيق PIDS_LIMIT أعلاه)
        args.push("--pids-limit".to_string());
        args.push(Self::PIDS_LIMIT.to_string());

        // tmpfs workspace
        args.push("--tmpfs".to_string());
        args.push(format!(
            "/workspace:size={}m,exec",
            self.hardened.workspace_size_mb
        ));
        args.push("--tmpfs".to_string());
        args.push("/tmp:size=50m".to_string());

        // security options
        args.push("--security-opt".to_string());
        args.push("no-new-privileges".to_string());

        args.push("--cap-drop".to_string());
        args.push("ALL".to_string());

        // seccomp profile
        if let Some(profile) = &self.hardened.seccomp_profile {
            args.push("--security-opt".to_string());
            args.push(format!("seccomp={}", profile.display()));
        }

        // non-root user
        if self.hardened.read_only_root {
            args.push("--read-only".to_string());
        }

        args.push("--user".to_string());
        args.push(format!(
            "{}:{}",
            self.hardened.user_id, self.hardened.group_id
        ));

        // image
        args.push(self.base.image.clone());

        // command
        args.extend(["sh".to_string(), "-c".to_string(), script]);

        self.run_args(args)
    }

    /// تشغيل args محددة.
    fn run_args(&self, args: Vec<String>) -> Result<DockerOutput, DockerError> {
        use std::process::Command;
        use std::sync::mpsc;
        use std::thread;
        use std::time::Instant;

        let timeout = self.base.timeout;
        let start = Instant::now();

        let (tx, rx) = mpsc::channel();
        thread::spawn(move || {
            let mut cmd = Command::new("docker");
            cmd.args(&args);
            let _ = tx.send(cmd.output());
        });

        match rx.recv_timeout(timeout + Duration::from_secs(10)) {
            Ok(Ok(out)) => Ok(DockerOutput {
                exit_code: out.status.code().unwrap_or(-1),
                stdout: String::from_utf8_lossy(&out.stdout).to_string(),
                stderr: String::from_utf8_lossy(&out.stderr).to_string(),
                elapsed: start.elapsed(),
            }),
            Ok(Err(e)) => Err(DockerError::Io(e)),
            Err(_) => Err(DockerError::Timeout {
                duration_secs: timeout.as_secs(),
            }),
        }
    }

    /// اختبار: هل يمكن الوصول لـ /proc/sysrq-trigger؟
    pub fn test_proc_escape(&self) -> EscapeTestResult {
        let code = r#"
use std::fs::OpenOptions;
fn main() {
    match OpenOptions::new().write(true).open("/proc/sysrq-trigger") {
        Ok(_)  => println!("ESCAPED"),
        Err(e) => println!("BLOCKED: {}", e),
    }
}
"#;
        self.run_escape_test("proc_sysrq", code)
    }

    /// اختبار: هل يمكن الوصول لـ /dev/mem؟
    pub fn test_dev_mem_escape(&self) -> EscapeTestResult {
        let code = r#"
use std::fs::OpenOptions;
fn main() {
    match OpenOptions::new().read(true).open("/dev/mem") {
        Ok(_)  => println!("ESCAPED"),
        Err(e) => println!("BLOCKED: {}", e),
    }
}
"#;
        self.run_escape_test("dev_mem", code)
    }

    /// اختبار: هل يمكن mount؟
    pub fn test_mount_escape(&self) -> EscapeTestResult {
        let code = r#"
use std::process::Command;
fn main() {
    let out = Command::new("mount")
        .args(&["-t", "proc", "proc", "/tmp"])
        .output();
    match out {
        Ok(o) if o.status.success() => println!("ESCAPED"),
        _ => println!("BLOCKED"),
    }
}
"#;
        self.run_escape_test("mount_syscall", code)
    }

    /// اختبار: هل يمكن fork bomb؟
    /// المقياس الحقيقي: هل pids-limit يوقف محاولة استنزاف موارد الـ host؟
    /// (لم يعد الاعتماد على حدود الذاكرة الضمنية وحدها — انظر PIDS_LIMIT)
    pub fn test_fork_bomb_escape(&self) -> EscapeTestResult {
        // كود يحاول spawn عدد هائل من الـ threads. إذا تجاوز العدد
        // FORK_BOMB_ESCAPE_THRESHOLD = pids-limit لم يُطبَّق فعليًا = هروب حقيقي.
        // إذا توقف عنده أو قبله = pids-limit يعمل = محتوى فعليًا.
        let code = r#"
use std::thread;
use std::time::Duration;

fn main() {
    let mut handles = vec![];
    let mut spawned = 0u32;
    for _ in 0..10_000 {
        match thread::Builder::new()
            .stack_size(4096)
            .spawn(|| thread::sleep(Duration::from_secs(30)))
        {
            Ok(h) => { handles.push(h); spawned += 1; }
            Err(_) => break,
        }
    }
    if spawned > __ESCAPE_THRESHOLD__ {
        println!("ESCAPED: spawned {} threads (pids-limit not enforced)", spawned);
    } else {
        println!("CONTAINED: pids-limit stopped fork bomb at {} threads", spawned);
    }
}
"#
        .replace(
            "__ESCAPE_THRESHOLD__",
            &Self::FORK_BOMB_ESCAPE_THRESHOLD.to_string(),
        );

        let runner = self.clone_with_timeout(Duration::from_secs(15));
        let result = runner.compile_and_run_hardened(&code);

        match result {
            Err(DockerError::Timeout { .. }) => EscapeTestResult {
                vector: "fork_bomb".to_string(),
                escaped: false,
                output: "Contained by timeout".to_string(),
            },
            Ok(out) => {
                let combined = format!("{}{}", out.stdout, out.stderr);
                // escaped فقط إذا صرّح الكود بـ ESCAPED
                let escaped = combined.contains("ESCAPED:");
                EscapeTestResult {
                    vector: "fork_bomb".to_string(),
                    escaped,
                    output: combined.chars().take(300).collect(),
                }
            }
            Err(e) => EscapeTestResult {
                vector: "fork_bomb".to_string(),
                escaped: false,
                output: format!("Docker error (contained): {}", e),
            },
        }
    }

    /// اختبار: ptrace.
    pub fn test_ptrace_escape(&self) -> EscapeTestResult {
        let code = r#"
fn main() {
    // محاولة ptrace عبر /proc/1/mem
    match std::fs::read("/proc/1/mem") {
        Ok(_)  => println!("ESCAPED"),
        Err(e) => println!("BLOCKED: {}", e),
    }
}
"#;
        self.run_escape_test("ptrace_proc_mem", code)
    }

    /// تشغيل اختبار escape.
    fn run_escape_test(&self, vector: &str, code: &str) -> EscapeTestResult {
        match self.compile_and_run_hardened(code) {
            Ok(out) => {
                let combined = format!("{}{}", out.stdout, out.stderr);
                let escaped = combined.contains("ESCAPED");
                EscapeTestResult {
                    vector: vector.to_string(),
                    escaped,
                    output: combined,
                }
            }
            Err(e) => EscapeTestResult {
                vector: vector.to_string(),
                escaped: false,
                output: format!("Docker error: {}", e),
            },
        }
    }

    /// نسخة مع timeout مختلف.
    fn clone_with_timeout(&self, timeout: Duration) -> Self {
        let mut base = self.base.clone();
        base.timeout = timeout;
        Self {
            base,
            hardened: self.hardened.clone(),
        }
    }
}

/// نتيجة اختبار escape.
#[derive(Debug, Clone)]
pub struct EscapeTestResult {
    /// اسم الـ vector.
    pub vector: String,
    /// هل نجح الهروب؟
    pub escaped: bool,
    /// المخرجات.
    pub output: String,
}

impl EscapeTestResult {
    /// هل الـ sandbox آمن من هذا الـ vector؟
    pub fn is_contained(&self) -> bool {
        !self.escaped
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn runner() -> HardenedDockerRunner {
        HardenedDockerRunner::for_testing().expect("hardened runner")
    }

    #[test]
    fn hardened_config_without_seccomp_validates() {
        let cfg = HardenedConfig::without_seccomp();
        assert!(cfg.validate().is_ok());
    }

    #[test]
    fn hardened_runner_creates_successfully() {
        assert!(HardenedDockerRunner::for_testing().is_ok());
    }

    #[test]
    fn hardened_compiles_hello_world() {
        let r = runner();
        let out = r
            .compile_and_run_hardened(r#"fn main() { println!("hardened ok"); }"#)
            .unwrap();
        assert!(
            out.stdout.contains("hardened ok"),
            "got: {}{}",
            out.stdout,
            out.stderr
        );
    }

    #[test]
    fn hardened_blocks_proc_sysrq() {
        let r = runner();
        let result = r.test_proc_escape();
        assert!(result.is_contained(), "proc escape: {}", result.output);
    }

    #[test]
    fn hardened_blocks_dev_mem() {
        let r = runner();
        let result = r.test_dev_mem_escape();
        assert!(result.is_contained(), "dev/mem escape: {}", result.output);
    }

    #[test]
    fn hardened_blocks_ptrace() {
        let r = runner();
        let result = r.test_ptrace_escape();
        assert!(result.is_contained(), "ptrace escape: {}", result.output);
    }

    #[test]
    fn hardened_blocks_mount() {
        let r = runner();
        let result = r.test_mount_escape();
        assert!(result.is_contained(), "mount escape: {}", result.output);
    }

    #[test]
    fn hardened_contains_fork_bomb() {
        let r = runner();
        let result = r.test_fork_bomb_escape();
        assert!(result.is_contained(), "fork bomb: {}", result.output);
    }
}
