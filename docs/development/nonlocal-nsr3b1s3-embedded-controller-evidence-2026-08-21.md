# Nonlocal NSR3-B1S3 embedded-controller evidence -- 2026-08-21

Status: `PASS / EMBEDDED_SPECTRAL_ERROR_CONTROLLER_R0 / REPORT_ONLY`

## Outcome

The frozen controller passes all six B1S2 parent cases and the previously
unexecuted `0.97dx`, base-`kappa` holdout. The pressure spectrum is useful as
an initial step estimate; deterministic step doubling then measures the
finite-amplitude trajectory error and refines only the cases that need it.

All `0.99dx` cases accept the initial spectral count. All `0.98dx` cases and
the `0.97dx` holdout reject that count and accept its first doubling. The
holdout accepts 92 substeps per frame with normalized velocity difference
`0.000903905c`, below the unchanged `0.001c` gate. Its final position and
velocity self-convergence ratios are `1.89989` and `1.89323`.

## Selection matrix

| Spacing | `kappa` factor | Initial | Accepted | Comparator | Controller total | Discarded | Error / `c` |
|---:|---:|---:|---:|---:|---:|---:|---:|
| `0.99` | `0.25` | 20 | 20 | 40 | 60 | 40 | `0.000394` |
| `0.99` | `1` | 39 | 39 | 78 | 117 | 78 | `0.000402` |
| `0.99` | `4` | 78 | 78 | 156 | 234 | 156 | `0.000402` |
| `0.98` | `0.25` | 22 | 44 | 88 | 154 | 110 | `0.000527` |
| `0.98` | `1` | 43 | 86 | 172 | 301 | 215 | `0.000537` |
| `0.98` | `4` | 86 | 172 | 344 | 602 | 430 | `0.000537` |
| `0.97` holdout | `1` | 46 | 92 | 184 | 322 | 230 | `0.000904` |

Every active spectrum costs 48 HVP calls. The table's total is online
controller work through the accepted/comparator pair: `3x` the accepted
substeps at refinement depth zero and `3.5x` at depth one. The required
third-level convergence validation adds a separate `4n` run only to the
depth-zero research cases; the machine report exposes that validation-only
work independently rather than disguising it as runtime cost.

Nonlinear HVP overhead is lower than the raw substep ratio because the
pressure active set becomes empty during relaxation, but it remains material:
the holdout executes 348 nonlinear HVP calls, of which only 106 belong to the
accepted trajectory. B1S3 is therefore a correctness selection, not a
performance selection.

## Validity controls

- All 18 parent `n/2n/4n` phase-state hashes exactly reproduce B1S2.
- Every computed level passes finite-state, density, pressure-exit, COM,
  momentum, trust-work and capacity gates.
- Accepted substeps remain at or below 192; no validation level exceeds 768.
- `kappa=0` free flight/translation and inactive pressure states select one
  substep with zero spectral HVP calls and no comparator.
- Accepted state is always the coarse member of the passing pair. Comparator
  state and its work are discarded and separately reported.

## Repeatability and lineage

- B1S3 semantic result SHA-256:
  `fc3e0cff38f748f5cfd09803f2e0b2df2dd65069c91ef33938dd01245ab3b5c1`;
- two byte-identical raw reports:
  `dbe9ce4d3037ad69c8a97b5c84166473e7c6ee08c4d226e1df39a23c8ed36ec6`;
- all 15 checked historical reports remain byte-identical, including B0R,
  B1/B1D/B1D1 and B1S/B1S1/B1S2.

## Interpretation

The result resolves the B1S2 failure without fitting a new safety factor:
the controller observes an error, refines, and generalizes to a held-out
compression amplitude. It does not yet prove that independently accepting
successive macro frames preserves the same global accuracy, nor that paying
for discarded comparators every frame is viable.

The next discriminator is B1R: run the controller transactionally across the
full manufactured `0.05 s` horizon, recompute policy state only at macro-frame
boundaries, publish accepted and discarded work, and compare the composed
trajectory against an independently finer reference. Only that result may
reclose the original B1 multi-step gate and authorize B2 boundary design.

## Decision

Select `EMBEDDED_SPECTRAL_ERROR_CONTROLLER_R0` for report-only CPU research
and authorize B1R design. This grants no B2, boundary, hydrostatic, CUDA,
runtime, public-schema or production authority.
