# semver — Injected Bugs

Total mutations: 3 (from 709 commits scanned; 706 non-fix or terminally inexpressible — see below)

## Bug Index

| # | Name | Variant | File | Injection | Fix Commit |
|---|------|---------|------|-----------|------------|
| 1 | `less_prerelease` | `less_prerelease_5742fc2_1` | `patches/less_prerelease_5742fc2_1.patch` | `patch` | `5742fc2f584dc14b46199d797de65305fe9b5144` |
| 2 | `wildcard_patch_digit` | `wildcard_patch_digit_a5850bb_1` | `patches/wildcard_patch_digit_a5850bb_1.patch` | `patch` | `a5850bbd0d1bf6e5ae1ed1310cbe8919fa77d618` |
| 3 | `debug_omits_empty` | `debug_omits_empty_ae1b06c_1` | `patches/debug_omits_empty_ae1b06c_1.patch` | `patch` | `ae1b06c8c005345ec9d343ddb0f87f45e61ea4a8` |

## Property Mapping

| Variant | Property | Witness(es) |
|---------|----------|-------------|
| `less_prerelease_5742fc2_1` | `property_less_rejects_prerelease` | `witness_less_rejects_prerelease_case_1_0_0`, `witness_less_rejects_prerelease_case_2_5_3` |
| `wildcard_patch_digit_a5850bb_1` | `property_parse_rejects_digit_after_minor_wildcard` | `witness_parse_rejects_digit_after_minor_wildcard_case_greater_1_0`, `witness_parse_rejects_digit_after_minor_wildcard_case_caret_1_0` |
| `debug_omits_empty_ae1b06c_1` | `property_version_debug_omits_empty` | `witness_version_debug_omits_empty_case_1_0_0`, `witness_version_debug_omits_empty_case_42_7_99` |

## Framework Coverage

| Property | proptest | quickcheck | crabcheck | hegel |
|----------|---------:|-----------:|----------:|------:|
| `property_less_rejects_prerelease` | OK | OK | OK | OK |
| `property_parse_rejects_digit_after_minor_wildcard` | OK | OK | OK | OK |
| `property_version_debug_omits_empty` | OK | OK | OK | OK |

## Bug Details

### 1. less_prerelease (5742fc2_1)
- **Variant**: `less_prerelease_5742fc2_1`
- **Location**: `src/eval.rs`, `matches_impl` dispatch for `Op::Less`/`Op::LessEq`
- **Property**: `property_less_rejects_prerelease`
- **Witnesses**: `witness_less_rejects_prerelease_case_1_0_0`, `witness_less_rejects_prerelease_case_2_5_3`
- **Fix commit**: `5742fc2f584dc14b46199d797de65305fe9b5144` — "Fix <I.J to not match I.J.0 prereleases"
- **Invariant violated**: A `VersionReq` of the form `>M.m.p-alpha, <M.m` (Less comparator without a patch component) must NOT match `M.m.p-beta`. The Less comparator with `patch = None` is supposed to exclude the entire `M.m.*` line, even when combined with a prerelease-admitting Greater.
- **How the mutation triggers**: The buggy `Op::Less` branch is `!matches_exact && !matches_greater`. For a `<M.m` comparator with `cmp.patch = None`, `matches_greater` returns `false` (early return on the `None` patch) and `matches_exact` returns `false` (version has non-empty pre, cmp does not), so the naive complement yields `true` — the version matches, violating the invariant.

### 2. wildcard_patch_digit (a5850bb_1)
- **Variant**: `wildcard_patch_digit_a5850bb_1`
- **Location**: `src/parse.rs`, `comparator` minor-wildcard handling
- **Property**: `property_parse_rejects_digit_after_minor_wildcard`
- **Witnesses**: `witness_parse_rejects_digit_after_minor_wildcard_case_greater_1_0`, `witness_parse_rejects_digit_after_minor_wildcard_case_caret_1_0`
- **Fix commit**: `a5850bbd0d1bf6e5ae1ed1310cbe8919fa77d618` — "Disallow patch version digit after minor version wildcard"
- **Invariant violated**: A comparator of the form `<op>M.*.P` — where `<op>` is an explicit operator (`>`, `<`, `~`, `^`) and `P` is a concrete digit — must be rejected by the parser. The minor wildcard cannot be followed by a concrete patch component.
- **How the mutation triggers**: The buggy patch-position check is `op == Op::Wildcard`. When the caller supplied an explicit `<op>`, `op` is `Greater`/`Less`/`Tilde`/`Caret` — never `Wildcard` — so the `UnexpectedAfterWildcard` guard never fires and `">1.*.0"` parses successfully as `op=Greater, minor=None, patch=Some(0)`.

### 3. debug_omits_empty (ae1b06c_1)
- **Variant**: `debug_omits_empty_ae1b06c_1`
- **Location**: `src/display.rs`, `impl Debug for Version`
- **Property**: `property_version_debug_omits_empty`
- **Witnesses**: `witness_version_debug_omits_empty_case_1_0_0`, `witness_version_debug_omits_empty_case_42_7_99`
- **Fix commit**: `ae1b06c8c005345ec9d343ddb0f87f45e61ea4a8` — "Customize Debug impl of Version to omit empty pieces"
- **Invariant violated**: The `Debug` rendering of a `Version` whose `pre` and `build` identifiers are both empty must be exactly `Version { major: M, minor: m, patch: p }`. Empty pre-release and build-metadata fields must be omitted.
- **How the mutation triggers**: The buggy `impl Debug for Version` unconditionally chains `.field("pre", &self.pre).field("build", &self.build)`, so `format!("{:?}", Version::new(1, 0, 0))` renders as `Version { major: 1, minor: 0, patch: 0, pre: Prerelease(""), build: BuildMetadata("") }` — including the empty fields the customized impl is supposed to suppress.

## Skipped candidates

### Pre-1.0.0 rewrite — surface removed
The entire crate was rewritten in commit `3a1b1eb` ("Rewrite the crate", 2021-05-24). All fix commits before that date target a completely different codebase (old `Version` shape with `Vec<Identifier>` pre/build, separate `ReqParseError` module, different comparator evaluation model). Those mutations are terminally inexpressible against the post-rewrite surface.

### Non-fix or unsuitable fix commits
- `00c04169` ("Parse x/X as wildcard"): feature addition recognizing `x`/`X` as wildcard tokens alongside `*`. Not a bug fix.
- `02a9ca80` ("isize overflow in pad"): overflow guard in a formatting padding helper. No observable PBT invariant distinguishable from framework-level arithmetic behavior.
- `9d801cf9` ("Error formatter flags"): cosmetic width/alignment flag pass-through in error formatting. No invariant separable from `fmt::Formatter` semantics.
