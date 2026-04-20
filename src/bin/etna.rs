// ETNA workload runner for semver.
//
// Usage: cargo run --release --bin etna -- <tool> <property>
//   tool:     etna | proptest | quickcheck | crabcheck | hegel
//   property: LessRejectsPrerelease
//           | ParseRejectsDigitAfterMinorWildcard
//           | VersionDebugOmitsEmpty
//           | All
//
// Every invocation prints exactly one JSON line to stdout and exits 0
// (except argv parsing, which exits 2).

use crabcheck::quickcheck::Arbitrary as CcArbitrary;
use hegel::{generators as hgen, HealthCheck, Hegel, Settings as HegelSettings, TestCase};
use proptest::prelude::*;
use proptest::test_runner::{Config as ProptestConfig, TestCaseError, TestError};
use quickcheck_etna::{Arbitrary as QcArbitrary, Gen, QuickCheck, ResultStatus, TestResult};
use rand::Rng;
use semver::etna::{
    property_less_rejects_prerelease, property_parse_rejects_digit_after_minor_wildcard,
    property_version_debug_omits_empty, PropertyResult,
};
use std::fmt;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;
use std::time::{Duration, Instant};

#[derive(Default, Clone, Copy)]
struct Metrics {
    inputs: u64,
    elapsed_us: u128,
}

impl Metrics {
    fn combine(self, other: Metrics) -> Metrics {
        Metrics {
            inputs: self.inputs + other.inputs,
            elapsed_us: self.elapsed_us + other.elapsed_us,
        }
    }
}

type Outcome = (Result<(), String>, Metrics);

fn to_err(r: PropertyResult) -> Result<(), String> {
    match r {
        PropertyResult::Pass | PropertyResult::Discard => Ok(()),
        PropertyResult::Fail(m) => Err(m),
    }
}

const ALL_PROPERTIES: &[&str] = &[
    "LessRejectsPrerelease",
    "ParseRejectsDigitAfterMinorWildcard",
    "VersionDebugOmitsEmpty",
];

fn cases_budget() -> u64 {
    std::env::var("ETNA_CASES")
        .ok()
        .and_then(|s| s.parse().ok())
        .unwrap_or(1_000_000)
}

fn run_all<F: FnMut(&str) -> Outcome>(mut f: F) -> Outcome {
    let mut total = Metrics::default();
    for p in ALL_PROPERTIES {
        let (r, m) = f(p);
        total = total.combine(m);
        if let Err(e) = r {
            return (Err(e), total);
        }
    }
    (Ok(()), total)
}

// ---------- Canonical witness inputs (tool=etna) ----------

fn check_less_rejects_prerelease() -> Result<(), String> {
    to_err(property_less_rejects_prerelease(1, 0, 0))
}

fn check_parse_rejects_digit_after_minor_wildcard() -> Result<(), String> {
    to_err(property_parse_rejects_digit_after_minor_wildcard(0, 1, 0))
}

fn check_version_debug_omits_empty() -> Result<(), String> {
    to_err(property_version_debug_omits_empty(1, 0, 0))
}

fn run_etna_property(property: &str) -> Outcome {
    if property == "All" {
        return run_all(run_etna_property);
    }
    let t0 = Instant::now();
    let result = match property {
        "LessRejectsPrerelease" => check_less_rejects_prerelease(),
        "ParseRejectsDigitAfterMinorWildcard" => check_parse_rejects_digit_after_minor_wildcard(),
        "VersionDebugOmitsEmpty" => check_version_debug_omits_empty(),
        _ => {
            return (
                Err(format!("Unknown property for etna: {property}")),
                Metrics::default(),
            );
        }
    };
    (
        result,
        Metrics {
            inputs: 1,
            elapsed_us: t0.elapsed().as_micros(),
        },
    )
}

