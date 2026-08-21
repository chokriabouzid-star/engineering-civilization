#![forbid(unsafe_code)]

//! Docker CLI wrapper — Week 14
//!
//! يوفر `DockerRunner` كحامل إعدادات (image/memory/cpu/timeout) يُستخدَم
//! كأساس لـ `HardenedDockerRunner` (crate::hardened). لا تنفيذ فعلي لكود
//! غير موثوق من هذا الملف مباشرة — انظر ADR-024 (F1): مسار التنفيذ غير
//! المُحصَّن (`compile_and_run_code`/`run_simple`) أُزيل عمدًا؛ الحصانة
//! (seccomp + cap-drop + read-only + non-root + pids-limit) إلزامية الآن
//! لا اختيارية.

use std::process::Command;
use std::time::Duration;

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

/// حامل إعدادات Docker الأساسية (image/موارد/timeout).
///
/// **لا تنفيذ فعلي هنا.** يُستخدَم حصرًا كـ `base` لـ
/// `hardened::HardenedDockerRunner`، وهو المسار الوحيد المسموح به لتنفيذ
/// كود غير موثوق (راجع ADR-024 §F1).
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

    /// تحقق من Docker.
    pub fn check_docker_available(&self) -> Result<(), DockerError> {
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
}
