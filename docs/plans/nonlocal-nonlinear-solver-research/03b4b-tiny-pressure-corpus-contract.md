# NSR3-B4B -- tiny pressure-only physical corpus contract

Status: `FROZEN / IMPLEMENTATION_AUTHORIZED / NOMINAL_CORPUS_BLOCKED`

Parent B4A semantic SHA-256 is
`5e069b7e7c86da39aef944d31fc184a564eaa8e794c54f9e63d0e855852567b1`;
its JSON-without-final-LF SHA-256 must equal
`7282f6ab736d4bea423cc20a1dd20092a156afb3eb5d61741d6b8b4e10674c51`.

## Identity and unchanged model

```text
tiny-pressure-water-corpus-r0
```

Use the selected normalized pressure objective, two-layer box-owned static
support, B3R post-solve analytical contact, owned displacement/inertia,
reaction-aware stop and at most four strictly decreasing floor-residual
accepts. Constants are unchanged:

```text
rho0=1000 kg/m^3, dx=0.05 m, R=0.025 m, H=0.15 m,
m=0.125 kg, kappa=1226.25 J, g=(0,-9.81,0) m/s^2,
macro frame=1/240 s, lambda=mu=gamma=0.
```

No coefficient or support normalization is selected by this corpus.

## Fixture P1 -- supported-column startup

- inner box: `[0,0,0]..[0.2,0.3,0.2] m` (`4 x 6 x 4` cells);
- complete box-owned two-layer shell: `(8*10*8)-(4*6*4)=544` fixed samples;
- fluid: `4 x 3 x 4=48` samples at exact lattice centres
  `(R+i*dx,R+j*dx,R+k*dx)`, zero initial velocity;
- horizon: 8 macro frames, `1/30 s` total.

The full horizontal section is filled and the upper half is air. No interior
air position is support. This case is named startup, not equilibrium.

Blocking physical gates:

- exact count `48`, mass `6 kg`, finite state and bit-fixed support;
- lateral COM drift in x/z `<=1e-10 m`;
- final vertical COM differs from its initial value by `<=0.05dx`;
- pressure and lower contact each activate at least once;
- maximum positive density strain `<=1e-3`;
- maximum speed `<=0.01c`, `c=sqrt(kappa/m)`;
- penetration `<=1e-12 m`, support/contact closure and complete per-substep
  momentum ledger retain the B3R limits;
- mechanical energy `K + Phi + sum(m*g*y)` may dissipate, but may not exceed
  its initial value by more than `1%` of
  `max(|E_initial|, M*g*dx, 1e-12 J)`.

Report vertical momentum, support/contact reaction, energy deficit and the
bottom-to-top density profile; they do not receive additional fitted gates.

## Fixture P2 -- released block and floor impact

- inner box: `[0,0,0]..[0.4,0.4,0.4] m` (`8^3` cells);
- complete box-owned two-layer shell: `12^3-8^3=1216` fixed samples;
- fluid: `3^3=27` samples, x/z coordinates `0.125,0.175,0.225 m`, y
  coordinates `0.035,0.085,0.135 m`, zero initial velocity;
- horizon: 16 macro frames, `1/15 s` total.

Before the first contact, require zero active pressure centres, no support
reaction, uniform velocity and the exact semi-implicit gravity recurrence for
the executed substep schedule within `1e-12` position/velocity absolute error.
The first-contact time must agree with the fixed-192 reference within one
accepted candidate substep.

By the terminal frame require at least one pressure-active substep and one
lower-face contact. Also require exact count/mass, lateral COM drift
`<=1e-10 m`, penetration `<=1e-12 m`, B3R support/contact/complete-ledger
limits, finite aggregates and no positive mechanical-energy creation above
the same `1%` formula. Energy deficit and repeated contact remain diagnostic.

## Fine-owned controller and fixed reference

Reuse B3R's boundary-aware 48-HVP spectral estimate and target `0.15`.
At every macro frame execute `(n,2n)` and at most two further doublings from
an immutable frame start. Commit the fine member of the first passing pair;
charge all probes and failed levels. Pressure-inactive rigid free flight may
use the inherited exact one-step fast path only while contact is also absent
over the complete candidate segment.

Each fixture also runs an independent fixed `48/96/192` substeps per macro
frame. Successive final position and velocity RMS differences must either:

- be positive with ratio in `[1.25,2.75]`; or
- both be no greater than a computed binary64 forward bound, reported as an
  exact-floor overlap rather than an invented convergence order.

Compare the candidate with fixed 192 at every macro-frame boundary:

- mass-weighted RMS position `<=0.05dx`;
- mass-weighted RMS velocity `<=0.001c`;
- absolute COM difference per axis `<=0.05dx`;
- q99 height and q99 x-front differences `<=0.10dx`;
- relative kinetic-energy difference `<=0.15` whenever either energy exceeds
  `1e-12 J`, otherwise require the computed floor overlap.

Final contacted particle/feature IDs must match fixed 192 exactly.

## Work, capacity and failure

- fluid plus static support is capped at 1,280 samples;
- active support pairs are capped at `160*fluid_count`;
- accepted substeps are capped at 192 per macro frame and any executed level
  at 768 per macro frame;
- every objective/gradient/HVP, spectral estimate, trust trial, floor accept,
  contact, cache invalidation and discarded substep is reported;
- current all-pairs candidate checks are reported but receive no timing credit.

The first failed fixture/gate stops the corpus. One derivation/transcription
repair may receive a new contract identity; no threshold, horizon, level or
coefficient adjustment is pre-authorized.

## Repeatability and exit

Two complete reports must be byte-identical. B4A, B3R, D5, original B3 and B2
raw outputs remain byte-exact.

PASS selects `TINY_PRESSURE_CORPUS_CANDIDATE` and authorizes B4C joint
fluid/support neighborhood plus canonical-runner design. It does not authorize
nominal hydro/dam-break, external-reference comparison, equilibrium, internal
aperture, viscosity, surface tension, CUDA, performance, runtime or production
integration.