// ---------- Shared Arbitrary-biased generators (qc + cc) ----------
//
// Bounded triples of (major, minor, patch). Keeping the values small
// makes the generated SemVer strings short and avoids pathological
// overflow cases. The wildcard-op property also needs a `u8` that
// picks among {>, <, ~, ^}.

#[derive(Clone)]
struct Triple {
    major: u64,
    minor: u64,
    patch: u64,
}
#[derive(Clone)]
struct OpTriple {
    op: u8,
    major: u64,
    patch: u64,
}

impl fmt::Debug for Triple {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}.{}.{}", self.major, self.minor, self.patch)
    }
}
impl fmt::Display for Triple {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}.{}.{}", self.major, self.minor, self.patch)
    }
}
impl fmt::Debug for OpTriple {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let op_str = match self.op % 4 {
            0 => ">",
            1 => "<",
            2 => "~",
            _ => "^",
        };
        write!(f, "{}{}.*.{}", op_str, self.major, self.patch)
    }
}
impl fmt::Display for OpTriple {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Debug::fmt(self, f)
    }
}

const MAJOR_MAX: u64 = 16;
const MINOR_MAX: u64 = 16;
const PATCH_MAX: u64 = 16;

fn random_u64<R: Rng>(rng: &mut R, max: u64) -> u64 {
    rng.random_range(0..=max)
}

impl QcArbitrary for Triple {
    fn arbitrary(g: &mut Gen) -> Self {
        Triple {
            major: g.random_range(0..=MAJOR_MAX),
            minor: g.random_range(0..=MINOR_MAX),
            patch: g.random_range(0..=PATCH_MAX),
        }
    }
}

impl QcArbitrary for OpTriple {
    fn arbitrary(g: &mut Gen) -> Self {
        OpTriple {
            op: g.random_range(0u8..=255),
            major: g.random_range(0..=MAJOR_MAX),
            patch: g.random_range(0..=PATCH_MAX),
        }
    }
}

impl<R: Rng> CcArbitrary<R> for Triple {
    fn generate(rng: &mut R, _n: usize) -> Self {
        Triple {
            major: random_u64(rng, MAJOR_MAX),
            minor: random_u64(rng, MINOR_MAX),
            patch: random_u64(rng, PATCH_MAX),
        }
    }
}
impl<R: Rng> CcArbitrary<R> for OpTriple {
    fn generate(rng: &mut R, _n: usize) -> Self {
        OpTriple {
            op: rng.random_range(0u8..=255),
            major: random_u64(rng, MAJOR_MAX),
            patch: random_u64(rng, PATCH_MAX),
        }
    }
}

// ---------- proptest ----------

fn triple_strategy() -> BoxedStrategy<(u64, u64, u64)> {
    (0u64..=MAJOR_MAX, 0u64..=MINOR_MAX, 0u64..=PATCH_MAX).boxed()
}

fn op_triple_strategy() -> BoxedStrategy<(u8, u64, u64)> {
    (any::<u8>(), 0u64..=MAJOR_MAX, 0u64..=PATCH_MAX).boxed()
}

