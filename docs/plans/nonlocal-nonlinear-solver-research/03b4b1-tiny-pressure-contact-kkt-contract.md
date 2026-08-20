# NSR3-B4B1 -- tiny pressure corpus with contact KKT

Status: `FROZEN / IMPLEMENTATION_AUTHORIZED / NOMINAL_CORPUS_BLOCKED`

Parent B4BK1 selects `BOX_CONTACT_KKT_CANDIDATE`; semantic SHA-256 is
`48db28247059f4f61870ee8ff9bc680ebbb5e672039c5c199b11e14dbe980197`
and JSON-without-final-LF SHA-256 must equal
`eed7934a81dc451edc2eeaaee81dcb403e2bb94e6799edfdc1646cb7970b4bdd`.
B4B r0 remains exact FAIL.

## Identity and sole composition change

```text
tiny-pressure-water-corpus-r1-contact-kkt
```

Reuse every B4B P1/P2 particle, support sample, coefficient, macro horizon,
spectral target, embedded `(n,2n)` controller, fixed `48/96/192` ladder,
state/aggregate gate, work cap and stop policy unchanged.

Replace only this r0 substep:

```text
unconstrained pressure solve -> post-solve analytical box sweep
```

with the selected B4BK1 bound-constrained solve:

```text
min M/(2h^2)||delta-delta*||^2 + Phi(x+delta;q)
s.t. low-x <= delta <= high-x.
```

Use owned displacement/inertia, feasible projected predictor, active/free
Steihaug trust steps, at most 64 outer trials and four strictly decreasing
projected-KKT floor-merit accepts exactly as B4BK1. Velocity is `delta/h`.
There is no subsequent position sweep.

## Contact identity and ledger

At every accepted substep derive multiplier and feature IDs from the final
KKT state. First contact is the first positive multiplier. Report per-face
counts, multiplier sums and signed impulses; counts may change with the
trajectory.

Keep pressure support and contact reactions separate:

```text
J_pressure_fluid = -h sum grad_fluid Phi
J_pressure_wall  = -h sum grad_support Phi
J_contact_fluid  =  h sum g_active
J_contact_wall   = -J_contact_fluid.
```

Require exact complementarity, nonnegative multipliers, penetration
`<=1e-12 m`, pressure translation closure `<=1e-10`, contact closure
`<=1e-12 N s`, projected KKT impulse within the inherited mixed limit and the
complete normalized momentum ledger `<=1e-9` on every accepted and reference
substep.

## Inherited physical and accuracy gates

P1 remains supported-column startup for 8 frames; P2 remains released-block
impact for 16. All B4B gates are inherited literally, including:

- counts/mass, lateral COM, P1 vertical COM, compression/speed and energy;
- P2 exact pressure/contact-free semi-implicit recurrence before its first
  multiplier, terminal pressure/contact activation and contact-time accuracy;
- per-frame candidate versus fixed-192 RMS position/velocity, COM, q99 and
  kinetic gates;
- fixed `48/96/192` convergence ratio or computed binary64 floor overlap;
- exact terminal particle/feature IDs.

The embedded spectral estimate remains the unconstrained pressure-Hessian
upper estimate. It may conservatively over-substep active wall states; no
projected-spectrum optimization is authorized here.

## Work, repeatability and exit

Charge all pressure evaluations, analytic HVPs, projected trials, active-set
changes, KKT floor trials/accepts, comparator levels and discarded work.
Retain 1,280 participants, `160*fluid` active pairs, 192 accepted substeps per
frame and 768 substeps for any executed level.

The first failed fixture/gate stops execution. Two reports must be
byte-identical. B4BK1, B4BK r0, B4B r0, B4A, B3R, D5, original B3 and B2 raw
outputs remain exact.

PASS selects `TINY_PRESSURE_CONTACT_KKT_CANDIDATE` and authorizes only B4C
joint fluid/support neighborhood plus canonical-runner design. FAIL preserves
the KKT one-step result but blocks trajectory authority; one-step evidence
may not override a temporal/reference failure.

No general mesh, moving solid, friction, equilibrium, internal aperture,
viscosity, surface tension, nominal water, CUDA, runtime or production
integration is authorized.
