# NSR3-B3 -- split boundary-composition smoke contract

Status: `FROZEN / IMPLEMENTATION_AUTHORIZED / PHYSICAL_CORPUS_BLOCKED`

Parent selection: `SPLIT_STATIC_BOUNDARY_FORMULA_CANDIDATE`, B2 semantic
SHA-256 `80a01b2ed0cf844841da322233b121b33e273c71eace8682795d0cad4e1dfb80`.

Objective/solver identity remains
`nuv-variational-fcr2 + split-static-boundary-r0` /
`nuv-newton-krylov-r0 + outer-state-hessian-tape-v1`. B3 adds only the
report identity `post-solve-swept-contact-composition-r0`.

## Single tested hypothesis

For every accepted substep of duration `h`:

1. predict `y*=x+h(v+h*g)`;
2. rebuild exact fluid-fluid and visible fluid-boundary support;
3. minimize inertia plus B2 fluid-centred pressure energy with immutable
   boundary positions;
4. sweep every segment from substep start `x` to smooth result `ys` against
   the frozen static plane/box;
5. commit the swept position and reconstruct `v=(y-x)/h`;
6. invalidate the smooth support, active set, spectrum and Hessian tape before
   the next substep.

No predictor pre-clamp, post-contact positional repair, warm curvature cache,
contact HVP, restitution or friction is admitted. `lambda=mu=gamma=0` remains
the B2 isolation. Gravity is the only external body force.

## Fixtures

Use exact `dx=0.05 m`, `H=0.15 m`, `R=0.025 m`, `rho0=1000 kg/m3`,
`m=0.125 kg`, `kappa=1226.25`, frame time `1/240 s` and four macroframes
(`1/60 s` total).

Two lattice fixtures are frozen:

- **face slab:** `5x3x5` fluid samples, two static support layers below the
  `y=0` plane, `0.99` compression about the fluid centroid followed by a
  translation that places the minimum fluid `y` at `0.035 m`,
  `v=(0,-1,0) m/s`, `g=(0,-9.81,0) m/s2`;
- **corner column:** `3x3x5` fluid samples, union of two static support layers
  behind `x=0` and `y=0`, the same compression followed by translation to
  minimum fluid `x=y=0.035 m`, `v=(-0.7,-0.7,0) m/s`, the same gravity.

Tangent ranges include the full nonzero cubic stencil for at least one
face/corner centre. Boundary samples are deduplicated exact integer-lattice
coordinates and filtered by `r<=H`. The report publishes fluid/boundary
counts, maximum pair count and exact geometry hashes. Each fixture must remain
within 512 total samples and `80*(F+B)` unique support pairs.

## Fine-owned transaction

At each macroframe start:

- if pressure is active, estimate the largest **boundary-aware pressure**
  tangent eigenvalue with the frozen 48-HVP Lanczos control and set
  `n=max(1,ceil(frame_time*sqrt(max(lambda_max,0)/m)/0.15))`;
- if pressure is inactive, set `n=1` and charge zero spectral HVPs;
- execute `(n,2n)`, then at most `(2n,4n)` and `(4n,8n)` from the same immutable
  frame start until the B1S3 interval gate passes;
- commit the fine member of the first passing pair and charge every coarser
  member as discarded work, exactly as B1R1.

Candidate trajectories include the complete smooth-solve/contact/rebuild
sequence. Contact counts and impulses cannot be omitted from coarse/fine
comparison. A candidate failure or rejected trust step cannot mutate the
frame start.

## Independent reference and accuracy

For each fixture run fixed `96/192/384` substeps **per macroframe**, including
the identical analytical contact source but independent fixed scheduling.
Require the two successive reference differences to be finite, positive and
have position and velocity ratios in `[1.25,2.75]`; the wider event ratio is
frozen because the impact is nonsmooth.

Compare the committed candidate to the 384-level reference at `1/60 s`:

- mass-weighted RMS position error `<=0.05dx`;
- mass-weighted RMS velocity error `<=0.001c`, where
  `c=sqrt(kappa/m)`;
- relative kinetic-energy error `<=0.15`;
- first-contact time difference `<=` one accepted candidate substep;
- identical set of contacted stable particle/feature IDs at the terminal
  state.

The candidate must encounter at least one contact and at least one active
pressure centre. It need not retain smooth second-order convergence through
impact.

## Solver, contact and ledger gates

Every executed smooth solve must retain the selected trust gates: finite
state, positive actual/model reduction on accepted trials, ratio `>=0.1`, at
most eight rejected trials and immutable rejected state. The report publishes
outer/reject/HVP counts, negative-curvature exits and arithmetic-floor stops.

For every accepted substep require:

- penetration `<=1e-12 m` and sorted stable contact feature IDs;
- finite time of impact in `[0,1]` for each active feature;
- velocity exactly reconstructed from accepted position and substep start;
- pressure support reaction closes its virtual derivative at `<=1e-10`
  normalized residual;
- contact fluid impulse plus contact reaction norm `<=1e-12 kg m/s`;
- complete momentum ledger
  `Delta P - M*h*g + Jb_support + Jb_contact` has normalized residual
  `<=1e-9`;
- boundary positions remain bit-zero and every contact invalidates the cached
  support/spectrum/tape epoch before reuse.

The final report separately publishes fluid support impulse, static support
reaction, contact impulse, contact reaction, gravity impulse and ledger
residual. Repeated contact on the same feature is reported, not treated as a
failure unless it breaks accuracy, capacity or work bounds.

## Repeatability, regression and exit

- Two complete B3 reports are byte-identical.
- B2 and B1R1 raw reports remain byte-identical.
- One derivation/transcription correction is allowed. A second smooth-formula
  mismatch stops B3. One contact-order remediation is allowed only if the
  failure localizes to post-solve composition and must receive a new identity.

PASS selects `STATIC_BOUNDARY_SMOKE_CANDIDATE` and authorizes only B4 physical-
corpus contract design. It does not authorize hydrostatic/dam-break execution,
moving or deformable solids, triangle meshes, friction, CUDA, performance,
runtime schemas, save/replay or production integration.