fn run_proptest_property(property: &str) -> Outcome {
    if property == "All" {
        return run_all(run_proptest_property);
    }
    let counter = Arc::new(AtomicU64::new(0));
    let t0 = Instant::now();
    let cfg = ProptestConfig {
        cases: cases_budget().min(u32::MAX as u64) as u32,
        max_shrink_iters: 32,
        failure_persistence: None,
        ..ProptestConfig::default()
    };
    let mut runner = proptest::test_runner::TestRunner::new(cfg);
    let c = counter.clone();
    let result: Result<(), String> = match property {
        "LessRejectsPrerelease" => runner
            .run(&triple_strategy(), move |(a, b, p)| {
                c.fetch_add(1, Ordering::Relaxed);
                let outcome = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
                    property_less_rejects_prerelease(a, b, p)
                }));
                match outcome {
                    Ok(PropertyResult::Pass) | Ok(PropertyResult::Discard) => Ok(()),
                    Ok(PropertyResult::Fail(_)) | Err(_) => {
                        Err(TestCaseError::fail(format!("({} {} {})", a, b, p)))
                    }
                }
            })
            .map_err(|e| match e {
                TestError::Fail(reason, _) => reason.to_string(),
                other => other.to_string(),
            }),
        "ParseRejectsDigitAfterMinorWildcard" => runner
            .run(&op_triple_strategy(), move |(op, a, p)| {
                c.fetch_add(1, Ordering::Relaxed);
                let outcome = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
                    property_parse_rejects_digit_after_minor_wildcard(op, a, p)
                }));
                match outcome {
                    Ok(PropertyResult::Pass) | Ok(PropertyResult::Discard) => Ok(()),
                    Ok(PropertyResult::Fail(_)) | Err(_) => {
                        Err(TestCaseError::fail(format!("({} {} {})", op, a, p)))
                    }
                }
            })
            .map_err(|e| match e {
                TestError::Fail(reason, _) => reason.to_string(),
                other => other.to_string(),
            }),
        "VersionDebugOmitsEmpty" => runner
            .run(&triple_strategy(), move |(a, b, p)| {
                c.fetch_add(1, Ordering::Relaxed);
                let outcome = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
                    property_version_debug_omits_empty(a, b, p)
                }));
                match outcome {
                    Ok(PropertyResult::Pass) | Ok(PropertyResult::Discard) => Ok(()),
                    Ok(PropertyResult::Fail(_)) | Err(_) => {
                        Err(TestCaseError::fail(format!("({} {} {})", a, b, p)))
                    }
                }
            })
            .map_err(|e| match e {
                TestError::Fail(reason, _) => reason.to_string(),
                other => other.to_string(),
            }),
        _ => {
            return (
                Err(format!("Unknown property for proptest: {property}")),
                Metrics::default(),
            );
        }
    };
    let elapsed_us = t0.elapsed().as_micros();
    let inputs = counter.load(Ordering::Relaxed);
    (result, Metrics { inputs, elapsed_us })
}

// ---------- quickcheck (forked crate with `etna` feature) ----------

static QC_COUNTER: AtomicU64 = AtomicU64::new(0);

fn qc_less_rejects_prerelease(t: Triple) -> TestResult {
    QC_COUNTER.fetch_add(1, Ordering::Relaxed);
    match property_less_rejects_prerelease(t.major, t.minor, t.patch) {
        PropertyResult::Pass => TestResult::passed(),
        PropertyResult::Discard => TestResult::discard(),
        PropertyResult::Fail(_) => TestResult::failed(),
    }
}

fn qc_parse_rejects_digit_after_minor_wildcard(t: OpTriple) -> TestResult {
    QC_COUNTER.fetch_add(1, Ordering::Relaxed);
    match property_parse_rejects_digit_after_minor_wildcard(t.op, t.major, t.patch) {
        PropertyResult::Pass => TestResult::passed(),
        PropertyResult::Discard => TestResult::discard(),
        PropertyResult::Fail(_) => TestResult::failed(),
    }
}

fn qc_version_debug_omits_empty(t: Triple) -> TestResult {
    QC_COUNTER.fetch_add(1, Ordering::Relaxed);
    match property_version_debug_omits_empty(t.major, t.minor, t.patch) {
        PropertyResult::Pass => TestResult::passed(),
        PropertyResult::Discard => TestResult::discard(),
        PropertyResult::Fail(_) => TestResult::failed(),
    }
}

