# Physical sound V24 X0 — disclosed Blue Bowl pilot result

| Field | Value |
| --- | --- |
| Date | `2026-09-01` |
| Status | `REPEAT_EXACT_PASS / COMBINED_V3_VALIDATED_AXIS_INCOMPLETE_BY_DESIGN / M0_PROTOCOL_FREEZE_AUTHORIZED / TRAINING_REAL_ADMISSION_AND_RUNTIME_UNAUTHORIZED` |
| Protocol | [V24 X0 Blue Bowl protocol](physical-sound-v24-x0-blue-bowl-pilot-protocol-2026-09-01.md), SHA-256 `74b80bb02d51d3ed527c179c40ef1928c61dd8947a7abb16cbc266c2333fd14b` |
| X0 implementation | `84b25193bb4abd14c3bdec832c80461f4eb0fba1` |
| V3 assembler | `23c674c7760fd956ada57d0346deccf32f9dbcb5` |
| Product effect | None; disclosed development evidence only, with authored clips still authoritative |

## Result

X0 passes its frozen structural and provenance question twice exactly. The
owner accepted the four previously disclosed REALIMPACT `6_Bowl` transfer
rows and three ObjectFolder `6 / Blue_Bowl / Glass` recordings without
inventing support, excitation, composition, impact-normal or recording
contact/listener claims.

Each official X0 run bounded-refetched only the exact compressed OBJ and
listener-coordinate members from the official archive. It did not fetch or
decompress the 2.39 GiB transfer member. The existing four-row array remained
float32 little-endian with shape `4 x 230215`, all four row hashes were
distinct, the canonical mesh contained `47,738` vertices, and every disclosed
impact position matched its exact vertex. REALIMPACT row `2407` remained a
commitment with zero decoded samples and is absent from every lane row and D0
projection.

The new assembler then combined 144 T0 rows and seven X0 rows into one
hash-closed 151-row V3 manifest. Two executions of the actual Rust
`physical-sound-registry neural-data-plane` owner were byte-identical and
reported `Validated`, all 151 lane contracts complete and no cross-role
leakage.

The owner's decision is intentionally `DeclaredAxisCoverageIncomplete`: the
seven real rows keep their genuinely absent axes absent. This is a successful
X0 honesty gate, not an authorization to supervise those missing targets. M0
may now be preregistered to use synthetic full-axis supervision plus only the
observed real losses, but no training value may open before that protocol.

## Exact observations

| Observation | Result |
| --- | --- |
| X0 official executions | `PASS / PASS`, eight output files byte-identical |
| X0 rows | exact transfer `4`; identified recording `3`; all `development` |
| Original / derived mesh SHA-256 | `f23127b45b0b163c796b4ef44d707b59fac58c4ef6e94e7676a4d88470e7f973` / `f207ed633557e2d0bd2f510815495c31cec1a813e63db3421d1d57adef1f025a` |
| Mesh size | `47,738` vertices; `15,914` canonical triangles |
| X0 output size | `5,039,546` bytes per run |
| X0 wall time / peak RSS | `3.81 s / 69,780 KiB`; `3.63 s / 71,272 KiB` |
| X0 manifest SHA-256 | `da2b1f8ffc2378a6b5ae6778256780c2ee1051de280e7370f955af62229a29bc` |
| X0 lineage / lane / report SHA-256 | `e4f6bb117a09f77b2bf8602ec895c95ae63262bc9d42db1233d972d73242534f` / `fe869bb247f25da15d7e62360451dbb236d698ad0f26079badf845d7b9dca545` / `2ae6a01e979f5a715b91d54654fb09bbf022461e75ee1477265e743482529735` |
| Sealed contact | row `2407`; decoded samples `0`; no materialized row/hash/samples |
| Combined V3 manifest | `151` rows, `626,031` bytes, SHA-256 `c43ba8eac68e32a8ef8fbe37d3ffa3d21db1d59e0443766cee1d98f51cad70bb` |
| Combined lanes | synthetic teacher `144`; exact real transfer `4`; identified real recording `3` |
| Combined roles | train `48`; development `31`; calibration `24`; method holdout `24`; admission shadow `24` |
| D0 owner | `Validated / DeclaredAxisCoverageIncomplete`; `151/151` lane contracts complete |
| D0 leakage | all six disjointness checks true; `NoCrossRoleLeakage` |
| D0 A/B report SHA-256 | `ea6a343d7b92226269bfabb4a1aeb279f0d097e17353a951e4630846830ddd66` |
| Authority flags | model training `false`; real-material admission `false`; runtime `false`; protected roles unmaterialized |

External evidence remains under
`/tmp/nextengine-v24-x0-official-1RipdA`; no dataset, source recording,
generated mesh, transfer row or projection is tracked by Git.

## Verification

| Check | Result |
| --- | --- |
| X0 plus T0/assembler focused Python tests | `PASS`, `13 passed / 0 failed` |
| X0 official owning CLI A/B | `PASS`, recursive byte equality |
| V3 assembler A/B | `PASS`, manifest SHA-256 identical |
| Rust D0 owning CLI A/B | `PASS`, four emitted files byte-identical |
| X0 mutations | every predecessor/audio/mesh hash, dtype, shape, finiteness, contact vertex/position, listener, material, semantics, missing/fabricated axis, sealed counter, occupied output and interrupted publication reject |
| `git diff --check` | `PASS` |
| `cargo run -p xtask -- boundary-scan` | `FAIL`, known unrelated `SOURCE_LAYOUT_ESCAPE_HATCH` in `realimpact_transfer_fixture.rs` |

The first D0 invocation included a debug rebuild (`8.04 s`, peak
`2,605,976 KiB`); the immediately repeated owner invocation was `2.70 s`,
peak `63,900 KiB`. These are tooling observations, not a runtime or
production-cost claim.

## Decision and next boundary

X0 and the combined V3 evidence plane are complete. They authorize freezing
M0's exact-object neural-student protocol against these hashes. That protocol
must declare the model, deterministic initialization, losses per evidence lane,
classical controls, causal ablations, protected synthetic roles, disclosed-real
development-only metrics, resource ceiling and fail/stop rules before any
checkpoint or model-derived value opens.

No protected real role, Metal admission, validator threshold, cooker or demo
integration is authorized by this result.
