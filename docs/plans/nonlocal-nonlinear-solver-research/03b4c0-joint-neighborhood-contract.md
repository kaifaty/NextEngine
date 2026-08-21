# NSR3-B4C0 -- joint fluid/support neighborhood contract

Status: `FAIL / P1_TWO_PASS_WORK_GATE / B4C1_BLOCKED`

Parent B4B2 selects `TINY_PRESSURE_CONTACT_FORECAST_CANDIDATE`; semantic
SHA-256 is `b79e537d44519391d9ec65f56134f409122c10ca7135d62f797b026d062b7286`
and JSON-without-final-LF SHA-256 must equal
`8f9c0fb92216770a11f8f0b603bb66f5ce27858815d73b90b2e09266ec19e64b`.

## Identity and operator

```text
joint-fluid-support-cell-pairs-r0
```

Canonicalize fluid and support arrays independently by stable unsigned ID.
Build one cell index over both roles with edge `H=0.15 m`. Emit only:

- fluid-fluid unique pairs with `i<j`;
- fluid-support pairs with the support participant indexed after all fluid;
- no support-support pair.

Final pairs sort lexicographically by `(fluid_i, participant_j)`. Each
fluid-centred adjacency row sorts by combined participant index. Membership
uses the exact inherited binary64 `norm(delta) <= H` predicate so this stage
can compare literally with B4B2; canonical integer continuation belongs to
B4C3.

## Positive controls

Run all-pairs and cell paths on:

1. B4B P1 initial supported column;
2. its feasible clamped macro predictor, which is pressure-active;
3. B4B P2 detached initial block;
4. a signed-coordinate/cell-boundary control containing distances immediately
   below, exactly at and immediately above `H`.

For every case require:

- pair membership and order exact;
- repeated pair digest exact;
- density, energy, active count, branch margin and full fluid/support gradient
  exact;
- pressure HVP exact for a deterministic joint direction and for a
  fluid-only direction with zero support motion;
- cell distance tests strictly below quadratic candidate checks for P1/P2;
- maximum fluid degree `<=160`, pairs `<=160*N_fluid`, and finite output.

Repeat identity, reverse and coprime-affine input storage orders. After stable
ID canonicalization, pair digest, evaluation and HVP must be exact. Report
fluid-fluid and fluid-support counts separately.

## Failure controls

Require exact first errors, no returned partial pair list and no fallback for:

- zero or `50,001` fluid samples: `JOINT_FLUID_CAPACITY`;
- `32,769` support samples: `JOINT_SUPPORT_CAPACITY`;
- duplicate fluid or support ID: `JOINT_DUPLICATE_ID`;
- nonfinite position or unrepresentable cell coordinate:
  `JOINT_POSITION_INVALID`;
- more than `160` participants for one fluid centre:
  `JOINT_NEIGHBOR_CAPACITY`.

Count membership before allocating the final pair/adjacency storage. Checked
arithmetic guards all capacity products.

## Repeatability and exit

Two reports must be byte-identical. B4B2 and all historical B4BF through B2
raw reports remain exact.

PASS selects `JOINT_PRESSURE_NEIGHBORHOOD_CANDIDATE` and authorizes only B4C1
pressure-tape contract design. FAIL preserves B4B2 and blocks scalable,
canonical and nominal work.

No trajectory substitution, canonical continuation, nominal corpus, CUDA,
performance, runtime, public schema or production authority is granted.