fn run_quickcheck_property(property: &str) -> Outcome {
    if property == "All" {
        return run_all(run_quickcheck_property);
    }
    QC_COUNTER.store(0, Ordering::Relaxed);
    let t0 = Instant::now();
    let budget = cases_budget();
    let mut qc = QuickCheck::new()
        .tests(budget)
        .max_tests(budget.saturating_mul(2))
        .max_time(Duration::from_secs(86_400));
    let result = match property {
        "LessRejectsPrerelease" => {
            qc.quicktest(qc_less_rejects_prerelease as fn(Triple) -> TestResult)
        }
        "ParseRejectsDigitAfterMinorWildcard" => qc.quicktest(
            qc_parse_rejects_digit_after_minor_wildcard as fn(OpTriple) -> TestResult,
        ),
        "VersionDebugOmitsEmpty" => {
            qc.quicktest(qc_version_debug_omits_empty as fn(Triple) -> TestResult)
        }
        _ => {
            return (
                Err(format!("Unknown property for quickcheck: {property}")),
                Metrics::default(),
            );
        }
    };
    let elapsed_us = t0.elapsed().as_micros();
    let inputs = QC_COUNTER.load(Ordering::Relaxed);
    let status = match result.status {
        ResultStatus::Finished => Ok(()),
        ResultStatus::Failed { arguments } => Err(format!("({})", arguments.join(" "))),
        ResultStatus::Aborted { err } => Err(format!("quickcheck aborted: {err:?}")),
        ResultStatus::TimedOut => Err("quickcheck timed out".to_string()),
        ResultStatus::GaveUp => Err(format!(
            "quickcheck gave up after {} tests",
            result.n_tests_passed
        )),
    };
    (status, Metrics { inputs, elapsed_us })
}

// ---------- crabcheck ----------

static CC_COUNTER: AtomicU64 = AtomicU64::new(0);

fn cc_less_rejects_prerelease(t: Triple) -> Option<bool> {
    CC_COUNTER.fetch_add(1, Ordering::Relaxed);
    match property_less_rejects_prerelease(t.major, t.minor, t.patch) {
        PropertyResult::Pass => Some(true),
        PropertyResult::Fail(_) => Some(false),
        PropertyResult::Discard => None,
    }
}

fn cc_parse_rejects_digit_after_minor_wildcard(t: OpTriple) -> Option<bool> {
    CC_COUNTER.fetch_add(1, Ordering::Relaxed);
    match property_parse_rejects_digit_after_minor_wildcard(t.op, t.major, t.patch) {
        PropertyResult::Pass => Some(true),
        PropertyResult::Fail(_) => Some(false),
        PropertyResult::Discard => None,
    }
}

fn cc_version_debug_omits_empty(t: Triple) -> Option<bool> {
    CC_COUNTER.fetch_add(1, Ordering::Relaxed);
    match property_version_debug_omits_empty(t.major, t.minor, t.patch) {
        PropertyResult::Pass => Some(true),
        PropertyResult::Fail(_) => Some(false),
        PropertyResult::Discard => None,
    }
}

fn run_crabcheck_property(property: &str) -> Outcome {
    if property == "All" {
        return run_all(run_crabcheck_property);
    }
    CC_COUNTER.store(0, Ordering::Relaxed);
    let t0 = Instant::now();
    let cc_config = crabcheck::quickcheck::Config {
        tests: cases_budget(),
    };
    let result = match property {
        "LessRejectsPrerelease" => {
            crabcheck::quickcheck::quickcheck_with_config(cc_config, cc_less_rejects_prerelease)
        }
        "ParseRejectsDigitAfterMinorWildcard" => crabcheck::quickcheck::quickcheck_with_config(
            cc_config,
            cc_parse_rejects_digit_after_minor_wildcard,
        ),
        "VersionDebugOmitsEmpty" => {
            crabcheck::quickcheck::quickcheck_with_config(cc_config, cc_version_debug_omits_empty)
        }
        _ => {
            return (
                Err(format!("Unknown property for crabcheck: {property}")),
                Metrics::default(),
            );
        }
    };
    let elapsed_us = t0.elapsed().as_micros();
    let inputs = CC_COUNTER.load(Ordering::Relaxed);
    let status = match result.status {
        crabcheck::quickcheck::ResultStatus::Finished => Ok(()),
        crabcheck::quickcheck::ResultStatus::Failed { arguments } => {
            Err(format!("({})", arguments.join(" ")))
        }
        crabcheck::quickcheck::ResultStatus::TimedOut => Err("crabcheck timed out".to_string()),
        crabcheck::quickcheck::ResultStatus::GaveUp => Err(format!(
            "crabcheck gave up: passed={}, discarded={}",
            result.passed, result.discarded
        )),
        crabcheck::quickcheck::ResultStatus::Aborted { error } => {
            Err(format!("crabcheck aborted: {error}"))
        }
    };
    (status, Metrics { inputs, elapsed_us })
}

