# Physical sound V35 C0 witness and hybrid-coverage census result

| Field | Value |
| --- | --- |
| Date | `2026-09-02` |
| Decision | `PASS / REPEAT_EXACT / P1_WITNESSES_AND_HYBRID_SUPPORT_NONVACUOUS / ZERO_TARGET_MODEL_PRIOR_VALUE_OR_SIGNAL_ACCESS` |
| Owner commit | `f5f570d47e7811977cd80a6908d6aae5fa986c10` |
| Owner SHA-256 | `c0f5aa7ea02e51b1c43e46d95ae0fd47518f411759ca1b32dd9d7f5b6def9cd7` |
| F0 profile SHA-256 | `019cadd51254c27da32e35ab39b35551ba965198f33282feb975af83cb9304cc` |
| Claim | `SIGNAL_BLIND_P1_WITNESS_AND_GEOMETRY_HYBRID_COVERAGE_CENSUS_ONLY / NO_TARGET_MODEL_QUALITY_REAL_MATERIAL_VALIDATOR_RELEASE_ADMISSION_COOKER_DEMO_OR_RUNTIME_AUTHORITY` |

## Outcome

The committed C0 owner ran in two fresh CPU processes and solved all `1,512`
P1 structural cases / `15,120` modal rows across train, development and method
holdout. Both runs return
`C0_WITNESS_AND_GEOMETRY_HYBRID_COVERAGE_PASS`, authorize only
`V35-B0-discarded-local-and-gate-conformance`, and publish identical
`census.json`, `evidence.json`, `report.json` and stdout byte-for-byte.

The owner imports only the target-free V35 F0 verifier and classical P1 modal
owner. Its AST boundary rejects V33/V34 model/tournament owners and Torch. It
does not evaluate an oracle, construct a target, initialize a model, read an
earlier target/prediction/weight/metric value or decode a real/protected signal.

## Actual P1 witness closure

| Role / stratum | Nodal rows | Material-sensitive | Contact-sensitive | Both experts reachable |
| --- | ---: | ---: | ---: | ---: |
| Development / contact-only | `243` | `1,080` | `1,080` | `1,080` |
| Development / geometry-only | `180` | `2,160` | `2,160` | `2,160` |
| Development / joint | `243` | `1,080` | `1,080` | `1,080` |
| Method holdout / contact-only | `198` | `1,080` | `1,080` | `1,080` |
| Method holdout / geometry-only | `180` | `2,160` | `2,160` | `2,160` |
| Method holdout / joint | `204` | `1,080` | `1,080` | `1,080` |

Every stratum meets the frozen `>=180` nodal minimum. Counts above the minimum
are lawful additional analytic nodes and did not select any profile value.
Each evaluation role also has `2,016` positive and `2,304` negative pickup
rows. Every family contributes `144` exact remesh cases per role and finite
non-silent case counts of `30 / 66 / 30` for contact-only, geometry-only and
joint transfer.

## Hybrid support closure

- every one of the `8,640` evaluation rows has exactly `216` compatible
  train-only rows in its family/support/mode-ordinal partition;
- every one of the `6,480` train rows retains exactly `215` compatible rows
  after excluding its complete causal case/remesh group;
- Gaussian denominators and all 15-field causal distances are finite-positive;
- every evaluation row is inside the frozen normalized squared OOD radius
  `<16`; OOD count is exactly zero in all six role/stratum cells;
- local weights span `0.1546010256516271 .. 0.7341994450454713`, so local and
  neural experts are both strictly reachable on every evaluation row;
- local/gate inputs have an empty intersection with object, record, role,
  stratum, geometry-cell, contact-set, mesh, target, oracle and validator IDs;
- the structural bandwidths reproduce F0 exactly:
  contact `0x1.e6d4df96cc6b2p-2`, geometry `0x1.ac6db4c237254p-1`.

The census computes no local prediction because doing so would require target
values. Continuity, permutation invariance, exact group leave-out behavior and
zero-distance execution remain B0 obligations on discarded analytic fields.

## Exact access receipt

Every forbidden counter is zero in both runs:

| Access | Count |
| --- | ---: |
| Train/development/method-holdout target rows | `0 / 0 / 0` |
| Feature rows materialized | `0` |
| Oracle values evaluated | `0` |
| Model parameters initialized | `0` |
| Prior-generation target/prediction/weight/metric values read | `0 / 0 / 0 / 0` |
| Real/protected signal values decoded | `0 / 0` |
| Network requests | `0` |

P1 structural rows are reported separately and cannot be reclassified as
target access. Method-holdout structure is inspected exactly as preregistered;
its targets and metrics remain unopened.

## Repeatability and resources

| Artifact | Bytes | SHA-256 |
| --- | ---: | --- |
| `census.json` | `26,727` | `139334dd737e34f794eb9a3180f9d6584591507c1aa890805974d8c7e2ae87cf` |
| `evidence.json` | `2,641` | `95f54845a8c608acfa37d0705fa5cde5d5b3e43c07d8cb3ccc2e78d96f0af3cd` |
| `report.json` | `1,473` | `6942708c07c7c96f0915a60cec8944ddab9c8bbd1527dafd71ac4fb616957619` |

The repeated stdout has `204` bytes and SHA-256
`ef218be5a2547f9ec485a8c3c7eae677096f2a7b72be99c19e8827f67eb598b0`.
Runs take `61.02 / 57.54 s`; peak RSS is `62,972 / 63,208 KiB`. The owner
reports the same `30,841` output bytes. Time and RSS are external diagnostics,
not exact-payload fields, and both remain within `300 s / 1 GiB`.

## Conclusion and next boundary

C0 closes the structural-vacuity risk before target access. B0 must now execute
the frozen local interpolator and gate on discarded analytic fields and prove
continuity, canonical train-permutation invariance, case/remesh leave-out,
same-equation zero-distance behavior, bounded blending and deterministic OOD
twice exactly. Official V35 targets and model parameters remain forbidden until
B0 and then I0 pass.

SPEC-45 remains `Proposed`. C0 creates no model-quality, real-material,
validator, admission, cooker, demo, runtime or ProductCheck authority.

## Verification

- Ruff format/check and Python compile: `PASS`;
- focused V35 C0 Python suite: `PASS`, `6/6` tests;
- full smoke census: `PASS`, `1,512 / 15,120` structural rows;
- official process A/B: `PASS / REPEAT_EXACT`, `3/3` artifacts and stdout;
- focused xtask physical-sound registry suite: `PASS`, `161/161` tests;
- `git diff --check` and direct documentation/link/task-state checks: `PASS`;
- `cargo run -p xtask -- boundary-scan`: `FAIL`, known unchanged
  `SOURCE_LAYOUT_ESCAPE_HATCH` in
  `tools/xtask/src/physical_sound_registry_command/realimpact_transfer_fixture.rs`;
- no ProductCheck applies because there is no production consumer or promoting
  ADR.
