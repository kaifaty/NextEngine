# Physical sound V34 F0 target-safe profile freeze result

| Field | Value |
| --- | --- |
| Date | `2026-09-02` |
| Decision | `PASS / REPEAT_EXACT / ZERO_TARGET_MODEL_SIGNAL_ACCESS` |
| Owner commit | `121f2270434ecbdcfadfb061101b55b91ed9bd64` |
| Owner SHA-256 | `e71a2b9b8b86b190e6c129644b29306fe4a8cbffff68bd24c231f961f086ca28` |
| Profile SHA-256 | `7be7e96a2899d0e4ef7aad668368669886d7f45adac34ec6676afbbe6b45e429` |
| Claim | `TARGET_SAFE_SYNTHETIC_SPECTRAL_RECOVERY_PROFILE_ONLY / NO_MODEL_QUALITY_REAL_MATERIAL_VALIDATOR_RELEASE_ADMISSION_COOKER_DEMO_OR_RUNTIME_AUTHORITY` |

## Outcome

The frozen V34 F0 owner ran in two fresh CPU processes against the same
committed owner/profile. Both return `F0_TARGET_SAFE_PROFILE_FREEZE_PASS`,
authorize only `V34-C0-signal-blind-structural-witness-census` and publish the
same three artifacts byte-for-byte. Stdout also matches exactly and both stderr
streams are empty.

The canonical overlay freezes fresh synthetic materials, ten geometry cells,
`12 / 6 / 6` disjoint train/development/method-holdout contacts, fresh truth
expressions and a new role-identity prefix. It is disjoint from both V32 and
V33 evidence. One declared `u=0` contact in each set algebraically supplies
`180` candidate nodal rows in every evaluation stratum; C0 must still prove
their actual P1 values before training.

Hash closure proves V33's features, normalization, `1,811`-parameter candidate,
`1,491`-parameter raw MLP, all controls, optimizer, steps, seeds, gates,
family/role/count algebra and resource bounds are unchanged. No unpublished
V33 metric selected a V34 model value.

## Exact access receipt

Every field is zero in both runs:

| Access | Count |
| --- | ---: |
| Train target rows | `0` |
| Development target rows | `0` |
| Method-holdout target rows | `0` |
| Feature rows materialized | `0` |
| Oracle values evaluated | `0` |
| Model parameters initialized | `0` |
| Real signal values decoded | `0` |
| Protected signal values decoded | `0` |
| Network requests | `0` |

F0 therefore spends no scientific role. Its pass authorizes the signal-blind
C0 owner only; T0/D0/H0 remain blocked.

## Repeatability and resources

| Artifact | Bytes | SHA-256 |
| --- | ---: | --- |
| `conformance.json` | `4,717` | `2f912b7ef077c5b539207aa254eb04421c2ad5903c306774ed891ffe86573625` |
| `evidence.json` | `1,152` | `942ea1c83aab9c8a4dc6abc3d2af0af6f98213bab32a13414d0e6a3079f8f633` |
| `report.json` | `900` | `2a812c6ecd34bcb79a420bdf5aa6dbd3a21605b73e1089c5e9843ae2af935b7b` |

The repeated stdout SHA-256 is
`f1fc2a32ab648167633c28e60341e1def99c3074e2d1c64b723f706ca511719d`.
Run A uses `0.06 s / 32,496 KiB`; run B uses `0.05 s / 33,672 KiB`.
Timing/RSS are external diagnostics and intentionally excluded from exact
payload comparison.

## Conclusion and next boundary

V34 now has a target-safe, fresh and model-invariant preregistration. C0 must
independently call P1 on signal-blind structure and publish actual witness IDs,
counts and access receipts twice exactly. It may not import/call oracle,
optimizer, model owner or development metric code. Empty, non-exact or
target-dependent witness sets reject before T0.

SPEC-45 remains `Proposed`. F0 creates no model-quality, real-material,
validator, admission, cooker, demo, runtime or ProductCheck authority.

## Verification

- Ruff format/check and Python compile: `PASS`;
- focused V34 F0 Python suite: `PASS`, `5/5` tests;
- official process A/B: `PASS / REPEAT_EXACT`, `3/3` artifacts and stdout;
- focused xtask physical-sound registry suite: `PASS`, `161/161` tests;
- `git diff --check` and direct documentation/profile/hash checks: `PASS`;
- `cargo run -p xtask -- boundary-scan`: `FAIL`, the known unchanged
  `SOURCE_LAYOUT_ESCAPE_HATCH` in
  `tools/xtask/src/physical_sound_registry_command/realimpact_transfer_fixture.rs`;
- no ProductCheck applies because no production consumer or promoting ADR
  exists.
