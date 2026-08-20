# Nonlocal formula reclosure — current task state

| Field | Value |
| --- | --- |
| Status | `STOPPED / FORMULA_RECLOSURE_STOP / FAST_SOLVER_GATE_FAILED` |
| Updated | `2026-08-20` |
| Task key | `nonlocal-continuum-formula-reclosure` |
| Scope | Prove or reject a separately rooted energy/force-consistent Nonlocal continuum identity through algebra, physical, CUDA and performance gates |
| Definition of done | FCR0–FCR7 select a production-roadmap candidate or stop with an exact reproducible first failing boundary |
| Authority | Working context only; SPEC-38, ADR-076, ADR-081, `docs/roadmap.md` and the frozen stage contracts outrank this file |

## Resume in 60 seconds

- **Current conclusion:** The corrected energy has a slow bounded f64 oracle, but no admissible fast solver in this lineage.
- **Why:** v1 stalls on pressure quality; the sole frozen Chebyshev remediation creates non-descent directions in every pressure-bearing case.
- **Next action:** None in this roadmap. Reopen only with a separately specified nonlinear method and primary formula/code evidence.
- **Current blocker:** Terminal roadmap gate; no in-scope fast-solver remediation remains.
- **Do not retry:** Repairing or retuning `nuv-basin-48k-static-support-h3-physical.v4`; its formula identity, coefficients and roots are closed historical evidence.
- **Reconsider when:** A new nonlinear solver has complete primary formulas or reproducible code; it still cannot relabel old roots.

## Current evidence

| Evidence | Result | Consequence |
| --- | --- | --- |
| `docs/development/nonlocal-continuum-npr1b-term-controls-evidence-2026-08-20.md` | `FAIL` for the stopped source-shaped identity | Old NPR1-C through NPR8 remain blocked |
| `docs/development/nonlocal-continuum-formula-reclosure-research-2026-08-20.md` | `DECISION` | New identity and formula contract required |
| `docs/plans/nonlocal-continuum-formula-reclosure/00-formula-contract.md` | `VERIFIED` | FCR0 algebra passes; no physical or runtime claim |
| `docs/development/nonlocal-continuum-fcr0-algebra-evidence-2026-08-20.md` | `PASS` | `FCR_ALGEBRA_CANDIDATE`; FCR1 authorized |
| `docs/development/nonlocal-continuum-fcr1-pair-pressure-evidence-2026-08-20.md` | `PASS` | CPU pair/pressure semantics selected; FCR2 authorized |
| `docs/development/nonlocal-continuum-fcr2-reference-evidence-2026-08-20.md` | `PASS / CONDITIONING_RISK` | Slow CPU objective authority selected; FCR3-A required before profile sweep |
| `docs/development/nonlocal-continuum-fcr3a-block-evidence-2026-08-20.md` | `FAIL` | Pure block v1 rejected; exactly one hybrid v2 remediation allowed |
| `docs/development/nonlocal-continuum-fcr3a-hybrid-evidence-2026-08-20.md` | `FAIL` | Conditioning branch closed; FCR3-B corrected SISSM is next |
| `docs/development/nonlocal-continuum-fcr3b-sissm-v1-evidence-2026-08-20.md` | `FAIL` | Isolated terms pass; one term-localized combined discriminator authorized |
| `docs/development/nonlocal-continuum-fcr3b1-term-local-evidence-2026-08-20.md` | `PASS / isolated-P` | One pressure-only Chebyshev remediation authorized |
| `docs/development/nonlocal-continuum-fcr3b2-chebyshev-evidence-2026-08-20.md` | `FAIL / non-descent` | `FORMULA_RECLOSURE_STOP`; FCR3-C through FCR7 blocked |

## Decisions that still constrain the work

### D-001 — New identity, no inherited profile

- **Observation:** The old candidate failed energy/force consistency.
- **Evidence:** NPR1-B raw report SHA `e8ed6950e33fb9388887c40e304c74b517761698c0c7accbaff9a99899bf2c3a`.
- **Decision:** Use `nuv-variational-fcr1`; restart coefficient, support, cadence, physics and performance closure.
- **Rejected alternatives:** Patching v4 or preserving its coefficient names under new arithmetic.
- **Consequences:** Old performance data is comparison-only.
- **Uncertainty:** Corrected solver convergence and cost are unknown.
- **Reconsider when:** Never for old roots; new evidence may supersede only the new identity.

### D-002 — Compression-only default

- **Observation:** SISPH and released Nonlocal code clamp `rho` to `rho0`; literal Eq. 7 does not print the clamp.
- **Evidence:** SISPH Algorithm 1 and Section 4.3; pinned Nonlocal source.
- **Decision:** Declare `max(rho/rho0-1,0)^2` as the operational potential and retain two-sided Eq. 7 as an FCR1 comparator.
- **Rejected alternatives:** Silently differentiating two-sided energy while executing clamped force.
- **Consequences:** Directional tests avoid the non-smooth threshold.
- **Uncertainty:** The bounded physical magnitude of tensile artifacts at product scale.
- **Reconsider when:** FCR1 evidence rejects compression-only behavior against analytic/DFSPH references.

