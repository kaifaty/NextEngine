# Nonlocal formula reclosure — current task state

| Field | Value |
| --- | --- |
| Status | `ACTIVE / FCR0_PASS / FCR1_NEXT` |
| Updated | `2026-08-20` |
| Task key | `nonlocal-continuum-formula-reclosure` |
| Scope | Prove or reject a separately rooted energy/force-consistent Nonlocal continuum identity through algebra, physical, CUDA and performance gates |
| Definition of done | FCR0–FCR7 select a production-roadmap candidate or stop with an exact reproducible first failing boundary |
| Authority | Working context only; SPEC-38, ADR-076, ADR-081, `docs/roadmap.md` and the frozen stage contracts outrank this file |

## Resume in 60 seconds

- **Current conclusion:** `nuv-variational-fcr1` is an algebra candidate; solver and physical validity are still unknown.
- **Why:** FCR0 passes every energy/force derivative below `1.51e-8`, with byte-identical reports and unchanged old-line controls.
- **Next action:** Specify and execute FCR1 pair-enumeration and pressure-semantics discriminators.
- **Current blocker:** None.
- **Do not retry:** Repairing or retuning `nuv-basin-48k-static-support-h3-physical.v4`; its formula identity, coefficients and roots are closed historical evidence.
- **Reconsider when:** Only a reviewed upstream erratum can change source interpretation; it still cannot relabel old roots.

## Current evidence

| Evidence | Result | Consequence |
| --- | --- | --- |
| `docs/development/nonlocal-continuum-npr1b-term-controls-evidence-2026-08-20.md` | `FAIL` for the stopped source-shaped identity | Old NPR1-C through NPR8 remain blocked |
| `docs/development/nonlocal-continuum-formula-reclosure-research-2026-08-20.md` | `DECISION` | New identity and formula contract required |
| `docs/plans/nonlocal-continuum-formula-reclosure/00-formula-contract.md` | `VERIFIED` | FCR0 algebra passes; no physical or runtime claim |
| `docs/development/nonlocal-continuum-fcr0-algebra-evidence-2026-08-20.md` | `PASS` | `FCR_ALGEBRA_CANDIDATE`; FCR1 authorized |

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

## Open hypotheses

| Hypothesis | Evidence for | Evidence against | Next discriminator |
| --- | --- | --- | --- |
| H1: compression-only is the stable free-surface rule | SISPH text, both author code paths, standard negative-pressure clamp | printed Eq. 7 is two-sided | FCR1 underdense pair and free-surface patch |
| H2: one directed visit plus endpoint scatter is the clean coefficient implementation | Eq. 12/13 pair state and exact momentum closure | current neighbor graph likely stores both directions | FCR1 enumerated two-particle graph |
| H3: corrected identity can converge at product cadence | unified coupling and SISSM stability examples | no unconditional convergence; product `dt` is larger than paper examples | FCR2/FCR3 residual and accepted-step sweeps |

## Required context

Read these sources in precedence order before acting:

1. `docs/architecture/agent-routing.md`, SPEC-38, ADR-076 and ADR-081.
2. `docs/roadmap.md` R8 continuum row.
3. `docs/plans/nonlocal-continuum-formula-reclosure/README.md` and `00-formula-contract.md`.
4. `docs/development/nonlocal-continuum-formula-reclosure-research-2026-08-20.md`.
5. The stopped production roadmap and NPR1-B evidence for non-regression only.

## Next action

1. Freeze FCR1's exact two-particle directed/undirected graph cases and underdense free-surface patch.
2. Select one enumeration rule with analytical coefficient correspondence and one pressure rule with no tensile attraction.
3. Re-run FCR0 and both frozen old-line controls as non-regression.

## Do not retry

- Old v4 coefficient/profile tuning — formula identity is stopped; reconsider only for historical reproduction.
- Surface `m/r0` as an assumed paper repair — the physical-distance normalization was not frozen; reconsider only with new primary evidence.
- Product-scale or long corpus runs before tiny gates — they cannot distinguish algebra errors cheaply.

## Handoff

- **Workspace state:** branch `codex/nonlocal-continuum-n0`; FCR0 implementation committed at `0ddf579`; evidence/roadmap update is the current change.
- **Checks:** FCR0 PASS twice byte-identically; NPR1-A PASS unchanged; NPR1-B frozen FAIL unchanged; retained CUDA self-test PASS.
- **Remaining risk:** Pair enumeration, nonlinear convergence, coefficient calibration and corrected CUDA cost remain open.
- **Promotion needed:** Roadmap/evidence updates at each material gate; architecture promotion remains forbidden.
