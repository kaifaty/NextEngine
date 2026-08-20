# NPR0-E — tiny physical corpus

Status: SPECIFIED / IMPLEMENTATION_NEXT / REPORT_ONLY

## Purpose

This bounded binary64 corpus decides whether the unchanged-control or
dimensionally-derived static-support profile may enter NPR1. It is a profile
selection gate, not a replacement for the full external water corpus.

The corpus is specified before either profile is executed. A failed threshold
may not be relaxed in place after viewing the result. One remediation must
issue a new named corpus/profile identity.

## Compared profiles

| Short name | Full profile | kappa | lambda |
| --- | --- | ---: | ---: |
| control | nuv-basin-48k-static-support-control.v3 | 1 | 1.5 |
| derived | nuv-basin-48k-static-support-derived.v3 | 576 | 360 |

Both use spacing 0.05 m, support 0.1 m, mass 0.125 kg, 1/240 s cadence,
four binary64 SISSM iterations in the bounded CPU corpus, two-layer fixed
support and the split swept-sphere contact operation.

Four CPU iterations are a corpus capacity, not a production iteration
selection. NPR1 must reclose iteration/convergence behavior.

## Frozen cases and gates

### TPF-1 exact free fall

- one isolated sample, no boundary;
- gravity -9.81 m/s², zero initial velocity;
- 16 accepted substeps;
- semi-implicit recurrence uses velocity plus gravity then position plus the
  published velocity times dt.

Gate: maximum position and velocity absolute error are each at most 1e-12.
The case must activate no contact.

### TPH-1 hydrostatic rest

- analytical box 0.1 × 0.2 × 0.1 m;
- 2 × 2 × 2 fluid samples in the bottom half;
- exact two-layer complement: 272 fixed support samples, 280 total solver
  participants;
- gravity -9.81 m/s², zero initial velocity, 24 accepted substeps;
- split outer-box swept contact on every tentative trajectory.

Gates:

- exact fluid count and mass: 8 and 1 kg;
- mean positive compression at the accepted final state at most 1e-4;
- maximum centre penetration at most 0.0025 m;
- horizontal centre-of-mass drift at most 1e-10 m;
- every support sample remains fixed;
- all state and metrics are finite.

Maximum speed and maximum positive compression are reported but are not NPR0
gates. NPR1 owns broader hydrostatic reference and long-horizon criteria.

### TPR-1 reversible rigid mode

- free 2 × 2 × 2 block, no gravity or boundary;
- uniform velocity (0.2, -0.1, 0.15) m/s;
- one substep, negate the resulting uniform velocity, one return substep.

Because pair distances do not change under a rigid translation, this is a
zero-strain reversible control even with bulk viscosity enabled.

Gate: maximum return-position error divided by spacing and maximum
return-velocity error divided by initial speed are each at most 1e-9.

### TPW-1 face and corner contact

- one fluid sample plus the exact 208-sample two-layer complement of a 0.1 m
  cube;
- separate bottom-face and simultaneous lower X/Y/Z corner trajectories;
- initial speed magnitude is high enough to cross the complete box in one
  step, so tunnelling cannot pass accidentally.

Gates:

- accepted centre penetration at most 1e-12 m;
- expected stable outer feature IDs activate;
- fluid impulse plus boundary reaction absolute residual at most 1e-12;
- no support sample moves and all outputs are finite.

## Selection rule

1. If exactly one profile passes every gate, select it as
   NONLOCAL_PRODUCT_PROFILE_CANDIDATE.
2. If both pass, select derived only when its final hydro mean positive
   compression is no worse than control. Otherwise select control.
3. If neither passes, record PROFILE_RECLOSURE_REMEDIATION_1 with the first
   failing case/metric for each profile. Do not start NPR1 or runtime work.
4. The selected result remains report-only and must carry its exact corpus
   result root into NPR1.

No wall-clock timing participates in selection.

## Explicit exclusions

Dam break, orifice, long still tank, surface tension, shear viscosity,
moving rigid bodies, triangle/SDF boundaries, canonical micrometre
publication, GPU physical agreement, runtime integration and Windows.

