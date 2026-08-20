# Nonlocal NSR3-B1R transactional-composition evidence -- 2026-08-21

Status: `FAIL / COARSE_STATE_COMPOSITION_REJECTED / REPORT_ONLY`

## Outcome

Every local controller transaction, nonlinear solve, spectrum, conservation,
capacity and independent-reference gate passes. The composed trajectories do
not. Accepting the coarse member of each passing step-doubling pair produces
global position error above `0.05dx` for all three amplitudes.

The failure is not nonlinear instability and not an unresolved reference.
The fixed `96/192/384` references converge cleanly with position/velocity
ratios `1.95--1.97`. Pressure is active only in the first macro frame; its
accepted velocity error then propagates ballistically through the remaining
11 frames. A local interval tolerance is therefore not automatically a
global trajectory tolerance.

## Matrix

| Spacing | Accepted / executed / discarded substeps | Reference `q_x/q_v` | Final error / `dx` | Final error / `c` | KE error | Result |
|---:|---:|---:|---:|---:|---:|---|
| `0.99` | `50 / 150 / 100` | `1.972 / 1.971` | `0.07214` | `0.000745` | `0.1631` | FAIL |
| `0.98` | `97 / 334 / 237` | `1.967 / 1.966` | `0.08161` | `0.000843` | `0.0749` | FAIL |
| `0.97` | `103 / 355 / 252` | `1.950 / 1.949` | `0.13446` | `0.001387` | `0.0710` | FAIL |

All controllers complete 12 immutable transactions. Each case runs one
48-HVP spectrum in its first active frame. Later pressure-inactive expanding
states correctly retain bulk viscosity and enter the measured `1/2` path;
they are not misclassified as free flight. Final pressure-active count is
zero and density never exceeds its initial maximum.

## Degenerate comparison correction

The first pilot required bit-exact relative bond velocity for the inactive
fast path. Roundoff after one rigid-translation frame produced differences of
order `1e-15`, falsely disabling that path even though the existing analytic
translation gate passed by four orders of magnitude.

The sole correction admits
`32*epsilon*max(1,|v_i|,|v_j|)` in this comparison. It changes no material
state, trajectory tolerance or compression result. The final degenerate test
uses all 12 fast-path frames for both free flight and rigid translation, with
zero spectral, comparator and nonlinear-HVP work; analytic errors remain
between `7.1e-17` and `1.0e-14`.

## Root cause and next hypothesis

For a first-order method, the coarse/fine difference estimates the fine
solution's leading error more closely than the coarse solution's error. B1S3
deliberately assigned ownership to coarse so that cost and accuracy were not
blurred. B1R shows that ownership rule is unsuitable for long composition.

The fine comparator has already been computed. Committing it would require no
additional solver work, reduce the speculative multiplier from `3x` to
`1.5x` at depth zero and from `3.5x` to `1.75x` at depth one, and should
approximately halve the dominant first-frame error. This must be tested as a
new B1R1 state-ownership policy, not retroactively substituted into B1R.

## Repeatability and lineage

- B1R semantic result SHA-256:
  `377941cd110c07c265b882d8025d40daf0d22383847649c96ddd771a528346db`;
- two byte-identical raw reports:
  `cfed7f1a76193d5780209be5f78b61d99b08933873fe6c22218acc9c5b143620`;
- B1S1, B1S2 and B1S3 remain byte-identical at their prior raw hashes,
  including B1S3
  `dbe9ce4d3037ad69c8a97b5c84166473e7c6ee08c4d226e1df39a23c8ed36ec6`.

## Decision

Reject `TRANSACTIONAL_COARSE_STATE_R0` and keep B2 blocked. Freeze B1R1 to
commit the already computed fine member of the first passing pair while
retaining the same spectra, gates, caps, cases and independent references.
This grants no boundary, hydrostatic, CUDA, runtime or production authority.
