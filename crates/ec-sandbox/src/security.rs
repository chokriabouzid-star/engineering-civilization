#![forbid(unsafe_code)]

//! اكتشاف الانتهاكات الأمنية.

use serde::{Deserialize, Serialize};

/// أنواع الانتهاكات الأمنية.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum SecurityViolation {
    /// محاولة الهروب من Sandbox.
    SandboxEscape {
        /// الطريقة المستخدمة.
        method: String,
    },

    /// Syscall محظور.
    ForbiddenSyscall {
        /// اسم Syscall.
        syscall: String,
    },

    /// تجاوز حد الموارد.
    ResourceLimitExceeded {
        /// نوع المورد.
        resource: String,
        /// القيمة الفعلية.
        value: u64,
        /// الحد الأقصى.
        limit: u64,
    },

    /// محاولة الوصول لملف محظور.
    UnauthorizedFileAccess {
        /// مسار الملف.
        path: String,
    },

    /// محاولة اتصال شبكي غير مسموح.
    UnauthorizedNetworkAccess {
        /// العنوان المستهدف.
        target: String,
    },
}

impl SecurityViolation {
    /// هل الانتهاك catastrophic (يجب إيقاف التنفيذ فوراً)؟
    pub fn is_catastrophic(&self) -> bool {
        matches!(
            self,
            SecurityViolation::SandboxEscape { .. }
                | SecurityViolation::UnauthorizedFileAccess { .. }
        )
    }

    /// وصف الانتهاك.
    pub fn description(&self) -> String {
        match self {
            Self::SandboxEscape { method } => {
                format!("Sandbox escape attempt via: {}", method)
            }
            Self::ForbiddenSyscall { syscall } => {
                format!("Forbidden syscall: {}", syscall)
            }
            Self::ResourceLimitExceeded {
                resource,
                value,
                limit,
            } => {
                format!("{} exceeded: {} > {}", resource, value, limit)
            }
            Self::UnauthorizedFileAccess { path } => {
                format!("Unauthorized file access: {}", path)
            }
            Self::UnauthorizedNetworkAccess { target } => {
                format!("Unauthorized network access: {}", target)
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sandbox_escape_is_catastrophic() {
        let violation = SecurityViolation::SandboxEscape {
            method: "ptrace".to_string(),
        };
        assert!(violation.is_catastrophic());
    }

    #[test]
    fn forbidden_syscall_is_not_catastrophic() {
        let violation = SecurityViolation::ForbiddenSyscall {
            syscall: "reboot".to_string(),
        };
        assert!(!violation.is_catastrophic());
    }

    #[test]
    fn resource_limit_is_not_catastrophic() {
        let violation = SecurityViolation::ResourceLimitExceeded {
            resource: "memory".to_string(),
            value: 1024,
            limit: 512,
        };
        assert!(!violation.is_catastrophic());
    }

    #[test]
    fn unauthorized_file_access_is_catastrophic() {
        let violation = SecurityViolation::UnauthorizedFileAccess {
            path: "/etc/passwd".to_string(),
        };
        assert!(violation.is_catastrophic());
    }

    #[test]
    fn violation_description_is_informative() {
        let violation = SecurityViolation::ForbiddenSyscall {
            syscall: "kill".to_string(),
        };
        let desc = violation.description();
        assert!(desc.contains("kill"));
        assert!(desc.contains("Forbidden syscall"));
    }
}
