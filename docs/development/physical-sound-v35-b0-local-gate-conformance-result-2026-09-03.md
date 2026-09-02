# Physical sound V35 B0 local-expert and coverage-gate conformance result

| Field | Value |
| --- | --- |
| Date | `2026-09-03` |
| Decision | `PASS / REPEAT_EXACT / CONTINUOUS_LEAKAGE_FREE_LOCAL_GATE_EXECUTION / ZERO_OFFICIAL_TARGET_MODEL_ACCESS` |
| Owner commit | `b64a3a51569606e8b6190b6c78f34cf9768455c6` |
| Owner SHA-256 | `e92924fa25269ca28a0afcb8cc581040152ab6795c5e98be010f36dcbcfd6c8d` |
| F0 profile SHA-256 | `019cadd51254c27da32e35ab39b35551ba965198f33282feb975af83cb9304cc` |
| Claim | `DISCARDED_ANALYTIC_LOCAL_EXPERT_AND_COVERAGE_GATE_CONFORMANCE_ONLY / NO_OFFICIAL_TARGET_MODEL_QUALITY_REAL_MATERIAL_VALIDATOR_RELEASE_ADMISSION_COOKER_DEMO_OR_RUNTIME_AUTHORITY` |

## Outcome

The committed B0 owner ran in two fresh CPU processes and returned
`B0_LOCAL_EXPERT_AND_COVERAGE_GATE_CONFORMANCE_PASS` both times. It executed
the frozen local interpolation, causal group cross-fit, continuous coverage
gate, bounded blend and strict OOD policy over a discarded analytic corpus with
the official `6,480`-row train shape. Both runs publish identical
`conformance.json`, `evidence.json`, `report.json` and stdout byte-for-byte.

B0 imports only the target-safe V35 F0 owner and binds the committed C0 owner
and result. It imports no P1 oracle, prior tournament/model owner or Torch. No
official V35 role row, target, model parameter, oracle value, prior-generation
value or real/protected signal is materialized.

## Local interpolation and leakage closure

- all `6,480` discarded rows form `30` causal
  family/support/mode-ordinal partitions of exactly `216` rows;
- every group-cross-fitted query uses exactly `215` compatible contributors
  after excluding the complete case/remesh group;
- the `6,480` cross-fit predictions commit to
  `753136a19e07c0345d700e5af00e9b7bc061b6ac43fe9ad625a7de39ef8cc63a`;
- canonical train identity is
  `40241cdd991b1d161e36a64714a4224cb47f4643d0c0986755e13840cd8f46f8`;
- permutations `35001`, `35002` and `35003` produce the same train identity
  and the same 90-probe prediction commitment
  `84206dfd6aacd2f5318bd79fcdf457f46b8ad2d1a32f81eb7f47367864ecb4f6`;
- a zero-distance query still evaluates the same Gaussian weighted-sum
  equation over all `216` rows; its prediction differs from the exact stored
  target, so there is no nearest-row or label-copy shortcut;
- conflicting duplicate causal rows and non-finite targets fail closed.

The implementation has bounded `O(compatible_rows * 15)` time,
`O(compatible_rows)` scratch and a frozen maximum of `216` contributors. It
sorts by causal keys and uses `math.fsum`; trace IDs do not influence the
result.

## Continuity, blend and OOD closure

Positive perturbations of `2^-20`, `2^-21` and `2^-22` were applied separately
to all three geometry coordinates and both contact coordinates. For every axis,
the local prediction delta, coverage-weight delta and final hybrid delta remain
finite and refine monotonically as the perturbation halves. This covers the
frozen continuous equation without inspecting scientific targets.

The implementation AST contains zero equality comparisons in the local
prediction and coverage-gate functions. The exact-support gate yields local
weight `0.8`; squared distance just below `16` is `InDomain`, while exactly
`16` and just above it are deterministically `FallbackOutOfDomain`. Bounded
blend probes remain inside the frozen `[-0.25, 0.25]` neural-output envelope.
Forbidden object, role, stratum, geometry-cell, contact-set, mesh, target,
oracle and validator identifiers have an empty intersection with local/gate
inputs.

## Exact access receipt

Every official forbidden counter is zero in both runs: train, development and
method-holdout targets; feature rows; oracle values; model parameters; prior
targets, predictions, weights and metrics; real/protected signal values; and
network requests. The `6,480` discarded analytic targets and `270` discarded
permutation probes are reported separately and have no scientific or quality
authority.

## Repeatability and resources

| Artifact | Bytes | SHA-256 |
| --- | ---: | --- |
| `conformance.json` | `8,864` | `4af19e8fbda4ed9da6174519904035a2a4c164af26b34a3c927038ad9de69bae` |
| `evidence.json` | `2,009` | `7d9f3da47b432efff4e14bf6bcc4607a7024f9d1d8f8b6bbf5b4f99f0c104ef8` |
| `report.json` | `789` | `8a667056c453ed9c4e05931dd6b952dec014f87dea4708ad7e188aa888386150` |

Repeated stdout has `210` bytes and SHA-256
`cc8acca9a490816019cbdcefa91ae82ffa8433bf43d63dc71c226a7d46bc93d3`.
Runs take `5.65 / 5.82 s`; maximum RSS is `100,888 / 100,428 KiB`. The
owner reports the same `11,662` output bytes. Timing and RSS are external
diagnostics, not exact-payload fields, and remain within `300 s / 1 GiB`.

## Conclusion and next boundary

B0 closes the executable local/gate mechanics before official target access.
I0 may now build one discarded official-shape complete-owner proof. It must
import the frozen V34 terminal publisher and this B0 owner unchanged, exercise
complete Pass/metric/hard/resource/pre-access paths atomically and repeat all
artifacts/stdout exactly. Official V35 targets and model parameters remain
forbidden until I0 passes and a separately committed D0 owner is frozen.

SPEC-45 remains `Proposed`. B0 creates no model-quality, real-material,
validator, admission, cooker, demo, runtime or ProductCheck authority.

## Verification

- Ruff format/check and Python compile: `PASS`;
- focused V35 B0 Python suite: `PASS`, `6/6` tests;
- full discarded-shape smoke: `PASS`, `6,480` cross-fit predictions;
- official process A/B: `PASS / REPEAT_EXACT`, `3/3` artifacts and stdout;
- focused xtask physical-sound registry suite: `PASS`, `161/161` tests;
- `git diff --check` and direct documentation/link/task-state checks: `PASS`;
- `cargo run -p xtask -- boundary-scan`: `FAIL`, known unchanged
  `SOURCE_LAYOUT_ESCAPE_HATCH` in
  `tools/xtask/src/physical_sound_registry_command/realimpact_transfer_fixture.rs`;
- no ProductCheck applies because there is no production consumer or promoting
  ADR.
