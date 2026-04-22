# semver — ETNA Tasks

Total tasks: 12

## Task Index

| Task | Variant | Framework | Property | Witness |
|------|---------|-----------|----------|---------|
| 001 | `debug_omits_empty_ae1b06c_1` | proptest | `VersionDebugOmitsEmpty` | `witness_version_debug_omits_empty_case_1_0_0` |
| 002 | `debug_omits_empty_ae1b06c_1` | quickcheck | `VersionDebugOmitsEmpty` | `witness_version_debug_omits_empty_case_1_0_0` |
| 003 | `debug_omits_empty_ae1b06c_1` | crabcheck | `VersionDebugOmitsEmpty` | `witness_version_debug_omits_empty_case_1_0_0` |
| 004 | `debug_omits_empty_ae1b06c_1` | hegel | `VersionDebugOmitsEmpty` | `witness_version_debug_omits_empty_case_1_0_0` |
| 005 | `less_prerelease_5742fc2_1` | proptest | `LessRejectsPrerelease` | `witness_less_rejects_prerelease_case_1_0_0` |
| 006 | `less_prerelease_5742fc2_1` | quickcheck | `LessRejectsPrerelease` | `witness_less_rejects_prerelease_case_1_0_0` |
| 007 | `less_prerelease_5742fc2_1` | crabcheck | `LessRejectsPrerelease` | `witness_less_rejects_prerelease_case_1_0_0` |
| 008 | `less_prerelease_5742fc2_1` | hegel | `LessRejectsPrerelease` | `witness_less_rejects_prerelease_case_1_0_0` |
| 009 | `wildcard_patch_digit_a5850bb_1` | proptest | `ParseRejectsDigitAfterMinorWildcard` | `witness_parse_rejects_digit_after_minor_wildcard_case_greater_1_0` |
| 010 | `wildcard_patch_digit_a5850bb_1` | quickcheck | `ParseRejectsDigitAfterMinorWildcard` | `witness_parse_rejects_digit_after_minor_wildcard_case_greater_1_0` |
| 011 | `wildcard_patch_digit_a5850bb_1` | crabcheck | `ParseRejectsDigitAfterMinorWildcard` | `witness_parse_rejects_digit_after_minor_wildcard_case_greater_1_0` |
| 012 | `wildcard_patch_digit_a5850bb_1` | hegel | `ParseRejectsDigitAfterMinorWildcard` | `witness_parse_rejects_digit_after_minor_wildcard_case_greater_1_0` |

## Witness Catalog

- `witness_version_debug_omits_empty_case_1_0_0` — base passes, variant fails
- `witness_version_debug_omits_empty_case_42_7_99` — base passes, variant fails
- `witness_less_rejects_prerelease_case_1_0_0` — base passes, variant fails
- `witness_less_rejects_prerelease_case_2_5_3` — base passes, variant fails
- `witness_parse_rejects_digit_after_minor_wildcard_case_greater_1_0` — base passes, variant fails
- `witness_parse_rejects_digit_after_minor_wildcard_case_caret_1_0` — base passes, variant fails
