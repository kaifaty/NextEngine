# Nonlocal NSR3-B1R1 fine-state evidence -- 2026-08-21

Status: `PASS / NSR_MULTISTEP_CANDIDATE / REPORT_ONLY`

## Outcome

Committing the already computed fine member of each passing step-doubling pair
closes the manufactured multi-step gate across all three compression
amplitudes. No additional spectrum, trajectory or nonlinear solve is executed
relative to B1R; only authoritative state ownership and work attribution
change.

The strongest `0.97dx` case ends at `0.04684dx` position error,
`0.000483c` velocity error and `0.0250` relative kinetic error, all below the
unchanged B1R limits. Its coarse-owned parent ended at `0.13446dx` and failed.

## Matrix

| Spacing | Accepted / executed / discarded substeps | Error / `dx` | Error / `c` | KE error | Result |
|---:|---:|---:|---:|---:|---|
| `0.99` | `100 / 150 / 50` | `0.03324` | `0.000344` | `0.0770` | PASS |
| `0.98` | `194 / 334 / 140` | `0.02957` | `0.000305` | `0.0275` | PASS |
| `0.97` | `206 / 355 / 149` | `0.04684` | `0.000483` | `0.0250` | PASS |

Compared with B1R, executed substeps remain exactly `150/334/355`. Accepted
work rises because the fine trajectory becomes authoritative, while discarded
work falls from `100/237/252` to `50/140/149`. The first passing pair costs
`1.5x` accepted work at depth zero and `1.75x` at depth one; subsequent
pressure-inactive viscosity frames use the measured `1/2` pair and fine
ownership as well.

Position error improves by approximately `2.17x`, `2.76x` and `2.87x` over
coarse ownership. The gain is larger than a single terminal factor-of-two
because the more accurate first-frame velocity is composed through the next
11 macro frames.

## Correspondence and validity

- All nine fixed `96/192/384` reference phase hashes and work records exactly
  overlap B1R; reference position/velocity ratios remain `1.95--1.97`.
- Initial pressure counts, 48-HVP spectra, substep counts and all first-frame
  coarse/fine gate observables exactly overlap B1R.
- Every transaction is immutable until commit; all candidate and reference
  states pass density, pressure-exit, COM, momentum, trust-work and capacity
  gates.
- Each case has one pressure-active spectrum frame and finishes with zero
  active pressure centers.
- Free flight and rigid translation use all 12 inactive fast-path frames with
  zero spectrum, probe and nonlinear-HVP work and retain analytic accuracy.

## Repeatability and lineage

- B1R1 semantic result SHA-256:
  `e215b0facc30445541a6f2fa9446fd9d8140bf5f66983180cb435de9863f535e`;
- two byte-identical raw reports:
  `af34c3d8e142610bb11d26592a8b9f673af6ff941f89c7b8631f178cd70f0af1`;
- the B1R negative report remains byte-identical at
  `cfed7f1a76193d5780209be5f78b61d99b08933873fe6c22218acc9c5b143620`;
- B1S3 remains the separately rooted interval-controller selection and is not
  retroactively changed.

## Interpretation

Step doubling was doing two jobs: estimating error and producing a better
state. B1R discarded that better state, then paid the resulting velocity
error throughout the horizon. B1R1 separates those concerns correctly: the
coarse run is an error probe; the fine run is the accepted transition.

This is also a material performance result. Correct ownership reduces rather
than increases speculative overhead. It does not eliminate the probe or the
48-HVP spectrum, so later performance work must still amortize or predict
them without changing the accepted-state semantics.

## Decision

Select `NSR_MULTISTEP_CANDIDATE` and authorize design of NSR3-B2 static
boundary formulas. The original B1 and B1R failures remain preserved. This
does not authorize boundary execution, hydrostatics, CUDA, runtime integration
or production use.
