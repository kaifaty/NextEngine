# Nonlocal nonlinear solver research -- current task state

| Field | Value |
|---|---|
| Status | `ACTIVE / NSR3B4E1S_PASS / B4E1M_IMPLEMENTATION` |
| Updated | `2026-08-21` |
| Task key | `nonlocal-nonlinear-solver-research` |
| Scope | Fundamental solver research over the verified Nonlocal variational objective, isolated from runtime and the stopped SISSM lineage |
| Definition of done | NSR0--NSR6 select a production-roadmap candidate or stop at an exact reproducible boundary |
| Authority | Working context only; Accepted architecture, SPEC-38/ADR-076/ADR-081 and frozen stage contracts outrank this file |

## Resume in 60 seconds

- **Current conclusion:** B4C4 packaging is complete. The B4C4C1 packaged
  runner still reproduces identity `66e318cb...d6f3c` and semantic result
  `b4d52600...550c`; formula/profile/source hashes are unchanged.
- **Current conclusion:** B4D executes twice deterministically at stdout SHA
  `fbeb4040...ddd9` and fails closed at
  `CW-HYDRO-001:MISSING_ARTIFACT`. All three required external `CWREFV1`
  files are absent; `trajectory_started=false` and B4E is not authorized.
- **Current conclusion:** B4DR0 finds neither exact payloads nor the recorded
  adapter diff/source/binary. Matching GCC and prose are insufficient to
  reproduce the historical bytes.
- **Current decision:** select `NEW_REFERENCE_PROFILE_REQUIRED`. Preserve B4D
  FAIL and historical W0I/W1, then freeze a new reproducible external
  generator/profile; never synthesize old hashes or inherit W1 credit.
- **Current conclusion:** B4DR1 R1A passes after explicit provenance
  correction. One originally retained source copy lacked a complete Git
  object database; a verified true-full-clone build reproduces all eight
  static artifacts, and the historical R1C1 build reproduces executable/report
  roots exactly. The incomplete-clone fact remains recorded.
- **Current constraint:** upstream configure writes `Utilities/Version.h` into
  the source tree, and its revision probe rejects linked Git worktrees. Every
  profile build starts from an ordinary clean full clone and binds the
  generated header separately.
- **Current decision:** R1B freezes an independent standalone contact tool,
  separate validator, six parent vectors and internal-face/one-ulp restart
  sentinels. Process preflight must fail before contact on float/OMP/locale/ABI
  mismatch.
- **Current correction:** R1B v2 separates the unit outer-clamp fixture from
  the `[0,2] x [0,1] x [0,1]` orifice box. Rejected v1 placed wall `x=1` on
  the outer face and never reached implementation.
- **Current conclusion:** R1B passes. Two builds reproduce executable SHA
  `c1150fad...a5da`, two processes reproduce stdout SHA `c6a4950d...a8a`, all
  eight contact cases pass, and float/locale/OpenMP mutations reject before
  contact. No trajectory ran.
- **Negative result:** first R1C manifest-only run stopped at
  `CW-DAM-001:FLUID_ROOT`, with `simulation_created=false` and
  `trajectory_started=false`. The shortened ID conflicts with roots computed
  from normative `CW-DAMBREAK-001`; this is not a physics failure.
- **Current decision:** reject R1C identity `a061f43e...99d0` and select the
  narrow R1C1 manifest-identity reclosure `865570e1...8927`.
- **Current conclusion:** R1C1 passes. Two builds reproduce executable SHA
  `c8933e01...b6ca`; two processes reproduce report SHA `6d293328...8f3`;
  all roots/mutations pass and forced mismatch rejects before Simulation.
- **Negative result:** the first authorized Hydro trajectory exits at
  `PRESSURE_NOT_CONVERGED`; report root `1d3f7c4e...67e`, no stderr and zero
  output entries. No repeat, Dam or Orifice ran. R1C and R1D are blocked.
- **Current decision:** freeze R1C2 failure observability only. Preserve all
  physics/profile bytes and run one diagnostic Hydro process exposing the
  failing step, phase, iteration/residual/convergence and time-step bits.
- **Current conclusion:** R1C2 shows step 1 pressure reaches cap 100 with
  residual `0.8274405823` versus threshold `0.1`; divergence converges in one
  iteration with zero residual and timestep bits remain exact.
