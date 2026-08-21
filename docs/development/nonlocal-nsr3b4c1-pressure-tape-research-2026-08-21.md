# NSR3-B4C1 compact pressure-tape research -- 2026-08-21

Status: `COMPLETE / RADIUS_TAPE_CSR_SELECTED`

## Question

How should the selected joint pair order be reused across trust-region HVPs
without importing the fluid-only full multi-term tape's memory cost?

## Cost audit

B4C0R's untaped pressure HVP calls `evaluate_joint` internally. For an active
outer state that means, per HVP:

```text
P pair radii for density
+ D directed radii for pressure gradient
+ 2D directed radii for the HVP
= P + 3D norm/sqrt evaluations
```

Here `P` is unique fluid-fluid plus fluid-support pairs and `D` is directed
fluid-centred adjacency. Every Krylov/model HVP repeats work whose positions
and active set are constant for the outer state.

The earlier full A2 tape stores normal, Jacobian and radial coefficients for
all enabled pressure/viscosity/surface terms. B4C is pressure-only and adds
static-support participants. Copying the full record would spend roughly
`48--80` bytes per pair/directed record before CSR and can reach several
hundred megabytes at the admitted bound.

## Selected representation

Keep the B4C0R unique `u32[2]` pair list. Add:

```text
offsets[N_fluid + 1]       u32
directed_pair_index[D]     u32
radius[P]                  f64
compression[N_fluid]       f64
```

CSR rows follow the already proven canonical adjacency order. A directed slot
derives its participant and orientation from the unique pair record. Radius is
computed once with the exact inherited `norm`; HVP reconstructs displacement
from the immutable outer state, divides by the stored radius and evaluates the
unchanged kernel derivative expressions. Compression is copied from the same
outer evaluation used by the solver.

Tape build costs `P` radii once. Each taped HVP performs zero norm/sqrt calls
and no density/gradient evaluation. Arithmetic within both directed HVP loops
remains identical to B4C0R.

At the maximum admitted payload:

```text
existing pair list               64,000,000 bytes
tape radii                       64,000,000 bytes
CSR directed pair indices        32,000,000 bytes
CSR offsets                         200,004 bytes
centre compression                  400,000 bytes
total joint pressure payload    160,600,004 bytes
```

This is a bounded report-only ceiling, not a production memory budget. Later
nominal evidence must report actual admitted pairs/degrees; reducing the hard
cap or compressing exact `f64` coefficients requires a separate decision.

## Rejected alternatives

- full A2 coefficient records: unnecessary multi-term memory for pressure-only
  B4C;
- `f32` radii/normals: cannot preserve exact HVP arithmetic;
- participant-only CSR with a pair binary search: adds `log P` work to every
  directed record;
- unordered maps or cell-order rows: violate the proven reduction order;
- recompute density inside each HVP: preserves correctness but discards the
  selected outer-state reuse.

## Decision

Freeze a bit-exact radius-tape/CSR discriminator before substituting any
solver query. B4C2 remains blocked until inactive, onset-active, support
reaction, permutation, capacity and payload gates pass.
