//! ETNA framework-neutral property functions for semver.
//!
//! Each `property_<name>` is a pure function taking concrete, owned inputs
//! and returning `PropertyResult`. Framework adapters in `src/bin/etna.rs`
//! and witness tests in `tests/etna_witnesses.rs` all call these functions
//! directly — invariants are never re-implemented inside an adapter.

#![allow(missing_docs)]

use alloc::format;
use alloc::string::String;

use crate::{Version, VersionReq};

pub enum PropertyResult {
    Pass,
    Fail(String),
    Discard,
}

// ------------------------------------------------------------------
// property_less_rejects_prerelease
//
// Invariant (SemVer): a req of the form `>M.m.p-alpha, <M.m` (no patch on
// the Less comparator) must NOT match `M.m.p-beta`. The prerelease on the
// Greater side allows `M.m.p-*` generally; the Less side without a patch
// component is supposed to exclude the entire `M.m.*` line.
//
// Historical bug (5742fc2f): `Op::Less` was implemented as
// `!matches_exact && !matches_greater`. When `cmp.patch = None` and ver has
// the same major.minor as cmp, both matches_exact and matches_greater
// return false, so the naive complement returned true — incorrectly
// matching prereleases of `M.m.0`.

pub fn property_less_rejects_prerelease(major: u64, minor: u64, patch: u64) -> PropertyResult {
    let req_src = format!(">{major}.{minor}.{patch}-alpha, <{major}.{minor}");
    let ver_src = format!("{major}.{minor}.{patch}-beta");
    let req = match VersionReq::parse(&req_src) {
        Ok(r) => r,
        Err(_) => return PropertyResult::Discard,
    };
    let ver = match Version::parse(&ver_src) {
        Ok(v) => v,
        Err(_) => return PropertyResult::Discard,
    };
    if req.matches(&ver) {
        PropertyResult::Fail(format!(
            "req {req_src:?} unexpectedly matched {ver_src:?}"
        ))
    } else {
        PropertyResult::Pass
    }
}

// ------------------------------------------------------------------
// property_parse_rejects_digit_after_minor_wildcard
//
// Invariant: a version requirement of the form `>M.*.P` (with a digit in the
// patch position after a minor-wildcard) is invalid and must be rejected by
// the parser. The comparator grammar disallows a concrete patch digit when
// the minor field is a wildcard.
//
// Historical bug (a5850bbd): the comparator parser checked `op == Op::Wildcard`
// to decide whether to reject a patch digit after a minor-wildcard. That
// fails when the caller supplied an explicit operator (>, <, ~, ^), since
// the op is not Wildcard in that case. So `">1.*.0"` parsed successfully
// with `op=Greater, minor=None, patch=Some(0)` — a garbage comparator.

pub fn property_parse_rejects_digit_after_minor_wildcard(
    op: u8,
    major: u64,
    patch: u64,
) -> PropertyResult {
    let op_str: &'static str = match op % 4 {
        0 => ">",
        1 => "<",
        2 => "~",
        _ => "^",
    };
    let req_src = format!("{op_str}{major}.*.{patch}");
    match VersionReq::parse(&req_src) {
        Ok(r) => PropertyResult::Fail(format!(
            "req {req_src:?} was accepted; comparators={:?}",
            r.comparators
        )),
        Err(_) => PropertyResult::Pass,
    }
}

// ------------------------------------------------------------------
// property_version_debug_omits_empty
//
// Invariant: the `Debug` representation of a `Version` whose pre-release
// and build-metadata are both empty must NOT include those fields. The
// expected shape is exactly `Version { major: M, minor: m, patch: p }`.
//
// Historical bug (ae1b06c8): the auto-derived `Debug` printed every field,
// so a plain `Version::new(1, 0, 0)` debug-formatted to
// `Version { major: 1, minor: 0, patch: 0, pre: Prerelease(""), build: BuildMetadata("") }`.
// The fix customized `Debug` to skip empty pre/build.

pub fn property_version_debug_omits_empty(major: u64, minor: u64, patch: u64) -> PropertyResult {
    let v = Version::new(major, minor, patch);
    let rendered = format!("{:?}", v);
    let expected = format!("Version {{ major: {major}, minor: {minor}, patch: {patch} }}");
    if rendered == expected {
        PropertyResult::Pass
    } else {
        PropertyResult::Fail(format!(
            "debug output mismatch: got {rendered:?}, expected {expected:?}"
        ))
    }
}