- **Current decision:** select an ascending one-step R1C3 pressure-cap sweep
  over `25,50,75,100,125,150,200,300`, changing no other profile value.
- **Current conclusion:** R1C3 residual decreases monotonically; cap 300
  first permits convergence at iteration 220 with residual `0.0992042`.
- **Calibration conclusion:** `spacing^3` yields infinite-lattice density
  ratio `0.999972`; upstream's 0.8 startup heuristic yields `0.799978` and
  changes mass by 20%, so it is not selected for this comparator.
- **Current decision:** reclose R1C4 with pressure cap 300 only, new profile
  identity and otherwise byte-identical physics/serialization.
- **Current conclusion:** R1C4 Hydro and Dam pairs pass byte-identically. The
  first Orifice step converges, then contact rejects because analytical
  `x_max` was wrongly derived as 1 m from source-support `boundary_nx=20`.
- **Current decision:** R1C4 fails overall. R1C5 separates domain extent from
  boundary lattice extent, keeps Orifice support unchanged and uses a new
  global profile identity.
- **Current conclusion:** R1C5 passes all three pairs byte-identically. All six
  processes exit zero with empty stderr; Orifice completes with 28 receiver
  samples under analytical `x_max=2.0` and unchanged one-metre source support.
- **Current decision:** R1D uses new schedule-consistent scenario manifests,
  unchanged frame format/physics and separate q99-x/q99-y/receiver roots.
  Independent one-thread scenarios run in two waves of at most three.
- **Current conclusion:** R1D passes. All full scenario reports/payloads are
  pairwise byte-exact and published as verified regular content-addressed
  files. Orifice ends with 1,172 receiver samples.
- **Performance fact:** two three-process waves reduce harness wall by about
  `1.58x`; Hydro nevertheless takes 7:43 per 1,200-step process, exposing a
  real late-state DFSPH reference cost rather than I/O or memory starvation.
- **Current decision:** R1E uses a separate no-SPlisHSPlasH C++17 reader with
  descriptor-safe path admission, full independent parse, canonical decoded
  root, regenerated aggregates and serialized/decoded mutation controls.
- **Current conclusion:** R1E passes. Two builds reproduce reader SHA
  `8c4e7d61...55ea`; two positive processes reproduce report SHA
  `60e5575b...630e`; all semantic/aggregate/mutation gates pass and four
  external negative fixtures reject deterministically.
- **Current decision:** select `NEW_EXTERNAL_DFSPH_REFERENCE_CANDIDATE` and
  authorize only B4E nominal-corpus research and contract design.
- **Current conclusion:** B4E research finds exact Hydro/Dam initial geometry,
  constants and macro-step alignment, but the packaged entry point still owns
  only tiny P1/P2 fixtures. Per-particle and solver-iteration comparisons are
  rejected; canonical q99/COM/curve aggregates are selected.
- **Current decision:** use a B4E0 zero-trajectory alignment gate, then a
  one-macro cost probe and first-output pilots before any full run. Orifice is
  deferred to B4O; a projected combined cost above four machine-hours routes
  to B4EP optimization without becoming a physics failure.
- **Current conclusion:** B4E0 passes twice and across two identical builds.
  Hydro/Dam need at most 118/117 neighbors, about 47 MiB RSS and 0.4 s for the
  zero-step preflight; all roots and mutations are exact.
- **Numerical diagnostic:** Hydro has nine active centres at only
  `6.6613381477509392e-16` positive strain. This is machine-floor branch
  sensitivity, not pressure evidence; B4E1 must show its spectral cost.
- **Current decision:** split the resource probe. Because epsilon-active Hydro
  forces the 48-HVP spectrum, B4E1S measures that path before any KKT work.
  The derived initial count must be at most 96 so an adjacent fine level can
  remain within the selected 192-substep cap.
- **Current conclusion:** B4E1S passes across two independent builds and two
  fresh processes. Four bit-exact 48-HVP estimates give maximum eigenfrequency
  `499.43728723929792 s^-1` and 14 initial substeps, with zero all-pairs work.
- **Performance fact:** a process containing two complete B4E1S estimates
  takes 0.51 s wall and about 64 MiB RSS at 99% CPU. This is spectrum preflight
  cost, not macro-step or production throughput.
