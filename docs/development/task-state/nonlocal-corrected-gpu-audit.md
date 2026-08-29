# Nonlocal corrected GPU audit — current task state

| Field | Value |
| --- | --- |
| Status | `ACTIVE / NCGA0_EXECUTED / INDEPENDENT_REVIEW_PENDING` |
| Updated | `2026-08-29` |
| Task key | `nonlocal-corrected-gpu-audit` |
| Scope | Determine whether corrected Nonlocal scalar/pair terms correspond to a separate strict-f32 CUDA implementation before any neighborhood or solver port |
| Definition of done | NCGA0 receives one bounded reviewed result or stops at its first reproducible failing boundary |
| Authority | Working context only; SPEC-38, ADR-076/081, the FCR1 formula contract and frozen evidence outrank this file |

## Resume in 60 seconds

- **Current conclusion:** corrected scalar/pair terms correspond between an
  independent host `long double` oracle and strict CUDA `f32` on all nine NCGA0
  fixtures; the result remains bounded and review-pending.
- **Why:** 9/9 corrected cases passed, ten cold repetitions were byte-exact,
  the source-shaped negative rejected all three mandatory cases, and CUDA
  memcheck/initcheck/synccheck reported zero errors.
- **Next action:** freeze the exact candidate snapshot and obtain one
  independent read-only review before accepting `SUPPORTED_BOUNDED`.
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
| `docs/development/nonlocal-corrected-gpu-term-audit-evidence-2026-08-29.md` | `SUPPORTED_BOUNDED / REVIEW_PENDING` | corrected term translation closes locally; neighborhood and solver claims remain forbidden |

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

## Open hypotheses

| Hypothesis | Evidence for | Evidence against | Next discriminator |
| --- | --- | --- | --- |
| H1 corrected terms translate to CUDA | 9/9 NCGA0 cases and ten exact repeats pass | neighborhood composition remains untested | independent review of frozen candidate |
| H2 GPU-specific coefficient/sign defect remains | prior atomics defect shows GPU-specific errors are possible | no local term defect appears in NCGA0 | neighborhood/assembly audit after review |
| H3 shared source mathematics caused the main failure | old CPU/GPU correspond; old derivative control fails; corrected CPU/GPU terms pass | does not exclude later GPU neighborhood/solver defects | corrected neighborhood/linearization comparison |

## Required context

1. `docs/architecture/agent-routing.md`, SPEC-38, ADR-076 and ADR-081.
2. `docs/roadmap.md` continuum R8 row and `docs/plans/continuum-water/README.md`.
3. `docs/plans/nonlocal-continuum-formula-reclosure/00-formula-contract.md`.
4. NPR1-B and NR1-RC1 evidence.
5. `docs/plans/nonlocal-corrected-gpu-audit/00-corrected-term-correspondence-contract.md`.

## Next action

1. Freeze candidate source, binary, stdout, fixture/result roots and exact diff.
2. Obtain the one allowed independent read-only NCGA0 review.
3. If the review is clean, close NCGA0 and draft (but do not silently execute)
   the next neighborhood/linearization contract.

## Do not retry

- Historical atomic surface timing — correctness already failed and gather
  reclosed the bounded issue.
- Old source-shaped 50k performance as product evidence — NPR1-B invalidates
  its physics credit.
- Full corrected solver port before NCGA0 — term-level failure would make it
  wasted work.

## Handoff

- **Workspace state:** primary checkout, branch `codex/water-research`.
- **Checks:** corrected target 9/9 PASS twice; ten cold repetitions exact per
  process; memcheck/initcheck/synccheck zero errors; historical GPU 11/11 PASS;
  NPR1-B expected failure; FCR0 PASS.
- **Remaining risk:** local matrix, neighborhood, solver and trajectory remain
  outside NCGA0 even if it passes.
- **Promotion needed:** none; the track remains Proposed/report-only.
