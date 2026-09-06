# Physical sound V24 T0 — analytic teacher result

| Field | Value |
| --- | --- |
| Date | `2026-09-01` |
| Status | `REPEAT_EXACT_PASS / SYNTHETIC_CAUSAL_TEACHER_ONLY / X0_IMPLEMENTATION_AUTHORIZED / MODEL_REAL_MATERIAL_AND_RUNTIME_UNAUTHORIZED` |
| Protocol | [V24 T0 analytic teacher protocol](physical-sound-v24-t0-analytic-teacher-protocol-2026-09-01.md), SHA-256 `c47399a6aa9dd8909dea062a8a1a8f611d9c32baaa22783c96dcbf0cb87c4131` |
| Final implementation | `a6bf36408d3435c0c56fd93ed3da6d95019c78ad` |
| Product effect | None; external synthetic research evidence only, with authored clips still authoritative |

## Result

T0 passes the frozen analytic-teacher contract. The final owning CLI generated
twelve exact plate/beam objects, ten sorted damped modes per object and twelve
contact-conditioned three-second renders per object. Its refined meshes,
modal parameters and signed contact-gain fields form 144 V3-ready
`synthetic_teacher` rows across all five D0 roles. The coarse meshes remain
hash-bound remesh controls rather than extra objects or rows.

Two clean official executions of the final code were byte-identical across all
207 files. Both formula families passed their frozen scaling, boundary,
cantilever-root, remesh, contact-variation and force-linearity controls. This
establishes deterministic causal synthetic supervision only. It does not
establish real Metal, Glass or Wood fidelity, authorize M0 training, validate a
candidate, or define any runtime/public sound contract.

## Exact observations

| Observation | Result |
| --- | --- |
| Final official executions | `PASS / PASS`, byte-identical recursive trees |
| Objects / modes / contacts / lane rows | `12 / 10 per object / 12 per object / 144` |
| Role rows | train `48`; development `24`; calibration `24`; method holdout `24`; admission shadow `24` |
| Generated files | `207` per run: `204` teacher artifacts plus evidence, lane records and report |
| Output size | `85,078,034` bytes per run; frozen limit `256 MiB` |
| Wall time / peak RSS | `1.15 s / 98,880 KiB` and `1.14 s / 98,788 KiB`; frozen limits `300 s / 1 GiB` |
| Manifest SHA-256 | `f7c9cba67627f7fac1717f4c14f7f6f59808635b290d9f107e42412b93e46bec` |
| Implementation-file SHA-256 | `ef5458df64272714f8aaff38b004bd1320bf2a6fa2c58b587505146b18b73617` |
| Artifact-root SHA-256 | `b8c4d82f038b7b592584caf292087b76ed08219ebf2eada12a0285d6796a908c` |
| Teacher-evidence SHA-256 | `884da56ff9005e9dd63e11ec74bafa8796b187b15ed6099fbca99dda7617e7ed` |
| Lane-records SHA-256 | `f31890548c7073e68dc5b3f01a6610b5a60a72398b36c6b679f724b3a5a2fcc6` |
| Report SHA-256 | `95c86cd753aeb90ff49265367ba024bff5abfc3d3a90a1d0e4811ba51ccdf05f` |
| Maximum cantilever characteristic residual | `4.66861256532609e-15`, required `<= 1e-12` |
| Maximum scaling relative error | `0.0` |
| Maximum plate boundary absolute error | `3.252614618251668e-16`, required `<= 1e-12` |
| Remesh / force controls | common-vertex gains byte-exact; impulses `0.5/1.0/2.0` exactly linear |
| Authority flags | model training `false`; real material `false`; runtime `false` |

The external evidence root is
`/tmp/nextengine-v24-t0-official-sSS29H`; generated artifacts are intentionally
not tracked by Git.

## Verification

| Check | Result |
| --- | --- |
| `uv run --project lab python -m py_compile ...` | `PASS` |
| Focused T0 plus existing glass-corpus unit tests | `PASS`, `9 passed / 0 failed` |
| Full official CLI A/B on final code | `PASS`, recursive byte equality |
| Negative mutations | `PASS`: formula, root, material, support, dimensions, mode order/count, mesh binding, contact role, remesh, gain/WAV finiteness, artifact hash, occupied output and interrupted publication reject |
| `git diff --check` | `PASS` |
| `cargo run -p xtask -- boundary-scan` | `FAIL`, known unrelated `SOURCE_LAYOUT_ESCAPE_HATCH` in `realimpact_transfer_fixture.rs` |

The boundary-scan failure names no T0 file and predates this implementation.
The T0 implementation remains below the repository's 1,000-line module limit.

## Decision and next boundary

T0 is complete and authorizes implementing the already frozen X0 Blue Bowl
pilot. X0 may use only the disclosed REALIMPACT contacts and ObjectFolder
recordings named by its protocol; the fifth REALIMPACT contact remains sealed.
T0 and X0 produce separate lane fragments. Only their combination may be
assembled through the completed D0 owner into one V3 manifest.

No neural training starts until X0 and the combined V3 record pass their exact
contract checks. T0 remains synthetic teacher truth, never evidence that a real
material sounds correct.