- **Current decision:** B4E1M reuses the complete retained-flat adaptive
  transaction at exact nominal Hydro. It may attempt only `14,28,56,112`,
  commits one selected fine step, and runs once per fresh process under an
  external 900-second watchdog.
- **Next action:** implement `--nominal-hydro-macro-probe`, then run one macro
  in each of two independent builds. Keep time/RSS external, reference closed
  and step 2 forbidden.
- **Do not run:** unfrozen B4E corpus, CUDA, runtime/schema, PhysX coupling,
  persistence or production work.

## Current selected lineage

| Boundary | Selected result | Allowed claim |
|---|---|---|
| NSR0--NSR2C2 | analytic full HVP, safeguarded trust-region Newton-CG and numerical-floor stop | full-curvature tiny/scaling solver candidate |
| NSR3A/A1/A2 | serial baseline, allocation-free HVP workspace and outer-state Hessian tape | exact CPU research implementation and local cost evidence |
| NSR3B0R | `nuv-variational-fcr2` lattice-normalized cubic objective | formula/density/gradient/HVP correspondence |
| NSR3B1S3/R1 | embedded controller with selected-fine ownership | bounded report-only temporal controller |
| NSR3B2--B3R | split static support/contact plus owned-residual globalization | tiny static-boundary composition candidate |
| NSR3B4B2 | feasible contact-onset pressure forecast | tiny P1/P2 pressure/contact candidate |
| NSR3B4C3MC1 | balanced macro publication and tiny physical accuracy | P1/P2-only adaptive macro candidate; no temporal-equivalence claim |
| NSR3B4C4A/B/C | retained workspace, immutable support index and flat CSR ownership | packaged complete adaptive/fixed research runner |
| NSR3B4D | fail-closed external reference reader | deterministic missing-artifact boundary only |
| NSR3B4DR1A | reproducible strict external DFSPH library bootstrap | build/toolchain candidate only; no adapter or trajectory |
| NSR3B4DR1B | reproducible standalone contact/validation adapter | contact algebra and ABI gate only; R1C design authorized |
| NSR3B4DR1C contract | frozen manifests, source patch and short-trajectory format | manifest-only implementation; trajectory conditional on preflight PASS |
| NSR3B4DR1C1 | corrected normative dam scenario identity | repeat manifest-only gate; no solver object yet |
| NSR3B4DR1C1 PASS | reproducible zero-physics manifest preflight | R1C trajectory implementation only; no full schedules |
| NSR3B4DR1C trajectory | first Hydro rejects at pressure convergence with no payload | R1C/R1D blocked; do not advance scenarios |
| NSR3B4DR1C2 contract | observability-only diagnostic reclosure | one Hydro diagnostic; no solver tuning or R1C credit |
| NSR3B4DR1C2 PASS | step-1 cap hit isolated to pressure; divergence/dt exact | R1C3 cap-sweep design only |
| NSR3B4DR1C3 contract | fixed one-step pressure-cap sweep | diagnose bounded convergence; no payload or R1C credit |
| NSR3B4DR1C3 PASS | monotone residual; first convergence at iteration 220 | R1C4 cap-300 profile reclosure only |
| NSR3B4DR1C4 contract | cap 300 and new trajectory identity | paired short scenarios; R1D still blocked |
| NSR3B4DR1C4 execution | Hydro/Dam exact; Orifice fails before contact at extent mismatch | R1C4 FAIL; no R1D |
| NSR3B4DR1C5 contract | explicit domain/support extent ownership | rerun all pairs under new identity |
| NSR3B4DR1C5 PASS | all short pairs exact; Orifice reaches 28 receiver samples | R1D full generation only |
| NSR3B4DR1D contract | full schedule manifests, aggregate roots and verified publication | implement preflight/generator, then run full pairs |
| NSR3B4DR1D PASS | three full pairs exact and content-addressed | R1E contract design only |
| NSR3B4DR1E contract | actual reference closure plus independent fail-closed reader | implement/attest only; no B4E execution |
| NSR3B4DR1E PASS | independently decoded full DFSPH references and deterministic negative controls | new reference candidate; B4E contract design only |
| NSR3B4E research | staged Hydro/Dam aggregate comparison and cost ladder | B4E0 alignment preflight only; Orifice remains B4O |
| NSR3B4E0 contract | exact zero-trajectory nominal alignment | implement preflight only; B4E1 still blocked |
| NSR3B4E0 PASS | exact 6k Hydro/Dam inputs and capacity-valid flat neighborhoods | B4E1 one-macro contract design only |
| NSR3B4E1S contract | isolate nominal Hydro 48-HVP spectrum and temporal capacity | implement/execute spectrum only; no KKT or trajectory |
| NSR3B4E1S PASS | deterministic 48-HVP spectrum; 14 initial substeps | B4E1M one-macro contract design only |
| NSR3B4E1M contract | one nominal retained-flat Hydro transaction | implement/execute step 1 only; reference remains closed |

