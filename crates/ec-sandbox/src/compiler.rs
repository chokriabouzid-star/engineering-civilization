#![forbid(unsafe_code)]

//! Rust compilation + execution داخل Docker — Week 14
//!
//! الكود يُمرَّر مباشرة لـ DockerRunner بدون host filesystem.

use crate::docker::{DockerError, DockerOutput, DockerRunner};
use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};
use std::time::Duration;

/// نتيجة تشغيل واحد.
#[derive(Debug, Clone)]
pub struct RunOutput {
    /// stdout من البرنامج.
    pub stdout: String,
    /// stderr.
    pub stderr: String,
    /// كود الخروج.
    pub exit_code: i32,
    /// وقت التنفيذ.
    pub elapsed: Duration,
}

impl RunOutput {
    /// hash المحتوى للمقارنة.
    pub fn content_hash(&self) -> u64 {
        let mut h = DefaultHasher::new();
        self.stdout.hash(&mut h);
        self.exit_code.hash(&mut h);
        h.finish()
    }

    /// هل نجح؟
    pub fn success(&self) -> bool {
        self.exit_code == 0
    }
}

impl From<DockerOutput> for RunOutput {
    fn from(d: DockerOutput) -> Self {
        Self {
            stdout: d.stdout,
            stderr: d.stderr,
            exit_code: d.exit_code,
            elapsed: d.elapsed,
        }
    }
}

/// نتيجة compilation + execution.
#[derive(Debug, Clone)]
pub enum CompilationResult {
    /// فشل compilation.
    Failed {
        /// رسالة الخطأ.
        stderr: String,
    },
    /// نجح compilation والتنفيذ.
    Success {
        /// نتائج كل run.
        runs: Vec<RunOutput>,
    },
}

impl CompilationResult {
    /// هل نجحت؟
    pub fn succeeded(&self) -> bool {
        matches!(self, Self::Success { .. })
    }

    /// الـ runs.
    pub fn runs(&self) -> &[RunOutput] {
        match self {
            Self::Success { runs } => runs,
            Self::Failed { .. } => &[],
        }
    }
}

/// Rust compiler داخل Docker.
pub struct RustSandboxCompiler {
    runner: DockerRunner,
    runs: usize,
}

impl RustSandboxCompiler {
    /// بناء compiler جديد.
    pub fn new(runner: DockerRunner, runs: usize) -> Self {
        Self {
            runner,
            runs: runs.max(1),
        }
    }

    /// compile + run source code.
    pub fn compile_and_run(&self, source_code: &str) -> Result<CompilationResult, DockerError> {
        // تشغيل أول مرة: compilation + execution
        let first = self.runner.compile_and_run_code(source_code)?;

        // إذا فشل (لا يحتوي ---OUTPUT--- = فشل compilation)
        if !first.stdout.contains("---OUTPUT---") && first.exit_code != 0 {
            return Ok(CompilationResult::Failed {
                stderr: format!("{}{}", first.stdout, first.stderr),
            });
        }

        // استخراج output البرنامج فقط
        let program_output = extract_program_output(&first.stdout);
        let first_run = RunOutput {
            stdout: program_output,
            stderr: first.stderr.clone(),
            exit_code: if first.stdout.contains("---OUTPUT---") {
                0
            } else {
                first.exit_code
            },
            elapsed: first.elapsed,
        };

        let mut runs = vec![first_run];

        // تشغيلات إضافية للـ reproducibility
        for _ in 1..self.runs {
            let out = self.runner.compile_and_run_code(source_code)?;
            let program_output = extract_program_output(&out.stdout);
            runs.push(RunOutput {
                stdout: program_output,
                stderr: out.stderr,
                exit_code: if out.stdout.contains("---OUTPUT---") {
                    0
                } else {
                    out.exit_code
                },
                elapsed: out.elapsed,
            });
        }

        Ok(CompilationResult::Success { runs })
    }
}

/// استخراج output البرنامج بعد marker.
fn extract_program_output(raw: &str) -> String {
    if let Some(pos) = raw.find("---OUTPUT---") {
        raw[pos + "---OUTPUT---".len()..]
            .trim_start_matches('\n')
            .to_string()
    } else {
        raw.to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::docker::DockerRunner;

    fn compiler(runs: usize) -> RustSandboxCompiler {
        RustSandboxCompiler::new(DockerRunner::default(), runs)
    }

    #[test]
    fn compiles_hello_world() {
        let result = compiler(1)
            .compile_and_run(r#"fn main() { println!("hello world"); }"#)
            .unwrap();

        assert!(result.succeeded(), "got: {:?}", result);
        assert!(result.runs()[0].stdout.contains("hello world"));
    }

    #[test]
    fn fails_on_syntax_error() {
        let result = compiler(1).compile_and_run("this is not rust").unwrap();

        assert!(!result.succeeded());
    }

    #[test]
    fn runs_multiple_times() {
        let result = compiler(3)
            .compile_and_run(r#"fn main() { println!("stable"); }"#)
            .unwrap();

        assert!(result.succeeded());
        assert_eq!(result.runs().len(), 3);
    }

    #[test]
    fn deterministic_output_has_same_hash() {
        let result = compiler(3)
            .compile_and_run(r#"fn main() { println!("deterministic"); }"#)
            .unwrap();

        let runs = result.runs();
        let h0 = runs[0].content_hash();
        let h1 = runs[1].content_hash();
        let h2 = runs[2].content_hash();

        assert_eq!(h0, h1);
        assert_eq!(h1, h2);
    }

    #[test]
    fn network_access_fails_inside_container() {
        let result = compiler(1)
            .compile_and_run(
                r#"
use std::net::TcpStream;
fn main() {
    match TcpStream::connect("8.8.8.8:80") {
        Ok(_)  => println!("CONNECTED"),
        Err(e) => println!("BLOCKED: {}", e),
    }
}
"#,
            )
            .unwrap();

        if result.succeeded() {
            assert!(
                result.runs()[0].stdout.contains("BLOCKED"),
                "got: {}",
                result.runs()[0].stdout
            );
        }
    }
}