### D-003 — Explicit surface units

- **Observation:** The paper omits the closed form and units of `C`; source stores a primitive in normalized `q`.
- **Evidence:** Eq. 14/15 and pinned source energy/force splines.
- **Decision:** Define `C(r)=r0*C_hat(r/r0)` and keep physical `gamma` distinct from source `strength`.
- **Rejected alternatives:** Assuming an `m/r0` correction or numerical `gamma==strength` without a units contract.
- **Consequences:** FCR0 checks `dC/dr=c`; later coefficients are rederived.
- **Uncertainty:** Calibration of `gamma` to macroscopic surface tension remains FCR3/FCR4 work.
- **Reconsider when:** Primary-source errata or a dimensional physical calibration requires a different explicit potential.

### D-004 — Independent optimizer before SISSM

- **Observation:** SISSM's split and overshoot control introduce solver-specific correctness questions beyond the declared energy.
- **Evidence:** Nonlocal limitations plus the SISPH and Projective Peridynamics convergence procedures in the research report.
- **Decision:** FCR2 uses deterministic Armijo gradient descent as a slow f64 variational reference; corrected SISSM correspondence moves to FCR3.
- **Rejected alternatives:** Treating a source-shaped or newly transcribed SISSM step as its own physical oracle.
- **Consequences:** FCR2 performance has no product meaning; FCR3 must compare SISSM objective/residual behavior against FCR2.
- **Uncertainty:** Whether corrected SISSM can match the reference efficiently at product cadence.
- **Reconsider when:** A stronger independently verified nonlinear reference replaces Armijo without weakening the gates.

### D-005 — Stop the current fast-SISSM lineage

- **Observation:** v1 is pressure-limited and the one frozen `rho=0.9` Chebyshev remediation produces non-descent directions after 2-7 iterations.
- **Evidence:** FCR3-B1/B2 raw hashes and the exact objective safeguard.
- **Decision:** Select `FORMULA_RECLOSURE_STOP`; retain FCR2 only as a slow research oracle.
- **Rejected alternatives:** spectral-radius sweep, silent v1 fallback, larger iteration cap, relaxed gradient gate and coefficient tuning.
- **Consequences:** FCR3-C through FCR7 and all integration/performance promotion are blocked.
- **Uncertainty:** A different globally convergent nonlinear optimizer may still make the corrected energy practical.
- **Reconsider when:** A separately reviewed solver contract is backed by complete primary formulas or reproducible code, preferably the announced pairwise-descent method after publication.

## Open hypotheses

| Hypothesis | Evidence for | Evidence against | Next discriminator |
| --- | --- | --- | --- |
| H1: compression-only is the stable free-surface rule | SISPH text, both author code paths, standard negative-pressure clamp | printed Eq. 7 is two-sided | FCR1 underdense pair and free-surface patch |
| H2: one directed visit plus endpoint scatter is the clean coefficient implementation | Eq. 12/13 pair state and exact momentum closure | current neighbor graph likely stores both directions | FCR1 enumerated two-particle graph |
| H3: corrected identity can converge at product cadence | unified coupling examples | v1 stalls and fixed SISPH Chebyshev becomes non-descent | rejected for this solver lineage; new optimizer required |

## Required context

Read these sources in precedence order before acting:

1. `docs/architecture/agent-routing.md`, SPEC-38, ADR-076 and ADR-081.
2. `docs/roadmap.md` R8 continuum row.
3. `docs/plans/nonlocal-continuum-formula-reclosure/README.md` and `00-formula-contract.md`.
4. `docs/development/nonlocal-continuum-formula-reclosure-research-2026-08-20.md`.
5. The stopped production roadmap and NPR1-B evidence for non-regression only.

## Next action

1. Preserve the exact negative evidence and slow FCR2 oracle.
2. Monitor the announced Semi-Implicit Pairwise Descent paper/code; do not infer its method from the title.
3. Start a new solver lineage only after a separate contract explicitly supersedes D-005.

## Do not retry

- Old v4 coefficient/profile tuning — formula identity is stopped; reconsider only for historical reproduction.
- Surface `m/r0` as an assumed paper repair — the physical-distance normalization was not frozen; reconsider only with new primary evidence.
- Product-scale or long corpus runs before tiny gates — they cannot distinguish algebra errors cheaply.
- Block v1, hybrid v2 or alternate block switch points — exact fixed-budget gradient failures close this branch.
- Chebyshev spectral-radius sweeps or larger iteration caps — the frozen remediation is non-descent and closes this lineage.

## Handoff

- **Workspace state:** branch `codex/nonlocal-continuum-n0`; FCR3-B2 implementation committed at `2b99afb`; final stop evidence/roadmap update is the current change.
- **Checks:** FCR3-B2 FAIL twice byte-identically at pressure non-descent; FCR0–FCR2 and frozen v1/B1 reports unchanged.
- **Remaining risk:** A production-capable nonlinear optimizer is unproven; profile, CUDA and cost are intentionally not evaluated.
- **Promotion needed:** None. Architecture/runtime promotion remains forbidden.