Candidate solver identity remains:

```text
formula  = nuv-variational-fcr2+split-static-boundary-r0
solver   = nuv-newton-krylov-r0+outer-state-hessian-tape-v1
publish  = balanced macro-boundary canonical publication
package  = complete-lane flat-adjacency candidate
```

No current runtime owner, public schema, save format, gameplay mutation or
production authority is created by this lineage.

## Evidence that still constrains work

| Evidence | Result | Instruction |
|---|---|---|
| [FCR3-B2](../nonlocal-continuum-fcr3b2-chebyshev-evidence-2026-08-20.md) | fixed Chebyshev directions become non-descent | never retune/relabel stopped SISSM lineage |
| [B1D1](../nonlocal-nsr3b1d1-floor-oracle-evidence-2026-08-20.md) | temporal stiffness is real below acoustic Courant about one | retain adaptive temporal policy |
| [B4C3TR](../nonlocal-nsr3b4c3tr-fixed-reference-evidence-2026-08-21.md) | per-substep micrometre publication destroys refinement reference | do not restore per-private-substep publication |
| [B4C3P](../nonlocal-nsr3b4c3p-publication-cadence-evidence-2026-08-21.md) | macro publication restores order but old scalar tube fails | retain mixed physical/temporal budget, not coefficient 32 widening |
| [B4C3MC1](../nonlocal-nsr3b4c3mc1-adaptive-accuracy-budget-evidence-2026-08-21.md) | unchanged B4B physical budgets pass on P1/P2 | expand diversity before physical production claims |
| [B4C4C1](../nonlocal-nsr3b4c4c1-complete-flat-adjacency-evidence-2026-08-21.md) | all eight lanes/rollback exact; zero final ownership | B4C4 is closed; keep legacy path as oracle/rollback |
| [B4D](../nonlocal-nsr3b4d-reference-reattestation-evidence-2026-08-21.md) | local identities exact, all external files missing | no nominal trajectory until reference closure is restored |
| [B4DR1A](../nonlocal-nsr3b4dr1a-external-bootstrap-evidence-2026-08-21.md) | strict closure reproduces 8/8 artifacts from a verified full clone after [provenance correction](../nonlocal-nsr3b4dr1a-full-clone-provenance-correction-evidence-2026-08-21.md) | require complete-object `fsck` before configure; never use linked upstream worktrees |
| [B4DR1B](../nonlocal-nsr3b4dr1b-contact-adapter-evidence-2026-08-21.md) | adapter binary/output reproduce and all contact/preflight gates pass | freeze R1C manifests before the first DFSPH step; do not inherit W1 credit |
| [B4DR1C research](../nonlocal-nsr3b4dr1c-trajectory-preflight-research-2026-08-21.md) | warm starts and hidden convergence diagnostics violate the intended profile | use only the tracked equation-preserving patch; pass manifest preflight first |
| [B4DR1C negative](../nonlocal-nsr3b4dr1c-manifest-preflight-negative-evidence-2026-08-21.md) | shortened dam ID contradicts frozen fluid/boundary roots; stopped before Simulation | preserve rejection; use only R1C1 normative ID reclosure |
| [B4DR1C1](../nonlocal-nsr3b4dr1c1-manifest-preflight-evidence-2026-08-21.md) | two builds/reports exact; roots and negative mismatch gate pass without Simulation | apply frozen patch in a fresh clone and implement short trajectory only |
| [B4DR1C trajectory](../nonlocal-nsr3b4dr1c-trajectory-negative-evidence-2026-08-21.md) | first Hydro fails at pressure convergence; current report hides solver fields | preserve FAIL; instrument only R1C2 observability before any tuning |
| [B4DR1C2](../nonlocal-nsr3b4dr1c2-failure-observability-evidence-2026-08-21.md) | pressure hits 100 iterations at `8.2744x` threshold on step 1; divergence and dt exact | sweep pressure cap before changing calibration/tolerance |
| [B4DR1C3](../nonlocal-nsr3b4dr1c3-pressure-cap-evidence-2026-08-21.md) | residual falls monotonically and crosses threshold at iteration 220; physical volume is lattice-normalized | reclose cap 300 only; retain mass/volume and cold policy |
| [B4DR1C4](../nonlocal-nsr3b4dr1c4-trajectory-evidence-2026-08-21.md) | Hydro/Dam pairs exact; Orifice domain extent conflated with source support | preserve partial evidence but grant no pass; separate ownership in R1C5 |
| [B4DR1C5](../nonlocal-nsr3b4dr1c5-trajectory-evidence-2026-08-21.md) | all three pairs byte-exact; corrected Orifice crosses into receiver | execute R1D full schedules; retain all earlier negative evidence |
| [B4DR1D](../nonlocal-nsr3b4dr1d-full-generation-evidence-2026-08-21.md) | all full pairs exact; verified external publication | freeze R1E reader/profile contract over actual roots |
| [B4DR1E](../nonlocal-nsr3b4dr1e-reference-attestation-evidence-2026-08-21.md) | independent reader accepts all full references; both mutation layers and four external negatives reject | design B4E against the new candidate; no execution before a frozen comparison contract |
| [B4E research](../nonlocal-nsr3b4e-nominal-corpus-research-2026-08-21.md) | Hydro/Dam align at input/step level; nominal entry point and cost evidence are missing | execute B4E0 alignment before any trajectory |
| [B4E0](../nonlocal-nsr3b4e0-nominal-alignment-evidence-2026-08-21.md) | exact roots/aggregates/mutations; nominal degrees 118/117; no trajectory | design one-macro Hydro resource probe only |
| [B4E1S research](../nonlocal-nsr3b4e1s-spectrum-research-2026-08-21.md) | epsilon-active parent forces spectrum before KKT; initial count must be <=96 | execute isolated spectrum before one-macro design |
| [B4E1S](../nonlocal-nsr3b4e1s-hydro-spectrum-evidence-2026-08-21.md) | four bit-exact estimates give 14 initial substeps and zero all-pairs work | design one Hydro macro transaction only |
| [B4E1M research](../nonlocal-nsr3b4e1m-hydro-macro-research-2026-08-21.md) | complete transaction can isolate levels `14,28,56,112`, fine commit and cost | execute one step-1 macro per fresh process |

