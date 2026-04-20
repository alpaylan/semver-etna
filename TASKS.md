# semver — ETNA Tasks

Total tasks: 12

ETNA tasks are **mutation/property/witness triplets**. Each row below is one runnable task. The `<PropertyKey>` token in the command column uses the PascalCase key recognised by `src/bin/etna.rs`; passing `All` runs every property for the named framework in a single invocation.

## Property keys

| Property | PropertyKey |
|----------|-------------|
| `property_less_rejects_prerelease` | `LessRejectsPrerelease` |
| `property_parse_rejects_digit_after_minor_wildcard` | `ParseRejectsDigitAfterMinorWildcard` |
| `property_version_debug_omits_empty` | `VersionDebugOmitsEmpty` |

## Task Index

| Task | Variant | Framework | Property | Witness | Command |
|------|---------|-----------|----------|---------|---------|
| 001 | `less_prerelease_5742fc2_1` | proptest | `property_less_rejects_prerelease` | `witness_less_rejects_prerelease_case_1_0_0` | `cargo run --release --bin etna -- proptest LessRejectsPrerelease` |
| 002 | `less_prerelease_5742fc2_1` | quickcheck | `property_less_rejects_prerelease` | `witness_less_rejects_prerelease_case_1_0_0` | `cargo run --release --bin etna -- quickcheck LessRejectsPrerelease` |
| 003 | `less_prerelease_5742fc2_1` | crabcheck | `property_less_rejects_prerelease` | `witness_less_rejects_prerelease_case_1_0_0` | `cargo run --release --bin etna -- crabcheck LessRejectsPrerelease` |
| 004 | `less_prerelease_5742fc2_1` | hegel | `property_less_rejects_prerelease` | `witness_less_rejects_prerelease_case_1_0_0` | `cargo run --release --bin etna -- hegel LessRejectsPrerelease` |
| 005 | `wildcard_patch_digit_a5850bb_1` | proptest | `property_parse_rejects_digit_after_minor_wildcard` | `witness_parse_rejects_digit_after_minor_wildcard_case_greater_1_0` | `cargo run --release --bin etna -- proptest ParseRejectsDigitAfterMinorWildcard` |
| 006 | `wildcard_patch_digit_a5850bb_1` | quickcheck | `property_parse_rejects_digit_after_minor_wildcard` | `witness_parse_rejects_digit_after_minor_wildcard_case_greater_1_0` | `cargo run --release --bin etna -- quickcheck ParseRejectsDigitAfterMinorWildcard` |
| 007 | `wildcard_patch_digit_a5850bb_1` | crabcheck | `property_parse_rejects_digit_after_minor_wildcard` | `witness_parse_rejects_digit_after_minor_wildcard_case_greater_1_0` | `cargo run --release --bin etna -- crabcheck ParseRejectsDigitAfterMinorWildcard` |
| 008 | `wildcard_patch_digit_a5850bb_1` | hegel | `property_parse_rejects_digit_after_minor_wildcard` | `witness_parse_rejects_digit_after_minor_wildcard_case_greater_1_0` | `cargo run --release --bin etna -- hegel ParseRejectsDigitAfterMinorWildcard` |
| 009 | `debug_omits_empty_ae1b06c_1` | proptest | `property_version_debug_omits_empty` | `witness_version_debug_omits_empty_case_1_0_0` | `cargo run --release --bin etna -- proptest VersionDebugOmitsEmpty` |
| 010 | `debug_omits_empty_ae1b06c_1` | quickcheck | `property_version_debug_omits_empty` | `witness_version_debug_omits_empty_case_1_0_0` | `cargo run --release --bin etna -- quickcheck VersionDebugOmitsEmpty` |
| 011 | `debug_omits_empty_ae1b06c_1` | crabcheck | `property_version_debug_omits_empty` | `witness_version_debug_omits_empty_case_1_0_0` | `cargo run --release --bin etna -- crabcheck VersionDebugOmitsEmpty` |
| 012 | `debug_omits_empty_ae1b06c_1` | hegel | `property_version_debug_omits_empty` | `witness_version_debug_omits_empty_case_1_0_0` | `cargo run --release --bin etna -- hegel VersionDebugOmitsEmpty` |

## Witness catalog

Each witness is a deterministic concrete test. Base build: passes. Variant-active build: fails. Witnesses live in `tests/etna_witnesses.rs`.

| Witness | Property | Detects | Input shape |
|---------|----------|---------|-------------|
| `witness_less_rejects_prerelease_case_1_0_0` | `property_less_rejects_prerelease` | `less_prerelease_5742fc2_1` | `(1, 0, 0)` — builds `">1.0.0-alpha, <1.0"`, checks it does not match `1.0.0-beta`. The `<1.0` Less comparator has `patch=None`, which triggers the `!matches_exact && !matches_greater` false-positive on the prerelease of `1.0.0` |
| `witness_less_rejects_prerelease_case_2_5_3` | `property_less_rejects_prerelease` | `less_prerelease_5742fc2_1` | `(2, 5, 3)` — same invariant with non-zero major/minor/patch, guards against the buggy path happening to skip `0,0,0` edge cases |
| `witness_parse_rejects_digit_after_minor_wildcard_case_greater_1_0` | `property_parse_rejects_digit_after_minor_wildcard` | `wildcard_patch_digit_a5850bb_1` | `(op=0, major=1, patch=0)` — renders `">1.*.0"`. Greater op with a minor wildcard plus a concrete patch digit — the `op == Op::Wildcard` check misses this because `op=Greater`, so the comparator parses as garbage |
| `witness_parse_rejects_digit_after_minor_wildcard_case_caret_1_0` | `property_parse_rejects_digit_after_minor_wildcard` | `wildcard_patch_digit_a5850bb_1` | `(op=3, major=1, patch=0)` — renders `"^1.*.0"`. Caret variant exercises the same invariant with a different explicit operator |
| `witness_version_debug_omits_empty_case_1_0_0` | `property_version_debug_omits_empty` | `debug_omits_empty_ae1b06c_1` | `Version::new(1, 0, 0)` — expected `Debug` is `Version { major: 1, minor: 0, patch: 0 }`; the buggy impl appends `, pre: Prerelease(""), build: BuildMetadata("")` |
| `witness_version_debug_omits_empty_case_42_7_99` | `property_version_debug_omits_empty` | `debug_omits_empty_ae1b06c_1` | `Version::new(42, 7, 99)` — non-trivial numeric components guard against a re-derive of `Debug` that happens to match on `0,0,0` by coincidence |
