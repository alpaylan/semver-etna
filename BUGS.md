# semver — Injected Bugs

Total mutations: 3

## Bug Index

| # | Variant | Name | Location | Injection | Fix Commit |
|---|---------|------|----------|-----------|------------|
| 1 | `debug_omits_empty_ae1b06c_1` | `debug_omits_empty` | `src/display.rs` | `patch` | `ae1b06c8c005345ec9d343ddb0f87f45e61ea4a8` |
| 2 | `less_prerelease_5742fc2_1` | `less_prerelease` | `src/eval.rs` | `patch` | `5742fc2f584dc14b46199d797de65305fe9b5144` |
| 3 | `wildcard_patch_digit_a5850bb_1` | `wildcard_patch_digit` | `src/parse.rs` | `patch` | `a5850bbd0d1bf6e5ae1ed1310cbe8919fa77d618` |

## Property Mapping

| Variant | Property | Witness(es) |
|---------|----------|-------------|
| `debug_omits_empty_ae1b06c_1` | `VersionDebugOmitsEmpty` | `witness_version_debug_omits_empty_case_1_0_0`, `witness_version_debug_omits_empty_case_42_7_99` |
| `less_prerelease_5742fc2_1` | `LessRejectsPrerelease` | `witness_less_rejects_prerelease_case_1_0_0`, `witness_less_rejects_prerelease_case_2_5_3` |
| `wildcard_patch_digit_a5850bb_1` | `ParseRejectsDigitAfterMinorWildcard` | `witness_parse_rejects_digit_after_minor_wildcard_case_greater_1_0`, `witness_parse_rejects_digit_after_minor_wildcard_case_caret_1_0` |

## Framework Coverage

| Property | proptest | quickcheck | crabcheck | hegel |
|----------|---------:|-----------:|----------:|------:|
| `VersionDebugOmitsEmpty` | ✓ | ✓ | ✓ | ✓ |
| `LessRejectsPrerelease` | ✓ | ✓ | ✓ | ✓ |
| `ParseRejectsDigitAfterMinorWildcard` | ✓ | ✓ | ✓ | ✓ |

## Bug Details

### 1. debug_omits_empty

- **Variant**: `debug_omits_empty_ae1b06c_1`
- **Location**: `src/display.rs`
- **Property**: `VersionDebugOmitsEmpty`
- **Witness(es)**:
  - `witness_version_debug_omits_empty_case_1_0_0`
  - `witness_version_debug_omits_empty_case_42_7_99`
- **Source**: Customize Debug impl of Version to omit empty pieces
  > The derived `Debug for Version` always emitted `pre` and `build` fields, so printing a bare `Version::new(1, 0, 0)` produced the noisy `Prerelease("")` / `BuildMetadata("")` lines; the custom impl drops those fields when they are empty.
- **Fix commit**: `ae1b06c8c005345ec9d343ddb0f87f45e61ea4a8` — Customize Debug impl of Version to omit empty pieces
- **Invariant violated**: The `Debug` rendering of a `Version` whose `pre` and `build` identifiers are both empty must be exactly `Version { major: M, minor: m, patch: p }`. Empty pre-release and build-metadata fields must be omitted.
- **How the mutation triggers**: The buggy `impl Debug for Version` unconditionally chains `.field("pre", &self.pre).field("build", &self.build)`, so `format!("{:?}", Version::new(1, 0, 0))` renders as `Version { major: 1, minor: 0, patch: 0, pre: Prerelease(""), build: BuildMetadata("") }` — including the empty fields the customized impl is supposed to suppress.

### 2. less_prerelease

- **Variant**: `less_prerelease_5742fc2_1`
- **Location**: `src/eval.rs`
- **Property**: `LessRejectsPrerelease`
- **Witness(es)**:
  - `witness_less_rejects_prerelease_case_1_0_0`
  - `witness_less_rejects_prerelease_case_2_5_3`
- **Source**: Fix <I.J to not match I.J.0 prereleases
  > `Op::Less` was evaluated as `!matches_exact && !matches_greater`, which for a patch-less comparator (`<M.m`) against a prerelease collapsed to `true` and accepted `M.m.p-beta`; the fix distinguishes patch-less Less from a full Less comparator so that the whole `M.m` line is excluded.
- **Fix commit**: `5742fc2f584dc14b46199d797de65305fe9b5144` — Fix <I.J to not match I.J.0 prereleases
- **Invariant violated**: A `VersionReq` of the form `>M.m.p-alpha, <M.m` (Less comparator without a patch component) must NOT match `M.m.p-beta`. The Less comparator with `patch = None` is supposed to exclude the entire `M.m.*` line, even when combined with a prerelease-admitting Greater.
- **How the mutation triggers**: The buggy `Op::Less` branch is `!matches_exact && !matches_greater`. For a `<M.m` comparator with `cmp.patch = None`, `matches_greater` returns `false` (early return on the `None` patch) and `matches_exact` returns `false` (version has non-empty pre, cmp does not), so the naive complement yields `true` — the version matches, violating the invariant.

### 3. wildcard_patch_digit

- **Variant**: `wildcard_patch_digit_a5850bb_1`
- **Location**: `src/parse.rs`
- **Property**: `ParseRejectsDigitAfterMinorWildcard`
- **Witness(es)**:
  - `witness_parse_rejects_digit_after_minor_wildcard_case_greater_1_0`
  - `witness_parse_rejects_digit_after_minor_wildcard_case_caret_1_0`
- **Source**: Disallow patch version digit after minor version wildcard
  > The parser's `UnexpectedAfterWildcard` guard gated on `op == Op::Wildcard`, so when an explicit operator preceded the wildcard (e.g. `>1.*.0`) the guard never fired and the parser silently accepted a concrete patch digit after a minor wildcard.
- **Fix commit**: `a5850bbd0d1bf6e5ae1ed1310cbe8919fa77d618` — Disallow patch version digit after minor version wildcard
- **Invariant violated**: A comparator of the form `<op>M.*.P` — where `<op>` is an explicit operator (`>`, `<`, `~`, `^`) and `P` is a concrete digit — must be rejected by the parser. The minor wildcard cannot be followed by a concrete patch component.
- **How the mutation triggers**: The buggy patch-position check is `op == Op::Wildcard`. When the caller supplied an explicit `<op>`, `op` is `Greater`/`Less`/`Tilde`/`Caret` — never `Wildcard` — so the `UnexpectedAfterWildcard` guard never fires and `">1.*.0"` parses successfully as `op=Greater, minor=None, patch=Some(0)`.
