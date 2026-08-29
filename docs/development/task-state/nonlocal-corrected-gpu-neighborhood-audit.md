# Nonlocal corrected GPU neighborhood audit — current task state

| Field | Value |
| --- | --- |
| Status | `ACTIVE / NCGA1_AUTHOR_PASS / INDEPENDENT_REVIEW_NEXT` |
| Updated | `2026-08-29` |
| Task key | `nonlocal-corrected-gpu-neighborhood-audit` |
| Scope | Isolate exact GPU cell-neighborhood construction and stable sample indexing before local assembly or solver work |
| Definition of done | NCGA1 receives one bounded independently reviewed result or stops at its first reproducible failing boundary |
| Authority | Working context only; SPEC-38, ADR-076/081 and the frozen NCGA1 contract outrank this file |

## Resume in 60 seconds

- **Current state:** frozen author validation passes: exact CPU/GPU graph roots,
  eight positives, four negatives, ten cold repeats, three sanitizers and both
  retained GPU controls.
- **Question:** can a device-built signed cell index emit the exact canonical
  integer support graph and stable `SampleId` CSR under boundary cases and
  input permutation?
- **Next action:** give snapshot `083e1164` to one fresh independent reviewer;
  keep candidate sources read-only until the verdict.
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

### D-003 — Preflight selects exact GPU cell construction

- **Observation:** the first isolated Release run produces byte-identical CPU
  and GPU graph roots for all eight fixtures; reversed cloud input produces the
  same graph/work roots.
- **Evidence:** development build output root
  `c8b905f78a161fbb1fa40a4bd05c7458e9d47be98343e934f3c1e12c11ca01e8`;
  all four named wrong identities are rejected.
- **Conclusion:** no neighborhood/index mismatch is observed in the corrected
  exact-integer candidate; the clean frozen protocol and review remain open.
- **Decision:** preserve the implementation and proceed only to validation;
  do not add matrix or solver work in NCGA1.

### D-004 — Author validation is complete but not promoted

- **Observation:** two clean Release outputs and binaries are exact; all
  sanitizers report zero errors; NCGA0 and historical tiny controls pass.
- **Evidence:**
  `docs/development/nonlocal-corrected-gpu-neighborhood-audit-evidence-2026-08-29.md`.
- **Conclusion:** the author result is `SUPPORTED_BOUNDED`; independent review
  is the only remaining NCGA1 gate.
- **Decision:** freeze snapshot `083e1164`; no candidate edits or local-assembly
  work before review.

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
