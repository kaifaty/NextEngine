# Nonlocal nonlinear solver research -- current task state

| Field | Value |
|---|---|
| Status | `ACTIVE / NSR3B4DR0_NEW_ROOT_REQUIRED / B4DR1_DESIGN` |
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
- **Next action:** design B4DR1 before cloning/building upstream code. Keep
  engine-owned adapter/patch and manifests reviewable while upstream source,
  binaries and generated trajectories stay outside Git.
- **Do not run:** B4E nominal corpus, CUDA, runtime/schema, PhysX coupling,
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

## Performance facts retained

- B4C4BM candidate construction wins all `63/63` paired rounds per fixture;
  median process speedups are `1.1399x/2.5952x` P1/P2.
- B4C4CM candidate neighborhood+evaluation+tape construction wins all `63/63`
  rounds; medians are `1.2274x/1.2055x`.
- Six independent fixed-reference lanes used about `3.06x` wall parallelism;
  that validates harness utilization, not runtime solver throughput.
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

1. Freeze B4DR1 new external generator/profile contract.
2. Bind upstream commit, adapter source/patch, compiler/build flags, dependency
   closure, self-tests, output format and durable external artifact manifest.
3. Only then clone/build the external comparator and run tiny geometry gates
   before any long payload generation.
4. Keep B4E blocked until a new reference attestation passes twice.

## Reconsideration triggers

- Exact W0I files become available: rerun B4D twice; do not redesign first.
- Original adapter/diff/binary becomes available: verify recorded hashes, then
  regenerate outside Git and require all three historical payload hashes.
- Pairwise Descent paper/code becomes public: compare only after its exact
  formula and identity are reviewable; it does not bypass B4D references.
