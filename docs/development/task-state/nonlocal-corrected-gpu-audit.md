# Nonlocal corrected GPU audit — current task state

| Field | Value |
| --- | --- |
| Status | `COMPLETE / NCGA0_REV2_GO / SUPPORTED_BOUNDED` |
| Updated | `2026-08-29` |
| Task key | `nonlocal-corrected-gpu-audit` |
| Scope | Determine whether corrected Nonlocal scalar/pair terms correspond to a separate strict-f32 CUDA implementation before any neighborhood or solver port |
| Definition of done | NCGA0 receives one bounded reviewed result or stops at its first reproducible failing boundary |
| Authority | Working context only; SPEC-38, ADR-076/081, the FCR1 formula contract and frozen evidence outrank this file |

## Resume in 60 seconds

- **Current conclusion:** NCGA0 revision 2 is independently reviewed
  `GO / SUPPORTED_BOUNDED`: corrected scalar/full-pair terms correspond between
  host `long double` and strict CUDA `f32` on the frozen profile.
- **Why:** all nine values, two energy derivatives, five mandatory negatives,
  cold repeatability, sanitizers and old controls passed; the re-review found no
  load-bearing defect.
- **Next action:** if GPU research continues, freeze a new neighborhood and
  local energy/source/matrix assembly contract before implementation.
- **Current blocker:** none.
- **Do not retry:** another long source-shaped 50k run; it cannot distinguish
  shared source mathematics from GPU translation.
- **Reconsider when:** the term implementation/profile/toolchain changes or a
  separately frozen neighborhood/linearization consumer needs this boundary.

## Current evidence

| Evidence | Result | Consequence |
| --- | --- | --- |
| `docs/development/nonlocal-continuum-nr1-rc1-evidence-2026-08-19.md` | `NR1_RECLOSED_GATHER_DIRECTED / REPORT_ONLY` | historical atomic ordering bug is real and the retained gather removes it on the frozen host |
| `docs/development/nonlocal-continuum-npr1b-term-controls-evidence-2026-08-20.md` | `FORMULA_MISMATCH` | source CPU/GPU equality cannot establish energy consistency |
| `docs/plans/nonlocal-corrected-gpu-audit/00-corrected-term-correspondence-contract.md` | `FROZEN` | only nine term fixtures are authorized in NCGA0 |
| `docs/development/nonlocal-corrected-gpu-term-audit-evidence-2026-08-29.md` | `AUTHOR_PASS / REVIEW_NO_GO` | execution is reproducible, but revision-1 semantics did not close |
| `docs/development/nonlocal-corrected-gpu-term-audit-independent-review-2026-08-29.md` | `NO-GO / REFUTED_AS_WRITTEN` | revision-1 agreement shared an untested viscosity observable |
| `docs/plans/nonlocal-corrected-gpu-audit/01-full-pair-force-correspondence-contract.md` | `FROZEN / REVIEWED_GO` | full-pair force and independent energy derivative are explicit |
| `docs/development/nonlocal-corrected-gpu-term-audit-repair-evidence-2026-08-29.md` | `GO / SUPPORTED_BOUNDED` | energy derivative selects full force and rejects the half identity |
| `docs/development/nonlocal-corrected-gpu-term-audit-independent-rereview-2026-08-29.md` | `GO / SUPPORTED_BOUNDED` | revision 2 closes the local term boundary |

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
- **Uncertainty:** later neighborhood and solver composition only; local strict
  binary32 correspondence is closed.
- **Reconsider when:** the selected profile, arithmetic mode or term code changes.

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
- **Consequences:** revision 1 remains NO-GO; revision 2 is reviewed GO and no
  further NCGA0 repair cycle remains.
- **Uncertainty:** no remaining local-term defect was observed; composition is
  outside this decision.
- **Reconsider when:** the energy/force observable or implementation changes.

## Open hypotheses

| Hypothesis | Evidence for | Evidence against | Next discriminator |
| --- | --- | --- | --- |
| H1 corrected full-pair terms translate to CUDA | reviewed 9/9 values, energy derivatives and negatives pass | no contrary local-term evidence | closed within NCGA0 |
| H2 GPU-specific coefficient/sign defect remains | prior atomics defect shows later GPU-specific errors are possible | no local term defect remains in NCGA0 | separately frozen neighborhood/matrix audit |
| H3 shared source mathematics caused the main historical failure | old CPU/GPU correspond; old derivative fails; corrected reviewed terms pass | does not exclude later composition defects | independent neighborhood/matrix oracle |

## Required context

1. `docs/architecture/agent-routing.md`, SPEC-38, ADR-076 and ADR-081.
2. `docs/roadmap.md` continuum R8 row and `docs/plans/continuum-water/README.md`.
3. `docs/plans/nonlocal-continuum-formula-reclosure/00-formula-contract.md`.
4. NPR1-B and NR1-RC1 evidence.
5. Both NCGA0 contracts and the independent revision-1/revision-2 reviews.

## Next action

1. Preserve NCGA0 sources, contracts and reviewed roots as the local boundary.
2. Draft a new contract for tiny fixed-set neighborhood plus local
   energy/source/matrix assembly.
3. Do not start full-solver, trajectory or performance work before that new
   boundary closes.

## Do not retry

- Historical atomic surface timing — correctness already failed and gather
  reclosed the bounded issue.
- Old source-shaped 50k performance as product evidence — NPR1-B invalidates
  its physics credit.
- Full corrected solver port before a neighborhood/matrix audit — composition
  failure would make it wasted work.

## Handoff

- **Workspace state:** primary checkout, branch `codex/water-research`.
- **Checks:** revision 2 PASS twice under author and reviewer builds; ten cold
  repetitions exact; five negatives reject; independent 70-digit derivative
  agrees; sanitizers zero errors; old controls unchanged.
- **Remaining risk:** local matrix, neighborhood, solver and trajectory remain
  outside the reviewed NCGA0 claim.
- **Promotion needed:** none; the track remains Proposed/report-only.