Detailed stage order, every intermediate negative and all evidence links remain
in the [research roadmap](../../plans/nonlocal-nonlinear-solver-research/README.md).
Git history before the B4D checkpoint retains the superseded long-form task
diary; it is not current authority.

## Current exact external boundary

Frozen W0I attestation root:
`186e1e31c0aa2636525bbc54e4fe4335b8432e7e99eddf3221d08e0380b65c90`.

| File | Required SHA-256 | Current state |
|---|---|---|
| hydro | `84ae867f5b336cd0bd51be6f29a6a2a1f27f702c424f1dbd0a8f735b9f4bb435` | missing |
| dam break | `853d965489a40082a024aeee5a19f98aef054417014af8556fa212687d88d12c` | missing |
| orifice | `60e9b3538d621ef1a3f1ae77569640740df471fbe4e5ef8eaa813d3751930849` | missing |

B4D identity:
`47c78bdb115c0e5d7ed7132a6de3e62e3ba9533396a7f346b510b354dc5222fe`.

The external payloads are deliberately outside Git. Local searches found no
copy in `/tmp`, NextEngine worktrees, Downloads, desktop/trash or exact-size
Git blobs. Exact-hash web search returned no result and is not proof of global
absence. Unreachable Git blobs contain neither a full payload nor the recorded
adapter/source/binary SHA-256 values.

The separate new-root R1E reference candidate is available under profile
`ba34b4e3b12986ebc831320d6811551d5311a6774a64f079aabe3a5eaa6bb746`
and is attested by identity
`9cf5fc571fee7bc0be5585d9b467f90cd8a27f40d9cf39d6be999b0c466cbccc`.
It does not replace the missing historical W0I bytes or inherit their credit.

