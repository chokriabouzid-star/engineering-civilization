#![forbid(unsafe_code)]

//! قانون الواقع المقاس — ADR-033.
//!
//! يجعل التنفيذ الحقيقي شرطًا دستوريًا غير قابل للتعويض: لا قبول بلا دليل مقاس،
//! ولا دليل بلا مصدر Docker، ولا دليل يحمل فشلًا مقيسًا.

use crate::invariant::{Invariant, ViolationReport};
use ec_epistemic::{EpistemicState, ExecutionProvenance};
use ec_fitness::FitnessVector;

/// الحد الأدنى لعدد التشغيلات الناجحة لمنح سلطة القبول.
///
/// منسوخ من عتبة `RealityVector::is_trustworthy`: `empirical_confidence > 0.8`
/// يعني ≥ 3 تشغيلات تحت الصيغة الموروثة `runs / (runs + 0.5)`.
/// تغييره قرار دستوري.
pub const MIN_RUNS_FOR_AUTHORITY: usize = 3;

/// الحد الأدنى لقابلية إعادة الإنتاج لمنح سلطة القبول.
///
/// منسوخ من `RealityVector::is_reproducible`: `reproducibility > 0.95` —
/// **سقف حصري**؛ `0.95` بالضبط لا يمنح سلطة.
/// فحص رموز الخروج يسبق هذا الفحص دائمًا.
pub const MIN_REPRODUCIBILITY_FOR_AUTHORITY: f64 = 0.95;

/// قانون يلزم بوجود دليل تنفيذ حقيقي قبل أي قبول.
#[derive(Debug, Clone, Copy, Default)]
pub struct MeasuredRealityInvariant;

impl Invariant for MeasuredRealityInvariant {
    fn name(&self) -> &'static str {
        "MeasuredRealityInvariant"
    }

    fn check(
        &self,
        _fitness: &FitnessVector,
        epistemic: &EpistemicState,
    ) -> Result<(), ViolationReport> {
        let m = match epistemic.measured.as_ref() {
            Some(m) => m,
            None => {
                return Err(ViolationReport::new(
                    self.name(),
                    "no measured evidence: acceptance requires real execution",
                    0.0,
                    1.0,
                    true,
                ))
            }
        };

        if m.provenance() != ExecutionProvenance::Docker {
            return Err(ViolationReport::new(
                self.name(),
                format!(
                    "provenance {:?} confers no authority; only Docker execution does",
                    m.provenance()
                ),
                0.0,
                1.0,
                true,
            ));
        }

        // ع1: فحص الخروج **قبل** فحص عدد التشغيلات — حتى يظهر السبب المقيس الأقوى،
        // ولا يُخفى خلف «لدينا تشغيل واحد فقط».
        if !m.all_succeeded() {
            return Err(ViolationReport::new(
                self.name(),
                format!(
                    "exit codes {:?} in {}/{} runs",
                    m.exit_codes(),
                    m.failed_runs(),
                    m.runs_completed()
                ),
                m.failed_runs() as f64,
                0.0,
                true,
            ));
        }

        if m.runs_completed() < MIN_RUNS_FOR_AUTHORITY {
            return Err(ViolationReport::new(
                self.name(),
                format!(
                    "insufficient runs: {} of {} required for authority",
                    m.runs_completed(),
                    MIN_RUNS_FOR_AUTHORITY
                ),
                m.runs_completed() as f64,
                MIN_RUNS_FOR_AUTHORITY as f64,
                true,
            ));
        }

        if m.reproducibility() <= MIN_REPRODUCIBILITY_FOR_AUTHORITY {
            return Err(ViolationReport::new(
                self.name(),
                format!(
                    "reproducibility {:.2} not above minimum {:.2}",
                    m.reproducibility(),
                    MIN_REPRODUCIBILITY_FOR_AUTHORITY
                ),
                m.reproducibility(),
                MIN_REPRODUCIBILITY_FOR_AUTHORITY,
                true,
            ));
        }

        Ok(())
    }
}
