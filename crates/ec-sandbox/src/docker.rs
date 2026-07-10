#![forbid(unsafe_code)]

//! Docker CLI wrapper — Week 14
//!
//! Strategy: الكود يُمرَّر عبر shell script مباشرة.
//! لا bind mount — tmpfs فقط داخل container.

use std::process::Command;
use std::time::{Duration, Instant};

/// Image الافتراضية.
pub const DEFAULT_IMAGE: &str = "rust:1.75-slim";

/// مخرجات تشغيل Docker container.
#[derive(Debug, Clone)]
pub struct DockerOutput {
    /// كود الخروج.
    pub exit_code: i32,
    /// stdout.
    pub stdout: String,
    /// stderr.
    pub stderr: String,
    /// وقت التنفيذ.
    pub elapsed: Duration,
}

impl DockerOutput {
    /// هل نجح التنفيذ؟
    pub fn success(&self) -> bool {
        self.exit_code == 0
    }
}

/// أخطاء Docker.
#[derive(Debug, thiserror::Error)]
pub enum DockerError {
    /// Docker غير متاح.
    #[error("Docker not available: {0}")]
    NotAvailable(String),

    /// انتهى timeout.
    #[error("Container timeout after {duration_secs}s")]
    Timeout {
        /// عدد الثواني.
        duration_secs: u64,
    },

    /// IO error.
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    /// Docker daemon error.
    #[error("Docker daemon error (exit {exit_code}): {stderr}")]
    DaemonError {
        /// exit code من docker.
        exit_code: i32,
        /// رسالة الخطأ.
        stderr: String,
    },
}

/// Docker runner — يشغّل Rust code داخل container معزول.
#[derive(Debug, Clone)]
pub struct DockerRunner {
    /// Docker image.
    pub image: String,
    /// memory limit بالميجابايت.
    pub memory_mb: u64,
    /// cpu limit (0.0-1.0).
    pub cpu_limit: f64,
    /// timeout.
    pub timeout: Duration,
}

impl Default for DockerRunner {
    fn default() -> Self {
        Self {
            image: DEFAULT_IMAGE.to_string(),
            memory_mb: 512,
            cpu_limit: 0.5,
            timeout: Duration::from_secs(60),
        }
    }
}

impl DockerRunner {
    /// الحد الأقصى لعدد الـ PIDs (processes/threads) داخل الحاوية عبر
    /// cgroup pids controller.
    ///
    /// اكتُشفت الحاجة لهذا في Phase 1 CI (GitHub Actions): الاحتواء كان
    /// يعتمد ضمنيًا على حدود الذاكرة/CPU فقط، وهذا غير كافٍ لمنع fork bomb —
    /// `--memory` لا تحدّ عدد الـ threads (task_struct allocations bypass
    /// memory accounting). على بيئة أقوى من WSL2 (4 CPU / 15.6GB) تمكّن
    /// اختبار fork bomb من إنشاء 10,000 thread دون أي حد.
    ///
    /// القيمة 256: هامش مريح فوق احتياجات تجميع/تشغيل Rust معقول (rustc +
    /// linker + البرنامج الناتج)، وصغيرة كفاية لإيقاف أي fork bomb حقيقي.
    /// إن كسرت اختبارات التجميع العادية، ارفعها إلى 512.
    pub const PIDS_LIMIT: u32 = 256;

    /// إنشاء runner جديد.
    pub fn new(image: &str, memory_mb: u64, cpu_limit: f64, timeout: Duration) -> Self {
        Self {
            image: image.to_string(),
            memory_mb,
            cpu_limit,
            timeout,
        }
    }

