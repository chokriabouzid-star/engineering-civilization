#![forbid(unsafe_code)]

//! تكوين Sandbox وحدود الموارد.

use serde::{Deserialize, Serialize};
use std::time::Duration;

/// حدود استهلاك الموارد للـ sandbox.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ResourceLimits {
    /// الحد الأقصى لاستخدام CPU (بالنسبة المئوية، 0.0-1.0).
    pub max_cpu_percent: f64,
    
    /// الحد الأقصى للذاكرة بالميجابايت.
    pub max_memory_mb: u64,
    
    /// الحد الأقصى لمساحة القرص بالميجابايت.
    pub max_disk_mb: u64,
    
    /// الحد الأقصى لوقت التنفيذ.
    pub max_execution_time: Duration,
}

impl Default for ResourceLimits {
    fn default() -> Self {
        Self {
            max_cpu_percent: 0.5,        // 50% من CPU واحد
            max_memory_mb: 512,          // 512 MB
            max_disk_mb: 100,            // 100 MB
            max_execution_time: Duration::from_secs(30),
        }
    }
}

impl ResourceLimits {
    /// التحقق من صحة الحدود.
    pub fn validate(&self) -> anyhow::Result<()> {
        anyhow::ensure!(
            self.max_cpu_percent > 0.0 && self.max_cpu_percent <= 1.0,
            "max_cpu_percent must be in (0.0, 1.0], got {}",
            self.max_cpu_percent
        );
        anyhow::ensure!(
            self.max_memory_mb > 0,
            "max_memory_mb must be > 0"
        );
        anyhow::ensure!(
            self.max_disk_mb > 0,
            "max_disk_mb must be > 0"
        );
        anyhow::ensure!(
            self.max_execution_time > Duration::from_millis(100),
            "max_execution_time must be > 100ms"
        );
        Ok(())
    }
}

/// سياسة الشبكة للـ sandbox.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[derive(Default)]
pub enum NetworkPolicy {
    /// معزول تماماً — لا اتصال شبكي.
    #[default]
    Isolated,
    
    /// Loopback فقط (localhost).
    LoopbackOnly,
    
    /// قائمة بيضاء من النطاقات المسموحة.
    Allowlist(Vec<String>),
}


/// سياسة Syscalls المسموحة.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[derive(Default)]
pub enum SyscallPolicy {
    /// Allowlist: syscalls محددة فقط.
    Allowlist(Vec<String>),
    
    /// Blocklist: كل syscalls ما عدا هذه.
    Blocklist(Vec<String>),
    
    /// Default safe set (read, write, exit, etc.).
    #[default]
    DefaultSafe,
}


impl SyscallPolicy {
    /// القائمة الآمنة الافتراضية.
    pub fn default_safe_syscalls() -> Vec<String> {
        vec![
            "read".to_string(),
            "write".to_string(),
            "open".to_string(),
            "close".to_string(),
            "stat".to_string(),
            "fstat".to_string(),
            "lstat".to_string(),
            "poll".to_string(),
            "lseek".to_string(),
            "mmap".to_string(),
            "mprotect".to_string(),
            "munmap".to_string(),
            "brk".to_string(),
            "rt_sigaction".to_string(),
            "rt_sigprocmask".to_string(),
            "ioctl".to_string(),
            "access".to_string(),
            "exit".to_string(),
            "exit_group".to_string(),
            "getpid".to_string(),
            "gettid".to_string(),
            "clone".to_string(),
            "futex".to_string(),
        ]
    }
}

/// وضع التشغيل للـ sandbox.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[derive(Default)]
pub enum SandboxMode {
    /// محاكاة — لا تنفيذ حقيقي (للتطوير).
    #[default]
    Simulated,
    
    /// تنفيذ محلي بدون docker.
    Local,
    
    /// تنفيذ في Docker container.
    Docker,
}


/// تكوين Sandbox الكامل.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SandboxConfig {
    /// وضع التشغيل.
    pub mode: SandboxMode,
    
    /// حدود الموارد.
    pub limits: ResourceLimits,
    
    /// سياسة الشبكة.
    pub network: NetworkPolicy,
    
    /// سياسة Syscalls.
    pub syscalls: SyscallPolicy,
    
    /// عدد التشغيلات لقياس reproducibility.
    pub runs_for_reproducibility: usize,
}

impl Default for SandboxConfig {
    fn default() -> Self {
        Self {
            mode: SandboxMode::Simulated,
            limits: ResourceLimits::default(),
            network: NetworkPolicy::Isolated,
            syscalls: SyscallPolicy::DefaultSafe,
            runs_for_reproducibility: 3,
        }
    }
}

impl SandboxConfig {
    /// بناء تكوين جديد مع الوضع المحدد.
    pub fn new(mode: SandboxMode) -> Self {
        Self {
            mode,
            ..Default::default()
        }
    }
    
    /// التحقق من صحة التكوين.
    pub fn validate(&self) -> anyhow::Result<()> {
        self.limits.validate()?;
        
        anyhow::ensure!(
            self.runs_for_reproducibility >= 1,
            "runs_for_reproducibility must be >= 1"
        );
        
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_config_is_valid() {
        let config = SandboxConfig::default();
        assert!(config.validate().is_ok());
    }

    #[test]
    fn default_mode_is_simulated() {
        let config = SandboxConfig::default();
        assert_eq!(config.mode, SandboxMode::Simulated);
    }

    #[test]
    fn default_network_is_isolated() {
        let config = SandboxConfig::default();
        assert_eq!(config.network, NetworkPolicy::Isolated);
    }

    #[test]
    fn invalid_cpu_percent_fails() {
        let mut limits = ResourceLimits::default();
        limits.max_cpu_percent = 1.5;
        assert!(limits.validate().is_err());
    }

    #[test]
    fn zero_memory_fails() {
        let mut limits = ResourceLimits::default();
        limits.max_memory_mb = 0;
        assert!(limits.validate().is_err());
    }
}