// ---------- hegel ----------

static HG_COUNTER: AtomicU64 = AtomicU64::new(0);

fn hegel_settings() -> HegelSettings {
    HegelSettings::new()
        .test_cases(cases_budget())
        .suppress_health_check(HealthCheck::all())
}

fn hg_draw_u64(tc: &TestCase, max: u64) -> u64 {
    tc.draw(hgen::integers::<u64>().min_value(0).max_value(max))
}

fn hg_draw_u8(tc: &TestCase) -> u8 {
    tc.draw(hgen::integers::<u8>().min_value(0).max_value(u8::MAX)) as u8
}

fn run_hegel_property(property: &str) -> Outcome {
    if property == "All" {
        return run_all(run_hegel_property);
    }
    HG_COUNTER.store(0, Ordering::Relaxed);
    let t0 = Instant::now();
    let settings = hegel_settings();
    let run_result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| match property {
        "LessRejectsPrerelease" => {
            Hegel::new(|tc: TestCase| {
                HG_COUNTER.fetch_add(1, Ordering::Relaxed);
                let a = hg_draw_u64(&tc, MAJOR_MAX);
                let b = hg_draw_u64(&tc, MINOR_MAX);
                let p = hg_draw_u64(&tc, PATCH_MAX);
                let outcome = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
                    property_less_rejects_prerelease(a, b, p)
                }));
                match outcome {
                    Ok(PropertyResult::Pass) | Ok(PropertyResult::Discard) => {}
                    Ok(PropertyResult::Fail(_)) | Err(_) => panic!("({} {} {})", a, b, p),
                }
            })
            .settings(settings.clone())
            .run();
        }
        "ParseRejectsDigitAfterMinorWildcard" => {
            Hegel::new(|tc: TestCase| {
                HG_COUNTER.fetch_add(1, Ordering::Relaxed);
                let op = hg_draw_u8(&tc);
                let a = hg_draw_u64(&tc, MAJOR_MAX);
                let p = hg_draw_u64(&tc, PATCH_MAX);
                let outcome = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
                    property_parse_rejects_digit_after_minor_wildcard(op, a, p)
                }));
                match outcome {
                    Ok(PropertyResult::Pass) | Ok(PropertyResult::Discard) => {}
                    Ok(PropertyResult::Fail(_)) | Err(_) => panic!("({} {} {})", op, a, p),
                }
            })
            .settings(settings.clone())
            .run();
        }
        "VersionDebugOmitsEmpty" => {
            Hegel::new(|tc: TestCase| {
                HG_COUNTER.fetch_add(1, Ordering::Relaxed);
                let a = hg_draw_u64(&tc, MAJOR_MAX);
                let b = hg_draw_u64(&tc, MINOR_MAX);
                let p = hg_draw_u64(&tc, PATCH_MAX);
                let outcome = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
                    property_version_debug_omits_empty(a, b, p)
                }));
                match outcome {
                    Ok(PropertyResult::Pass) | Ok(PropertyResult::Discard) => {}
                    Ok(PropertyResult::Fail(_)) | Err(_) => panic!("({} {} {})", a, b, p),
                }
            })
            .settings(settings.clone())
            .run();
        }
        _ => panic!("__unknown_property:{}", property),
    }));
    let elapsed_us = t0.elapsed().as_micros();
    let inputs = HG_COUNTER.load(Ordering::Relaxed);
    let metrics = Metrics { inputs, elapsed_us };
    let status = match run_result {
        Ok(()) => Ok(()),
        Err(e) => {
            let msg = if let Some(s) = e.downcast_ref::<String>() {
                s.clone()
            } else if let Some(s) = e.downcast_ref::<&str>() {
                s.to_string()
            } else {
                "hegel panicked with non-string payload".to_string()
            };
            if let Some(rest) = msg.strip_prefix("__unknown_property:") {
                return (
                    Err(format!("Unknown property for hegel: {rest}")),
                    Metrics::default(),
                );
            }
            Err(msg
                .strip_prefix("Property test failed: ")
                .unwrap_or(&msg)
                .to_string())
        }
    };
    (status, metrics)
}