    /// compile + run source code داخل container.
    ///
    /// الكود يُكتب في tmpfs داخل container فقط.
    /// لا host filesystem مُعرَّض.
    pub fn compile_and_run_code(&self, source_code: &str) -> Result<DockerOutput, DockerError> {
        self.check_docker_available()?;

        // escape single quotes في الكود
        let escaped = source_code.replace('\'', r#"'"'"'"#);

        let script = format!(
            "printf '%s' '{escaped}' > /workspace/main.rs && \
             rustc /workspace/main.rs -o /workspace/program 2>&1 && \
             echo '---OUTPUT---' && \
             /workspace/program"
        );

        let start = Instant::now();

        let mut cmd = Command::new("docker");
        cmd.args([
            "run",
            "--rm",
            "--network",
            "none",
            "--memory",
            &format!("{}m", self.memory_mb),
            "--memory-swap",
            &format!("{}m", self.memory_mb),
            "--cpus",
            &self.cpu_limit.to_string(),
            "--pids-limit",
            &Self::PIDS_LIMIT.to_string(),
            "--tmpfs",
            "/workspace:size=200m,exec",
            "--tmpfs",
            "/tmp:size=50m",
            "--security-opt",
            "no-new-privileges",
            "--cap-drop",
            "ALL",
            &self.image,
            "sh",
            "-c",
            &script,
        ]);

        let output = self.run_with_timeout(cmd)?;
        let elapsed = start.elapsed();

        // exit 125 = docker daemon failure
        if output.exit_code == 125 {
            return Err(DockerError::DaemonError {
                exit_code: output.exit_code,
                stderr: output.stderr.clone(),
            });
        }

        Ok(DockerOutput {
            exit_code: output.exit_code,
            stdout: output.stdout,
            stderr: output.stderr,
            elapsed,
        })
    }

    /// تشغيل command بسيط للاختبار.
    pub fn run_simple(&self, command: &[&str]) -> Result<DockerOutput, DockerError> {
        self.check_docker_available()?;

        let start = Instant::now();

        let mut cmd = Command::new("docker");
        cmd.args([
            "run",
            "--rm",
            "--network",
            "none",
            "--memory",
            &format!("{}m", self.memory_mb),
            "--memory-swap",
            &format!("{}m", self.memory_mb),
            "--cpus",
            &self.cpu_limit.to_string(),
            "--pids-limit",
            &Self::PIDS_LIMIT.to_string(),
            "--tmpfs",
            "/tmp:size=50m",
            "--security-opt",
            "no-new-privileges",
            "--cap-drop",
            "ALL",
            &self.image,
        ]);
        cmd.args(command);

        let mut output = self.run_with_timeout(cmd)?;
        output.elapsed = start.elapsed();

        Ok(output)
    }

    /// تشغيل مع timeout.
    fn run_with_timeout(&self, mut cmd: Command) -> Result<DockerOutput, DockerError> {
        use std::sync::mpsc;
        use std::thread;

        let timeout = self.timeout;
        let (tx, rx) = mpsc::channel();

        thread::spawn(move || {
            let _ = tx.send(cmd.output());
        });

        match rx.recv_timeout(timeout + Duration::from_secs(5)) {
            Ok(Ok(out)) => Ok(DockerOutput {
                exit_code: out.status.code().unwrap_or(-1),
                stdout: String::from_utf8_lossy(&out.stdout).to_string(),
                stderr: String::from_utf8_lossy(&out.stderr).to_string(),
                elapsed: Duration::ZERO,
            }),
            Ok(Err(e)) => Err(DockerError::Io(e)),
            Err(_) => Err(DockerError::Timeout {
                duration_secs: timeout.as_secs(),
            }),
        }
    }

    /// تحقق من Docker.
    fn check_docker_available(&self) -> Result<(), DockerError> {
        let result = Command::new("docker")
            .args(["info", "--format", "{{.ServerVersion}}"])
            .output();

        match result {
            Ok(o) if o.status.success() => Ok(()),
            Ok(o) => Err(DockerError::NotAvailable(
                String::from_utf8_lossy(&o.stderr).to_string(),
            )),
            Err(e) => Err(DockerError::NotAvailable(e.to_string())),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn runner() -> DockerRunner {
        DockerRunner::default()
    }

    #[test]
    fn docker_runner_default_values() {
        let r = runner();
        assert_eq!(r.image, DEFAULT_IMAGE);
        assert_eq!(r.memory_mb, 512);
    }

    #[test]
    #[cfg_attr(
        not(feature = "docker_tests"),
        ignore = "requires --features docker_tests"
    )]
    fn docker_available() {
        assert!(runner().check_docker_available().is_ok());
    }

    #[test]
    #[cfg_attr(
        not(feature = "docker_tests"),
        ignore = "requires --features docker_tests"
    )]
    fn docker_runs_echo() {
        let out = runner().run_simple(&["echo", "hello"]).unwrap();
        assert!(out.success());
        assert!(out.stdout.contains("hello"));
    }

    #[test]
    #[cfg_attr(
        not(feature = "docker_tests"),
        ignore = "requires --features docker_tests"
    )]
    fn docker_network_is_isolated() {
        let out = runner()
            .run_simple(&["sh", "-c", "wget -q google.com 2>&1 || echo BLOCKED"])
            .unwrap();
        let combined = format!("{}{}", out.stdout, out.stderr);
        assert!(combined.contains("BLOCKED") || !out.success());
    }

    #[test]
    #[cfg_attr(
        not(feature = "docker_tests"),
        ignore = "requires --features docker_tests"
    )]
    fn docker_workspace_tmpfs_is_writable() {
        // tmpfs /workspace يجب أن يكون writable
        let out = runner()
            .run_simple(&[
                "sh",
                "-c",
                "mkdir -p /workspace && echo test > /workspace/t.txt && cat /workspace/t.txt",
            ])
            .unwrap();
        // run_simple لا يُضيف tmpfs /workspace
        // هذا يختبر أن tmpfs يعمل عبر compile_and_run_code
        let out2 = runner()
            .compile_and_run_code(r#"fn main() { println!("workspace_ok"); }"#)
            .unwrap();
        assert!(
            out2.stdout.contains("workspace_ok"),
            "got: {}{}",
            out2.stdout,
            out2.stderr
        );
        let _ = out; // suppress unused
    }

    #[test]
    #[cfg_attr(
        not(feature = "docker_tests"),
        ignore = "requires --features docker_tests"
    )]
    fn docker_compiles_hello_world() {
        let out = runner()
            .compile_and_run_code(r#"fn main() { println!("hello from docker"); }"#)
            .unwrap();
        assert!(
            out.stdout.contains("hello from docker"),
            "got: {}{}",
            out.stdout,
            out.stderr
        );
    }

    /// اختبار أمني: يتحقق أن --pids-limit مفعّل ويمنع fork bomb.
    /// اكتُشفت الحاجة له في Phase 1 CI بعد فشل gate_escape_vector_5.
    #[test]
    #[cfg_attr(
        not(feature = "docker_tests"),
        ignore = "requires --features docker_tests"
    )]
    fn docker_pids_limit_blocks_fork_bomb() {
        let code = r#"
use std::thread;
use std::time::Duration;
fn main() {
    let mut spawned = 0u32;
    let mut handles = vec![];
    for _ in 0..10_000 {
        match thread::Builder::new()
            .stack_size(4096)
            .spawn(|| thread::sleep(Duration::from_secs(30)))
        {
            Ok(h) => { handles.push(h); spawned += 1; }
            Err(_) => break,
        }
    }
    println!("SPAWNED:{}", spawned);
}
"#;
        let out = runner().compile_and_run_code(code).unwrap();
        let combined = format!("{}{}", out.stdout, out.stderr);
        // pids-limit=256، فأي عدد > 306 (256 + 50 هامش) يعني الحد لا يعمل
        let has_high_spawn = combined.lines().any(|line| {
            if let Some(n) = line.strip_prefix("SPAWNED:") {
                n.trim().parse::<u32>().map(|v| v > 306).unwrap_or(false)
            } else {
                false
            }
        });
        assert!(!has_high_spawn, "pids-limit not enforced: {}", combined);
    }
}
