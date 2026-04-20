//! Deterministic witness tests for ETNA semver variants.
//!
//! Each `witness_<property>_case_<tag>` is a fixed concrete input chosen
//! so that on base HEAD the property holds (test passes), and under the
//! per-variant mutation the property fails.
//!
//! The tests delegate to `property_<name>` — no invariant logic lives
//! here.

use semver::etna::{
    property_less_rejects_prerelease, property_parse_rejects_digit_after_minor_wildcard,
    property_version_debug_omits_empty, PropertyResult,
};

fn assert_pass(r: PropertyResult) {
    match r {
        PropertyResult::Pass => {}
        PropertyResult::Discard => panic!("witness unexpectedly discarded"),
        PropertyResult::Fail(m) => panic!("property failed: {m}"),
    }
}

// --- less_prerelease_5742fc2_1 ------------------------------------

/// `>1.0.0-alpha, <1.0` must not match `1.0.0-beta`. The buggy
/// implementation computed Less as `!matches_exact && !matches_greater`;
/// when the Less comparator has patch=None (`<1.0`), both complements
/// return false, producing `true` for any prerelease of `M.m.0`.
#[test]
fn witness_less_rejects_prerelease_case_1_0_0() {
    assert_pass(property_less_rejects_prerelease(1, 0, 0));
}

/// Another triple, to force the mutated path with different numeric
/// inputs. Exercises the same invariant at higher major/minor/patch.
#[test]
fn witness_less_rejects_prerelease_case_2_5_3() {
    assert_pass(property_less_rejects_prerelease(2, 5, 3));
}

// --- wildcard_patch_digit_a5850bb_1 -------------------------------

/// `">1.*.0"` (explicit Greater op with a minor wildcard and a concrete
/// patch digit) must be rejected. The buggy parser keyed off
/// `op == Op::Wildcard`, which is false for explicit ops, so the
/// comparator parsed as `op=Greater, minor=None, patch=Some(0)`.
#[test]
fn witness_parse_rejects_digit_after_minor_wildcard_case_greater_1_0() {
    assert_pass(property_parse_rejects_digit_after_minor_wildcard(0, 1, 0));
}

/// Caret variant: `"^1.*.0"` must also be rejected. Exercises the same
/// parse path with a different explicit operator.
#[test]
fn witness_parse_rejects_digit_after_minor_wildcard_case_caret_1_0() {
    assert_pass(property_parse_rejects_digit_after_minor_wildcard(3, 1, 0));
}

// --- debug_omits_empty_ae1b06c_1 ----------------------------------

/// `format!("{:?}", Version::new(1, 0, 0))` must render exactly
/// `Version { major: 1, minor: 0, patch: 0 }` — no pre/build fields,
/// since both identifiers are empty. The auto-derived Debug printed
/// every field including the empty ones.
#[test]
fn witness_version_debug_omits_empty_case_1_0_0() {
    assert_pass(property_version_debug_omits_empty(1, 0, 0));
}

/// Non-trivial numeric components to guard against the Debug impl
/// getting re-derived in a way that happens to match for zero-only
/// inputs.
#[test]
fn witness_version_debug_omits_empty_case_42_7_99() {
    assert_pass(property_version_debug_omits_empty(42, 7, 99));
}