// ---------- dispatch ----------

fn run(tool: &str, property: &str) -> Outcome {
    match tool {
        "etna" => run_etna_property(property),
        "proptest" => run_proptest_property(property),
        "quickcheck" => run_quickcheck_property(property),
        "crabcheck" => run_crabcheck_property(property),
        "hegel" => run_hegel_property(property),
        _ => (Err(format!("Unknown tool: {tool}")), Metrics::default()),
    }
}

fn json_str(s: &str) -> String {
    let mut out = String::with_capacity(s.len() + 2);
    out.push('"');
    for c in s.chars() {
        match c {
            '"' => out.push_str("\\\""),
            '\\' => out.push_str("\\\\"),
            '\n' => out.push_str("\\n"),
            '\r' => out.push_str("\\r"),
            '\t' => out.push_str("\\t"),
            c if (c as u32) < 0x20 => out.push_str(&format!("\\u{:04x}", c as u32)),
            c => out.push(c),
        }
    }
    out.push('"');
    out
}

fn emit_json(
    tool: &str,
    property: &str,
    status: &str,
    metrics: Metrics,
    counterexample: Option<&str>,
    error: Option<&str>,
) {
    let cex = counterexample.map_or("null".to_string(), json_str);
    let err = error.map_or("null".to_string(), json_str);
    println!(
        "{{\"status\":{},\"tests\":{},\"discards\":0,\"time\":{},\"counterexample\":{},\"error\":{},\"tool\":{},\"property\":{}}}",
        json_str(status),
        metrics.inputs,
        json_str(&format!("{}us", metrics.elapsed_us)),
        cex,
        err,
        json_str(tool),
        json_str(property),
    );
}

fn main() {
    let args: Vec<String> = std::env::args().collect();
    if args.len() < 3 {
        eprintln!("Usage: {} <tool> <property>", args[0]);
        eprintln!("Tools: etna | proptest | quickcheck | crabcheck | hegel");
        eprintln!(
            "Properties: LessRejectsPrerelease | ParseRejectsDigitAfterMinorWildcard | VersionDebugOmitsEmpty | All"
        );
        std::process::exit(2);
    }
    let (tool, property) = (args[1].as_str(), args[2].as_str());

    let previous_hook = std::panic::take_hook();
    std::panic::set_hook(Box::new(|_| {}));
    let caught = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| run(tool, property)));
    std::panic::set_hook(previous_hook);

    let (result, metrics) = match caught {
        Ok(outcome) => outcome,
        Err(payload) => {
            let msg = if let Some(s) = payload.downcast_ref::<String>() {
                s.clone()
            } else if let Some(s) = payload.downcast_ref::<&str>() {
                s.to_string()
            } else {
                "panic with non-string payload".to_string()
            };
            emit_json(tool, property, "aborted", Metrics::default(), None, Some(&msg));
            return;
        }
    };

    match result {
        Ok(()) => emit_json(tool, property, "passed", metrics, None, None),
        Err(e) => emit_json(tool, property, "failed", metrics, Some(&e), None),
    }
}