## Decisions

### D-001 -- Preserve the new nonlinear lineage

- **Observation:** full coupled curvature plus trust-region globalization
  closes the stopped local-SISSM failure on bounded controls.
- **Decision:** retain `nuv-newton-krylov-r0`; Pairwise Descent remains
  monitor-only until public formulas/code exist.
- **Rejected:** Chebyshev radius sweeps, larger caps and silent fallback.

### D-002 -- Keep normalized corrected physics

- **Observation:** the raw FCR cubic integrates to `1/8`; the author's fixed
  lattice normalization restores the declared density.
- **Decision:** all current work uses `nuv-variational-fcr2`; no old source-
  shaped coefficient, root or performance result transfers.

### D-003 -- Publish only at macro boundaries

- **Observation:** canonical microunit noise scales with private substep count.
- **Decision:** private adaptive/fixed levels remain binary64 inside a macro
  transaction; only the accepted endpoint is balanced and published once.
- **Rejected:** epsilons, per-substep canonical continuation and fitted scalar
  amplification tubes.

### D-004 -- Close B4C4 packaging without a whole-solver claim

- **Observation:** retained workspaces, static support indexing and CSR
  ownership preserve all complete-lane roots and remove duplicate work.
- **Decision:** select the composed B4C4 candidate. Local construction timing
  improves `1.2274x/1.2055x` on P1/P2; this is not whole-solver performance.

### D-005 -- Preserve missing B4D evidence and recover reproducibility

- **Observation:** source/solver identities match, but the three `/tmp`
  payloads and their original external adapter artifacts are unavailable.
- **Decision:** keep B4D fail-closed and research artifact/generator recovery.
- **Rejected:** placeholder files, reconstructed aggregate curves, old
  non-clearance references, hash-manifest-only PASS and B4E without input.
- **Reconsider when:** exact W0I files or exact source/diff/binary lineage is
  restored; otherwise only a newly rooted reference profile may proceed.

### D-006 -- Select a strict complete-clone external build profile

- **Observation:** GCC/CMake/Ninja and all required pinned dependencies
  reproduce byte-identically with binary64, AVX/FMA/fast-math disabled from a
  verified complete clone. One originally retained source copy was not a
  complete object database and is preserved as negative provenance evidence.
- **Decision:** select B4DR1 R1A and require an ordinary clean full Git clone
  for every reference build; bind generated `Utilities/Version.h` separately.
- **Rejected:** lazy blob-by-blob checkout and linked Git worktrees.

### D-007 -- Select the independent contact adapter

- **Observation:** the v2 geometry, all eight branch cases and independent
  clearance/chord validator pass under a reproducible linked DFSPH ABI.
- **Decision:** select R1B only as the contact/preflight boundary and authorize
  R1C scenario-manifest design.
- **Rejected:** the v1 outer-box geometry, a weakened header-only ABI anchor,
  and starting trajectories before manifest/serialization closure.

### D-008 -- Freeze a cost-aware short external trajectory gate

- **Observation:** pinned upstream enables warm starts and does not expose the
  complete convergence state required for a fail-closed comparator.
- **Decision:** bind the tracked cold-start/diagnostic-only patch and exact
  R1C manifests/serialization. Require a zero-physics manifest preflight
  before any solver object or trajectory.
- **Rejected:** accepting default warm starts, inferring convergence from one
  iteration counter, running full schedules first, or modifying DFSPH
  equations.

### D-009 -- Reject the inconsistent shortened dam identity

- **Observation:** `CW-DAM-001` generates neither of the dam roots frozen by
  R1C; normative `CW-DAMBREAK-001` generates both exactly.
- **Decision:** preserve the failed R1C identity and reclose only the scenario
  ID and derived manifest root under R1C1.
- **Rejected:** changing the expected roots to fit the shortened ID, editing a
  frozen contract into an apparent PASS, or treating the manifest failure as
  DFSPH evidence.

### D-010 -- Select the corrected zero-physics manifest gate

- **Observation:** two independent builds/processes reproduce every corrected
  root and mutation; forced mismatch stops before Simulation creation.
- **Decision:** select R1C1 and authorize implementation of the already-frozen
  short trajectory path in a separately patched full clone.
