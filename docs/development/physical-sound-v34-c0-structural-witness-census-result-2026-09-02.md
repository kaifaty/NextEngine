# Physical sound V34 C0 structural witness census result

| Field | Value |
| --- | --- |
| Date | `2026-09-02` |
| Decision | `PASS / REPEAT_EXACT / ACTUAL_WITNESSES_NONVACUOUS / ZERO_TARGET_MODEL_SIGNAL_ACCESS` |
| Owner commit | `7cf30784be3f51c444f4098c312f66027fb9434b` |
| Owner SHA-256 | `5eedac04fecf2373189be1a19350ab0d0d6e3cec7184fb23bfd66dbfb652c238` |
| F0 profile SHA-256 | `7be7e96a2899d0e4ef7aad668368669886d7f45adac34ec6676afbbe6b45e429` |
| Claim | `SIGNAL_BLIND_P1_STRUCTURAL_WITNESS_CENSUS_ONLY / NO_TARGET_MODEL_QUALITY_REAL_MATERIAL_VALIDATOR_RELEASE_ADMISSION_COOKER_DEMO_OR_RUNTIME_AUTHORITY` |

## Outcome

The frozen C0 owner ran in two fresh CPU processes and solved exactly `864`
P1 structural cases / `8,640` modal rows across development and method holdout.
All three artifacts and stdout match byte-for-byte. Both reports return
`C0_STRUCTURAL_WITNESS_CENSUS_PASS` and authorize only the discarded-fixture
T0 whole-owner terminal-path proof.

The owner imports only the F0 overlay verifier and classical P1 modal solver.
Its AST import audit excludes V33 I0/D0, Torch and the target/model path. It
never constructs normalized feature rows, evaluates an oracle, initializes a
model or calls development metrics.

## Actual witness closure

| Role / stratum | Nodal rows | Material-sensitive rows | Contact-sensitive rows |
| --- | ---: | ---: | ---: |
| Development / contact-only | `180` | `1,080` | `1,080` |
| Development / geometry-only | `222` | `2,160` | `2,160` |
| Development / joint | `183` | `1,080` | `1,080` |
| Method holdout / contact-only | `222` | `1,080` | `1,080` |
| Method holdout / geometry-only | `222` | `2,160` | `2,160` |
| Method holdout / joint | `222` | `1,080` | `1,080` |

Every stratum exceeds the frozen `>=180` exact nodal-row minimum. Counts above
`180` are lawful additional zeros of higher modes at other rational contacts;
their complete sorted row-ID commitments are frozen in `census.json` and were
not used to change the profile.

Each role has `2,016` positive and `2,304` negative nonzero pickup rows. Every
family has `144` exact common-contact remesh cases per role. Non-silent case
counts per family are `30` in contact-only, `66` in geometry-only and `30` in
joint for both roles. Complete case/modal IDs, sign, remesh, nodal,
material/contact sensitivity and non-silent identities have canonical SHA-256
commitments in the census.

## Access and repeatability

Both runs preserve exact zero for train/development/method-holdout target rows,
feature rows, oracle evaluations, model parameters, real/protected signal
values and network requests. Observing P1 structure is explicitly reported
separately and cannot be confused with target access.

| Artifact | Bytes | SHA-256 |
| --- | ---: | --- |
| `census.json` | `17,991` | `e159e0de677fe337267d690991764427c9e83bb9d3f37ec6b3a8ad7426620dde` |
| `evidence.json` | `2,405` | `8540deab4a01d4277c4e2f4e62be2bdb8ee5692f0f23897070858ce5b9aa1128` |
| `report.json` | `1,244` | `51fb66bfdc4f5d6cce7e7a448c1c6faf667236a04669241bd1af16e5e43370e2` |

Repeated stdout SHA-256 is
`e0adc50786a977bc13d5370329b46d9645f67e5bc41a37232b9c0698c6d74b09`.
Run A uses `31.17 s / 45,204 KiB`; run B uses `30.87 s / 44,980 KiB`.
Owner-reported output closure is `21,640` bytes in both runs; wall/RSS remain
external non-deterministic diagnostics.

## Conclusion and next boundary

The V33 failure class is now closed at the corpus boundary: each official
evaluation stratum contains real P1 witnesses for every declared hard-gate
class before any target exists. T0 must next prove the second failure boundary:
the complete discarded-fixture owner must publish Pass, metric/hard/resource
rejects atomically after access and allow exceptions only before access.

No official V34 target or model value may open before T0 passes twice exactly.
SPEC-45 remains `Proposed`; C0 earns no model-quality, real-material,
validator, admission, cooker, demo, runtime or ProductCheck authority.

## Verification

- Ruff format/check and Python compile: `PASS`;
- focused V34 C0 Python suite: `PASS`, `5/5` tests;
- official process A/B: `PASS / REPEAT_EXACT`, `3/3` artifacts and stdout;
- focused xtask physical-sound registry suite: `PASS`, `161/161` tests;
- `git diff --check` and direct documentation/link/task-state checks: `PASS`;
- `cargo run -p xtask -- boundary-scan`: `FAIL`, the known unchanged
  `SOURCE_LAYOUT_ESCAPE_HATCH` in
  `tools/xtask/src/physical_sound_registry_command/realimpact_transfer_fixture.rs`;
- no ProductCheck applies because no production consumer or promoting ADR
  exists.
