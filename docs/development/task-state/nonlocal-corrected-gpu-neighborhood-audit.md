# Nonlocal corrected GPU neighborhood audit — current task state

| Field | Value |
| --- | --- |
| Status | `ACTIVE / NCGA1_CONTRACT_FROZEN / IMPLEMENTATION_NEXT` |
| Updated | `2026-08-29` |
| Task key | `nonlocal-corrected-gpu-neighborhood-audit` |
| Scope | Isolate exact GPU cell-neighborhood construction and stable sample indexing before local assembly or solver work |
| Definition of done | NCGA1 receives one bounded independently reviewed result or stops at its first reproducible failing boundary |
| Authority | Working context only; SPEC-38, ADR-076/081 and the frozen NCGA1 contract outrank this file |

## Resume in 60 seconds

- **Current state:** NCGA0 corrected scalar/full-pair terms are reviewed GO;
  NCGA1 revision 1 is frozen before implementation.
- **Question:** can a device-built signed cell index emit the exact canonical
  integer support graph and stable `SampleId` CSR under boundary cases and
  input permutation?
- **Next action:** implement the independent all-pairs host oracle and isolated
  CUDA radix/cell candidate without touching the historical full solver.
- **Current blocker:** none; RTX 3080 (`sm_86`) and CUDA 13.3 are available.
- **Claim ceiling:** neighborhood/cache/index correspondence only; no local
  assembly, solve, trajectory, performance, runtime or product claim.

## Competing hypotheses

| Hypothesis | Prediction | Discriminator |
| --- | --- | --- |
| H1 exact GPU indexing corresponds | all eight canonical payloads are byte-exact across input permutations | independent integer all-pairs oracle |
| H2 support boundary is mistranslated | exact-radius or one-micrometre-outside membership differs | `support_edge` plus strict-radius negative |
| H3 identity/cell mapping is unstable | negative coordinates or input permutation changes cache/CSR | floor, array-index and same-cell negatives |

## Decisions

### D-001 — Audit canonical integer state, not another shared float path

- **Observation:** SPEC-38 owns sample positions in integer micrometres and
  requires reconstructible stable `(cell key, SampleId)` caches.
- **Evidence:** SPEC-38 candidate state and one-pass cache order; historical
  CUDA uses private float positions and array indices.
- **Conclusion:** a float CPU clone could reproduce the same translation bug.
- **Decision:** exact integer all-pairs host oracle versus integer CUDA cell
  candidate; historical shortcuts become named negative controls.
- **Rejected alternatives:** infer correctness from old `11/11`, or compare two
  implementations sharing the same grid helper.
- **Consequence:** the result is narrower but causally identifies neighborhood
  and stable-index behavior.
- **Reconsider when:** a later private-float execution profile is frozen for a
  complete solver; that requires a separate correspondence contract.

### D-002 — Preserve historical and NCGA0 sources

- **Observation:** old CUDA evidence and NCGA0 are immutable controls.
- **Decision:** add a new executable/translation units; do not patch
  `cuda_baseline.cu`, `oracle.cpp` or corrected-term sources.
- **Consequence:** any result cannot silently rewrite historical evidence.

## Required context

1. `docs/architecture/agent-routing.md`, SPEC-38, SPEC-26, SPEC-21 and
   ADR-076/081/058/027/046/026/071.
2. `docs/roadmap.md` continuum R8 row.
3. `docs/development/task-state/nonlocal-corrected-gpu-audit.md`.
4. `docs/plans/nonlocal-corrected-gpu-neighborhood-audit/00-neighborhood-index-correspondence-contract.md`.

## Do not retry or infer

- do not use the historical full solver as the correctness oracle;
- do not host-sort or host-canonicalize candidate output;
- do not weaken `<=` support membership or remove permutation/negative cases;
- do not report the isolated work counts as game-scale performance;
- do not begin matrix/solver work before this boundary closes.