- **Rejected:** mutating a clean R1A clone or jumping directly to R1D.

### D-011 -- Preserve the first physical failure and reclose observability

- **Observation:** the first Hydro process fails at pressure convergence and
  publishes no payload, while the adapter discards its populated diagnostic
  fields when constructing the failure report.
- **Decision:** R1C fails and R1D remains blocked. Select R1C2 as a report-only
  reclosure followed by exactly one Hydro diagnostic process.
- **Rejected:** a blind iteration/tolerance change, warm-start restoration,
  retry under the failed identity, or advancing to Dam/Orifice.

### D-012 -- Measure the pressure convergence boundary before remediation

- **Observation:** R1C2 isolates a finite pressure-only cap hit at step 1 but
  provides only one convergence endpoint.
- **Decision:** run a fixed ascending one-step R1C3 cap sweep, stopping at the
  first converged result and changing no other profile field.
- **Rejected:** immediate cap promotion, tolerance loosening or simultaneous
  mass/volume/boundary changes.

### D-013 -- Reclose the short reference profile at cap 300

- **Observation:** cap 300 permits normal step-1 exit at iteration 220, while
  the selected physical volume gives an almost unit lattice density sum.
- **Decision:** create new-root R1C4 by changing only pressure maximum to 300;
  repeat paired short scenarios before R1D.
- **Rejected:** upstream 0.8 underdensity heuristic, warm starts, tolerance
  loosening and inheriting any payload/root from failed R1C.

### D-014 -- Separate analytical domain from boundary support

- **Observation:** Orifice intentionally has a two-metre analytical box but
  only one-metre source-side Akinci support; one `boundary_nx` field cannot own
  both meanings.
- **Decision:** add explicit scenario `domain_x_max`, retain boundary roots and
  reissue the global R1C5 payload identity before rerunning all pairs.
- **Rejected:** extending receiver-side Akinci support, weakening contact's
  two-metre assertion or inheriting R1C4 Hydro/Dam payload roots.

### D-015 -- Admit corrected short trajectories to full generation

- **Observation:** all six R1C5 processes pass with byte-identical paired
  reports/payloads; Orifice records 28 final receiver samples.
- **Decision:** authorize only R1D full external generation under the R1C5
  profile and cap 300.
- **Rejected:** inheriting short payloads as full references, starting R1E
  early or converting this research pass into runtime/production authority.

### D-016 -- Reclose manifests at the full schedule

- **Observation:** R1C blocks normatively claim 24 steps/every-step output and
  cannot be embedded unchanged in truthful full-schedule payloads.
- **Decision:** reissue only scenario schedule blocks and profile identity;
  retain all R1C5 physical bytes and the frame layout. Run independent
  scenarios concurrently, never parallelizing one solver process.
- **Rejected:** a contradictory embedded manifest, OpenMP reduction changes,
  `/tmp`-only output or publication before pair equality.

### D-017 -- Admit full references to attestation design

- **Observation:** all three full pairs and reports compare byte-for-byte;
  final content-addressed files rehash to their reported roots.
- **Decision:** select R1D PASS and authorize only R1E reader/profile contract
  design over the actual closure.
- **Rejected:** direct B4E use without a fail-closed reader, old W1 credit or a
  production claim from external-reference generation.

### D-018 -- Keep reader independent from generator

- **Observation:** generator parser reuse could reproduce a common layout bug
  while still matching complete-file hashes.
- **Decision:** implement a standalone reader that canonically reconstructs
  every decoded field and independently regenerates aggregate roots.
- **Rejected:** generator-source reuse, filename/report trust, path checks
  separated from open, or full hash without decoded semantic controls.

### D-019 -- Admit the new reference candidate to B4E design

- **Observation:** the independent reader reproduces all three decoded and
  aggregate roots twice; missing, symlink, oversized and complete-size mutated
  fixtures all fail closed.
- **Decision:** select `NEW_EXTERNAL_DFSPH_REFERENCE_CANDIDATE` and begin only
  B4E nominal-corpus research/contract design.
- **Rejected:** inheriting historical W1 credit, executing an unfrozen
  comparison, or treating reference integrity as runtime/production evidence.

### D-020 -- Stage nominal comparison behind alignment and cost gates

