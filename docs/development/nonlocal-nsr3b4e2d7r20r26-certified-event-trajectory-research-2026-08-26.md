# NSR3-B4E2D7R20R26 certified event-side trajectory research

Status: `RESEARCH COMPLETE / OPT-IN REPLACEMENT CANDIDATE SELECTED`.

## Question

Does replacing a dyadic pre-event acceptance by one forward-certified
post-event trial remove the shear active-face cycle within the unchanged
32-step cap without regressing the complete 12-case corpus?

## Candidate policy

Keep default semantics unchanged. Under a separate opt-in only, first execute
the existing dyadic line. If and only if its first accepted trial is mask-stable
and every larger existing dyadic trial crosses a mask, enumerate current-face
zero-lower-bound releases along the unchanged representative direction.

For a unique event inside the accepted/rejected-double bracket with derivative
`dz>0`, compute

```text
alpha_cert = nextafterq(root + B_z/dz, +HUGE_VALQ),
```

where `B_z` is exactly the R25 sparse-entry forward bound. Evaluate one new
trial. Replace the original accepted trial only if it is cone-feasible, changes
exactly the predicted scalar, retains ball activity and passes rigorous Armijo.
Otherwise commit the old dyadic acceptance byte-for-byte.

## Gates

- default R8 and R17 complete semantics remain exact;
- replay all eight v2 and four v3 cases once under the opt-in;
- require all historical certified cases to remain certified;
- report replacement/fallback counts, accepted iterations, line evaluations,
  active-set work and KKT tuple per case;
- preserve cap 32, all tolerances and dual-ratio refinement policy;
- candidate success requires all 12 cases certified, not merely shear progress.

This first trajectory deliberately pays one extra evaluation after an existing
dyadic bracket. It tests convergence causality, not performance. Proactive
event search and removal of rejected dyadic evaluations require a later
work-neutral contract even if R26 succeeds.

