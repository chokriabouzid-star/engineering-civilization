//! بوابة نقاء النواة (Kernel Purity Gate) — ADR-023 / ADR-027
//!
//! تضمن أن `ec-constitutional` لا تعتمد إطلاقًا على tokio/async على مسار
//! الربط التشغيلي (normal dependencies).
//!
//! نستخدم `cargo tree -e normal` = شجرة الربط الحقيقية:
//!   - تحترم features والهدف target
//!   - تستبعد اعتمادات الاختبار (dev) والبناء (build) — لا إيجابيات كاذبة
//!   - مستقلة عن صيغة معرّفات cargo الداخلية (تغيّرت حديثًا)
//!
//! التحكّم السلبي: أضف `tokio = "1"` تحت [dependencies] في
//! crates/ec-constitutional/Cargo.toml ثم شغّل هذا الاختبار → يجب أن يفشل.

use std::process::Command;

const FORBIDDEN_NAMES: &[&str] = &["tokio", "tokio-util", "async-trait"];

#[test]
fn ec_constitutional_never_depends_on_tokio() {
    let cargo = std::env::var("CARGO").unwrap_or_else(|_| "cargo".to_string());

    let output = Command::new(&cargo)
        .args([
            "tree",
            "-p",
            "ec-constitutional",
            "-e",
            "normal",
            "--prefix",
            "none",
            "--format",
            "{p}",
        ])
        .output()
        .expect("فشل تشغيل `cargo tree` — fail-closed: لا نتخطى البوابة بصمت");

    assert!(
        output.status.success(),
        "cargo tree فشل:\n{}",
        String::from_utf8_lossy(&output.stderr)
    );

    let stdout = String::from_utf8_lossy(&output.stdout);
    let names: Vec<&str> = stdout
        .lines()
        .filter_map(|line| line.split_whitespace().next())
        .collect();

    assert!(
        names.len() > 1,
        "بوابة فارغة (vacuous): cargo tree لم يُرجع اعتمادات — رُفض النجاح.\n{stdout}"
    );

    for name in &names {
        assert!(
            !FORBIDDEN_NAMES.contains(name),
            "KERNEL PURITY VIOLATION (ADR-023/ADR-027): `{name}` على مسار \
             الربط التشغيلي لـ ec-constitutional. النواة يجب أن تبقى خالية من tokio/async."
        );
    }
}
