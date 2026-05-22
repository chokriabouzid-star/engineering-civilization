#![forbid(unsafe_code)]
#![deny(missing_docs)]
//! النموذج المعرفي لعدم اليقين للمشروع.
//!
//! يوفر هذا الصندوق أدوات لتمثيل الثقة والأدلة وعدم اليقين
//! بالإضافة إلى آليات الدمج المحافظ والتلاشي الزمني.

/// موديول المعايرة وحساب خطأ المعايرة المتوقع.
pub mod calibration;
/// موديول التلاشي الزمني للثقة بناءً على عمر النصف.
pub mod decay;
/// موديول تعريف الأخطاء والتحقق من النطاقات.
pub mod error;
/// موديول دمج الحالات المعرفية بشكل محافظ.
pub mod propagation;
/// موديول تعريف الحالات الأساسية والأدلة وتحليل عدم اليقين.
pub mod state;
/// Week 35: Bayesian Evidence — إضافة فقط
pub mod bayesian;

pub use calibration::*;
pub use decay::*;
pub use error::*;
pub use propagation::*;
pub use state::*;
pub use bayesian::BayesianEvidence;