- **Observation:** the packaged path has nominal capacities and avoids
  all-pairs candidate HVPs, but only P1/P2 commands have executed; its active
  macro frame still requires spectral and nonlinear HVP work.
- **Decision:** run zero-trajectory B4E0, one-macro B4E1 and first-output B4E2
  before projecting or starting the full Hydro/Dam pair.
- **Rejected:** full-run-first execution, cross-solver iteration/density or
  per-particle gates, Orifice before B4O, and timing as a physics tolerance.

### D-021 -- Admit nominal alignment to one-macro cost design

- **Observation:** both 6k cases reproduce exact R1D roots and remain below
  flat-neighborhood capacities; two processes/builds agree exactly.
- **Decision:** select B4E0 PASS and design one Hydro macro step with timing
  outside deterministic physics/work evidence.
- **Rejected:** treating the 0.4 s zero-step preflight as solver throughput or
  interpreting machine-floor Hydro active centres as physical pressure.

### D-022 -- Isolate spectral cost before one macro

- **Observation:** any Hydro macro attempt first pays 48 HVPs solely because
  nine parent centres are epsilon-positive; a whole step cannot attribute
  that cost or preflight the 192-substep policy capacity.
- **Decision:** execute B4E1S spectrum twice and require derived initial count
  at most 96 before B4E1M design.
- **Rejected:** timing the whole macro first or clipping the active set under
  the already frozen solver identity.

### D-023 -- Admit the nominal spectrum to one-macro design

- **Observation:** four independent same-state estimates agree bit-for-bit,
  use exactly 48 joint HVPs each and derive 14 initial substeps versus the
  frozen capacity boundary of 96.
- **Decision:** select `NOMINAL_HYDRO_SPECTRUM_CANDIDATE` and design B4E1M as
  exactly one Hydro macro transaction with timing kept external.
- **Rejected:** interpreting the spectrum probe as macro throughput, opening
  the reference curve early or advancing directly to a multi-step run.

### D-024 -- Bound the first nominal nonlinear transaction

- **Observation:** the existing adaptive path must test adjacent temporal
  levels, so B4E1M can cost more than the 14-substep spectrum forecast alone.
- **Decision:** run exactly one retained-flat step-1 transaction per process
  over levels `14,28,56,112`, with a non-physical 900-second watchdog.
- **Rejected:** two in-process macros, a forced-failure duplicate, opening the
  first reference output early or treating a watchdog exit as physics FAIL.

## Performance facts retained

- B4C4BM candidate construction wins all `63/63` paired rounds per fixture;
  median process speedups are `1.1399x/2.5952x` P1/P2.
- B4C4CM candidate neighborhood+evaluation+tape construction wins all `63/63`
  rounds; medians are `1.2274x/1.2055x`.
- Six independent fixed-reference lanes used about `3.06x` wall parallelism;
  that validates harness utilization, not runtime solver throughput.
- A B4E1S process containing two complete nominal 48-HVP estimates takes
  0.51 s wall and about 64 MiB RSS at 99% CPU; no KKT solve runs.
- No nominal, 50k, GPU or production performance result exists for this
  corrected Nonlocal lineage.

## Required context

1. `docs/architecture/agent-routing.md`, SPEC-38, ADR-076 and ADR-081.
2. `docs/plans/nonlocal-continuum-formula-reclosure/README.md` and
   `00-formula-contract.md`.
3. Stopped formula-reclosure task state and FCR3-B2 evidence.
4. `docs/development/nonlocal-nonlinear-solver-research-2026-08-20.md`.
5. `docs/plans/nonlocal-nonlinear-solver-research/README.md` and current frozen
   stage contract.
6. W0I reference contract and B4D design/evidence.

## Exact next action

1. Implement `--nominal-hydro-macro-probe` over the frozen B4E1M transaction.
2. Build independently twice, then run one step-1 macro per process under the
   external watchdog and require byte-identical deterministic reports.
3. Preserve any failure without tuning; authorize B4E2 design only after
   physical, publication, ownership and repeatability gates all pass.

## Reconsideration triggers

- Exact W0I files become available: rerun B4D twice; do not redesign first.
- Original adapter/diff/binary becomes available: verify recorded hashes, then
  regenerate outside Git and require all three historical payload hashes.
- Pairwise Descent paper/code becomes public: compare only after its exact
  formula and identity are reviewable; it does not bypass B4D references.
