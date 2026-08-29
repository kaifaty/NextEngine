# Nonlocal corrected GPU audit — current task state

| Field | Value |
| --- | --- |
| Status | `ACTIVE / NCGA0_REV1_NO_GO / REV2_CONTRACT_FROZEN` |
| Updated | `2026-08-29` |
| Task key | `nonlocal-corrected-gpu-audit` |
| Scope | Determine whether corrected Nonlocal scalar/pair terms correspond to a separate strict-f32 CUDA implementation before any neighborhood or solver port |
| Definition of done | NCGA0 receives one bounded reviewed result or stops at its first reproducible failing boundary |
| Authority | Working context only; SPEC-38, ADR-076/081, the FCR1 formula contract and frozen evidence outrank this file |

## Resume in 60 seconds

- **Current conclusion:** revision 1 is `NO-GO`: both force evaluators agreed,
  but the contract named directed-edge viscosity coefficients while the output
  was the full undirected-pair force.
- **Why:** the reviewer exhibited a shared factor-of-two interpretation that
  closure and the kernel-only negative control could not distinguish.
- **Next action:** implement frozen revision 2 with a separate energy finite
  difference and a half-force negative control, then use the single re-review.
- **Current blocker:** none.
- **Do not retry:** another long source-shaped 50k run; it cannot distinguish
  shared source mathematics from GPU translation.
- **Reconsider when:** NCGA0 term correspondence closes and a separately
  frozen neighborhood/linearization consumer exists.

## Current evidence

| Evidence | Result | Consequence |
| --- | --- | --- |
| `docs/development/nonlocal-continuum-nr1-rc1-evidence-2026-08-19.md` | `NR1_RECLOSED_GATHER_DIRECTED / REPORT_ONLY` | historical atomic ordering bug is real and the retained gather removes it on the frozen host |
| `docs/development/nonlocal-continuum-npr1b-term-controls-evidence-2026-08-20.md` | `FORMULA_MISMATCH` | source CPU/GPU equality cannot establish energy consistency |
| `docs/plans/nonlocal-corrected-gpu-audit/00-corrected-term-correspondence-contract.md` | `FROZEN` | only nine term fixtures are authorized in NCGA0 |
| `docs/development/nonlocal-corrected-gpu-term-audit-evidence-2026-08-29.md` | `AUTHOR_PASS / REVIEW_NO_GO` | execution is reproducible, but revision-1 semantics did not close |
| `docs/development/nonlocal-corrected-gpu-term-audit-independent-review-2026-08-29.md` | `NO-GO / REFUTED_AS_WRITTEN` | revision-1 agreement shared an untested viscosity observable |
| `docs/plans/nonlocal-corrected-gpu-audit/01-full-pair-force-correspondence-contract.md` | `FROZEN / REPAIR_PENDING` | full-pair force and independent energy derivative are now explicit |

## Decisions that still constrain the work

### D-001 — Audit a new identity, preserve the historical GPU path

- **Observation:** the old CUDA kernels intentionally transcribe the pinned
  PeriDyno/source-shaped arithmetic.
- **Evidence:** `cuda_baseline.cu`, retained `11/11` correspondence and NPR1-B.
- **Decision:** add one separate corrected term target and keep the old binary
  as a negative/non-regression control.
- **Rejected alternatives:** patching old kernels in place or inferring
  corrected physics from their source-shaped correspondence.
- **Consequences:** old performance roots remain historical only; NCGA0 emits
  report-only JSON and owns no simulation state.
- **Uncertainty:** whether strict binary32 preserves corrected term values even
  before neighborhood and solver composition.
- **Reconsider when:** NCGA0 closes under independent review.

### D-002 — Distinguish full-pair force from directed-edge contribution

- **Observation:** revision 1 used directed-edge coefficient words for a
  full-pair endpoint-force output.
- **Evidence:** independent review of candidate `349f12e6`; both evaluators use
  `lambda,2*mu`, while the literal text named `lambda/2,mu`.
- **Conclusion:** CPU/CUDA equality and closure cannot select between identities
  when both implementations share the same factor.
- **Decision:** revision 2 defines the full energy gradient explicitly and adds
  an independent energy finite difference plus a half-force negative.
- **Rejected alternatives:** changing the force to half merely to satisfy the
  ambiguous phrase; accepting revision 1 by explanation alone.
- **Consequences:** revision 1 remains NO-GO; one repair/re-review remains.
- **Uncertainty:** whether another shared local-term defect survives the added
  energy discriminator.
- **Reconsider when:** revision 2 receives its single re-review.

## Open hypotheses

| Hypothesis | Evidence for | Evidence against | Next discriminator |
| --- | --- | --- | --- |
| H1 corrected full-pair terms translate to CUDA | 9/9 values and repeats passed | revision-1 viscosity meaning was not independently selected | energy derivative plus half-force negative |
| H2 GPU-specific coefficient/sign defect remains | prior atomics defect shows GPU-specific errors are possible | no local discrepancy appeared | repaired independent term audit, then neighborhood audit |
| H3 shared source mathematics caused the main historical failure | old CPU/GPU correspond and old derivative control fails | revision 1 showed a new shared-oracle risk | independent energy and later matrix oracles |

## Required context

1. `docs/architecture/agent-routing.md`, SPEC-38, ADR-076 and ADR-081.
2. `docs/roadmap.md` continuum R8 row and `docs/plans/continuum-water/README.md`.
3. `docs/plans/nonlocal-continuum-formula-reclosure/00-formula-contract.md`.
4. NPR1-B and NR1-RC1 evidence.
5. `docs/plans/nonlocal-corrected-gpu-audit/00-corrected-term-correspondence-contract.md`.

## Next action

1. Add a separate long-double viscosity energy/finite-difference translation
   unit without reusing force code.
2. Add a CUDA half-force variant and require both viscosity cases to reject.
3. Rerun all revision-1 gates, freeze the repaired snapshot and obtain the
   single permitted re-review.

## Do not retry

- Historical atomic surface timing — correctness already failed and gather
  reclosed the bounded issue.
- Old source-shaped 50k performance as product evidence — NPR1-B invalidates
  its physics credit.
- Full corrected solver port before NCGA0 — term-level failure would make it
  wasted work.

## Handoff

- **Workspace state:** primary checkout, branch `codex/water-research`.
- **Checks:** revision-1 execution gates passed, but independent semantic review
  returned NO-GO; revision-2 executable checks pending.
- **Remaining risk:** local matrix, neighborhood, solver and trajectory remain
  outside NCGA0 even if it passes.
- **Promotion needed:** none; the track remains Proposed/report-only.
