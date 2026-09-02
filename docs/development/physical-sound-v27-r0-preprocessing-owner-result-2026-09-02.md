# Physical sound V27 R0 — deterministic preprocessing owner result

| Field | Value |
| --- | --- |
| Date | `2026-09-02` |
| Status | `REPEAT_EXACT_CONTRACT_PASS / OFFICIAL_CONTEXT_PREFLIGHT_PASS / MODEL_VALUES_UNOPENED / R1_AUTHORIZED` |
| Roadmap package | V27 `R0` |
| Protocol | [P0a m0b-v1.1](physical-sound-v26-p0a-barycentric-and-padded-alignment-protocol-2026-09-02.md), SHA-256 `54522c26eb3db62ad316ea6001f9380651f760329f0db8556b441ad286b649e8` |
| Implementation commit | `db85c799c44b8a8d17fd0afe5442b46fd208a4a9` |
| Implementation root | `d013ec35f6499cfc17f5beeec3db8d245f2a296b88a104cc80047b2ece04f456` |
| Product effect | None; external feasibility tooling only, with authored clips authoritative |

## Question and answer

Can one independent M0b owner apply the frozen barycentric surface query and
bounded padded alignment rules through the complete entry point without
editing M0a, opening model values or weakening a role/gate?

**Yes for implementation conformance.** The contract fixture passes twice
byte-exactly and publishes nine canonical artifacts. A signal-blind preflight
over the real official context inputs also passes. This authorizes one frozen
R1 feasibility execution; it establishes no neural quality, material identity,
automatic admission, public content record or runtime capability.

## Implemented boundary

The new owner consists of five modules:

- `physical_sound_v26_m0b_common.py` binds P0a, inherited hashes, manifest and
  the complete nine-module implementation root;
- `physical_sound_v26_m0b_surface.py` validates full meshes, resolves one
  canonical face and interpolates float64 vertex fields at arbitrary legal
  surface points;
- `physical_sound_v26_m0b_alignment.py` performs first-absolute-maximum,
  bounded positive-zero padding without resampling or amplitude changes;
- `physical_sound_v26_m0b_preprocess.py` replaces only exact-vertex lookup and
  unpadded transfer alignment while retaining the M0a descriptors and roles;
- `physical_sound_v26_m0b_train.py` owns atomic orchestration and adds
  `surface-query-report.json` to the inherited eight-artifact output.

Every inherited M0a hash still equals the frozen P0/P0a map. No M0a file was
edited. Model ID, seed, capacity, optimizer, losses, schedule, controls,
thresholds, role order and resource limits are unchanged.

## Contract evidence

Two complete `contract-fixture-v1` CLI executions:

- emitted byte-identical directory trees with exactly nine files;
- returned fixture decision `PASS` with `contract_fixture_has_no_quality_claim`;
- kept admission shadow and REALIMPACT row `2407` unopened;
- ran 48 official-shape queries and 24 coarse/refined affine pairs;
- passed four face enumeration/cycle/winding invariance cases;
- rejected 15 mesh/query/field/tolerance mutations;
- passed nine alignment cases and rejected eight alignment mutations;
- exercised an off-vertex synthetic row and off-vertex transfer rows;
- rejected protocol, own root, inherited hash, occupied-output and interrupted
  publication mutations.

The combined inherited/new unit suite reports `10/10` passing tests. Re-running
the four inherited M0a hashes produced exactly:

| Module | SHA-256 |
| --- | --- |
| `physical_sound_v25_m0a_common.py` | `592b38137eb088e4ffc8087c3c68c0c19b0a8ceed8d049e7e9b2f6a002573e89` |
| `physical_sound_v25_m0a_evaluate.py` | `e5c48a74f5df89d60433d2ef1605bce648250836809e04721a2aa1fefe3e552a` |
| `physical_sound_v25_m0a_model.py` | `6075b4121c5eb45aca2934308fb4e73c078c467669e458c0d36b65b066f4009b` |
| `physical_sound_v25_m0a_train.py` | `86ada58126ad901021cdf297cef2fc46059dc33763e960dbdc21ebf3972630b5` |

## Official-context preflight

The preflight validated the already-frozen official source references through
the M0a manifest SHA-256
`37e3418a2de4bb7bc70eede12ae91e0d0120642dbd0861cbae2d600f8ecbd7e2`,
then substituted only the committed M0b protocol/root/hash identities in
memory. It did not construct a model, execute a training step, open a query or
write an official candidate manifest.

| Evidence | Result |
| --- | ---: |
| Official-shape queries / pairs | `48 / 24` |
| Official-shape mutations | `15` |
| T0 train-context rows preprocessed | `32` |
| Actual surface evaluations | `67` (`32 × 2` T0 plus `3` X0) |
| REALIMPACT adaptation contexts | `3` |
| ObjectFolder adaptation contexts | `2` |
| Query / protected roles opened | `0 / 0` |
| Model constructed / model values opened | `false / false` |

The three disclosed adaptation contexts reproduce the exact P0a facts:

| Row | Peak | Leading pad | Copied | Trailing pad |
| --- | ---: | ---: | ---: | ---: |
| `x0-realimpact-blue-bowl-contact-000` | `87` | `425` | `143575` | `0` |
| `x0-realimpact-blue-bowl-contact-001` | `73` | `439` | `143561` | `0` |
| `x0-realimpact-blue-bowl-contact-002` | `39` | `473` | `143527` | `0` |

## Verification

| Check | Result |
| --- | --- |
| `uvx ruff check` on five modules and owning test | `PASS` |
| `uvx ruff format --check` | `PASS`, six files already formatted |
| Python byte compilation | `PASS` |
| M0a plus M0b focused suites | `PASS`, `10` tests in `27.428 s` |
| Real official-context preprocessing preflight | `PASS` |
| `cargo run -p xtask -- boundary-scan` | `FAIL`, known unrelated `SOURCE_LAYOUT_ESCAPE_HATCH` in `tools/xtask/src/physical_sound_registry_command/realimpact_transfer_fixture.rs` |

The boundary-scan failure predates R0 and names no changed R0 file. It remains
a repository-wide hygiene issue, not a reason to claim a passing broad scan.

## Decision and next action

R0 is complete. Freeze one external `official-v1` M0b manifest against protocol
`54522c26…49e8`, implementation root `d013ec35…f456` and the unchanged official
source hashes. Then execute R1 A/B once under the inherited `1,800 s`, `4 GiB`,
`256 MiB`, CPU/one-thread and network-denied limits.

Do not change preprocessing, model, seed, losses, schedule, roles or gates
after any R1 value opens. Run B only after run A completes successfully. A
conformance/resource failure publishes no quality claim; a representation or
domain-gap decision closes or branches exactly as Roadmap V27 specifies.
