#![forbid(unsafe_code)]

//! تصنيف نتائج التنفيذ لمسار التحكيم — ADR-033 / PR2.
//!
//! يحوّل نتيجة `RustSandboxCompiler` إلى نوع خام صادق: إمّا `ProgramRuns`
//! (البرنامج بدأ وانتهى بحالة خروج مرصودة) أو `ExecutionFailed`
//! (لم تُنتَج نتيجة برنامج أصلًا). لا يُنتج هذا الملف أي نوع معرفي.

use crate::compiler::{CompilationResult, RunOutput, RustSandboxCompiler};
use crate::config::{SandboxConfig, SandboxMode};
use crate::docker::{DockerError, DockerRunner, DEFAULT_IMAGE};
use crate::hardened::{HardenedConfig, HardenedDockerRunner};

/// سبب فشل التنفيذ — نوع مُصنَّف، بلا تفصيل خام في السبب.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ExecutionFailure {
    /// Docker غير متاح: لم يبدأ التنفيذ.
    DockerUnavailable,
    /// خطأ إدخال/إخراج أثناء التشغيل.
    RunnerIo,
    /// فشل تهيئة المشغّل المُحصَّن.
    RunnerSetup,
    /// انتهت المهلة قبل أن تُنتَج نتيجة.
    ContainerTimeout { duration_secs: u64 },
    /// خطأ من docker daemon.
    DaemonError,
    /// لم تظهر علامة ما بعد نجاح الترجمة؛ لا توجد نتيجة برنامج مؤكدة.
    NoProgramResult,
    /// رمز خروج خارج `0..=255`: لا حالة خروج برنامج.
    InvalidExitCode { code: i32 },
    /// لا تشغيلات مكتملة.
    EmptyRuns,
}

impl ExecutionFailure {
    /// السبب العام المُبلَّغ — ثابت ومحدود، بلا تفصيل حساس.
    pub fn reason(&self) -> String {
        match self {
            Self::DockerUnavailable => "docker unavailable: execution could not start".to_string(),
            Self::RunnerIo => "runner I/O failure during execution".to_string(),
            Self::RunnerSetup => "sandbox runner setup failed".to_string(),
            Self::ContainerTimeout { duration_secs } => {
                format!("container timeout after {}s", duration_secs)
            }
            Self::DaemonError => "docker daemon error during execution".to_string(),
            Self::NoProgramResult => "no program result was observed".to_string(),
            Self::InvalidExitCode { code } => {
                format!("exit code {} outside 0..=255: no program exit status", code)
            }
            Self::EmptyRuns => "no completed runs were observed".to_string(),
        }
    }
}

/// نتيجة تنفيذ خام لمسار التحكيم.
#[derive(Debug, Clone, PartialEq)]
pub enum SandboxOutcome {
    /// البرنامج بدأ وانتهى بحالات خروج مرصودة.
    ProgramRuns {
        /// رموز الخروج بترتيب التشغيلات.
        exit_codes: Vec<i32>,
        /// معامل التكرار الصارم على `(stdout, stderr, exit_code)`.
        reproducibility: f64,
        /// صورة الحاوية التي استُعملت فعلًا.
        image: String,
    },
    /// لم تُنتَج نتيجة برنامج.
    ExecutionFailed {
        /// السبب المُصنَّف.
        failure: ExecutionFailure,
    },
}

/// تكرار صارم: يقارن `(stdout, stderr, exit_code)` مباشرة بلا تجزئة.
///
/// دالة جديدة خاصة بمسار التحكيم. `metrics::measure_reproducibility` القائمة
/// تهمل `stderr` ولا تُعدَّل (سلوك المسارات القديمة محفوظ).
pub fn measure_reproducibility_strict(runs: &[RunOutput]) -> f64 {
    if runs.is_empty() {
        return 0.0;
    }
    if runs.len() == 1 {
        return if runs[0].success() { 1.0 } else { 0.0 };
    }
    let first = &runs[0];
    let identical = runs
        .iter()
        .filter(|r| {
            r.stdout == first.stdout && r.stderr == first.stderr && r.exit_code == first.exit_code
        })
        .count();
    identical as f64 / runs.len() as f64
}

/// تصنيف نتيجة المترجم. دالة نقية — قابلة للاختبار بلا Docker.
///
/// **EC-ADJ-01 (مصحّح):** `-1` يعني أن عملية `docker` المحلية لم تُرجع رمز
/// خروج (`ExitStatus::code() == None`)، و**ليس** أن البرنامج «قُتل بإشارة».
/// القيم `101` و`134` و`137` المقاسة هي **حالات خروج مرصودة** داخل
/// `0..=255`؛ لا يميّز الرقم وحده بين `exit(137)` و`OOM-kill`، فلا تُكتب
/// في أي سبب أو تعليق أنها «OOM» أو «إشارة».
pub fn classify(result: Result<CompilationResult, DockerError>, image: String) -> SandboxOutcome {
    let failed = |failure| SandboxOutcome::ExecutionFailed { failure };

    let compilation = match result {
        Ok(c) => c,
        Err(DockerError::NotAvailable(_)) => return failed(ExecutionFailure::DockerUnavailable),
        Err(DockerError::Timeout { duration_secs }) => {
            return failed(ExecutionFailure::ContainerTimeout { duration_secs })
        }
        Err(DockerError::Io(_)) => return failed(ExecutionFailure::RunnerIo),
        Err(DockerError::DaemonError { .. }) => return failed(ExecutionFailure::DaemonError),
    };

    let runs = match compilation {
        CompilationResult::Failed { .. } => return failed(ExecutionFailure::NoProgramResult),
        CompilationResult::Success { runs } => runs,
    };

    if runs.is_empty() {
        return failed(ExecutionFailure::EmptyRuns);
    }

    if let Some(bad) = runs.iter().find(|r| !(0..=255).contains(&r.exit_code)) {
        return failed(ExecutionFailure::InvalidExitCode {
            code: bad.exit_code,
        });
    }

    let reproducibility = measure_reproducibility_strict(&runs);
    let exit_codes = runs.iter().map(|r| r.exit_code).collect();

    SandboxOutcome::ProgramRuns {
        exit_codes,
        reproducibility,
        image,
    }
}

/// مشغّل التنفيذ المقاس — الصورة مثبَّتة، والحدود من التكوين الافتراضي.
pub struct MeasuredSandboxRunner {
    runs: usize,
}

impl MeasuredSandboxRunner {
    /// بناء مشغّل بعدد تشغيلات لا يقل عن واحد.
    pub fn new(runs: usize) -> Self {
        Self { runs: runs.max(1) }
    }

    /// تنفيذ الكود وإرجاع النتيجة الخام المُصنَّفة.
    pub fn execute(&self, code: &str) -> SandboxOutcome {
        let config = SandboxConfig::new(SandboxMode::Docker);
        let base = DockerRunner::new(
            DEFAULT_IMAGE,
            config.limits.max_memory_mb,
            config.limits.max_cpu_percent,
            config.limits.max_execution_time,
        );
        let hardened = match HardenedDockerRunner::new(base, HardenedConfig::default()) {
            Ok(h) => h,
            Err(_) => {
                return SandboxOutcome::ExecutionFailed {
                    failure: ExecutionFailure::RunnerSetup,
                }
            }
        };
        let compiler = RustSandboxCompiler::new(hardened, self.runs);
        classify(compiler.compile_and_run(code), DEFAULT_IMAGE.to_string())
    }
}
