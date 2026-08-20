# FCR3-B — corrected SISSM/reference correspondence

Status: `FROZEN FOR IMPLEMENTATION / REPORT_ONLY`

Predecessor: FCR2 reference `PASS`; FCR3-A conditioning branch `CLOSED FAIL`.

## Candidate

`corrected-sissm-armijo-v1` transcribes the semi-implicit positive/negative
split against the corrected FCR0 energy and FCR1 pair convention. It does not
reuse the stopped source-shaped kernel, coefficients or profile.

For each iteration it builds local `3x3` matrices and sources, computes one
parallel-Jacobi-style candidate, then treats `candidate-y` as a direction.
The exact FCR2 objective accepts that direction with deterministic Armijo
backtracking. A non-descent direction is a typed failure.

## Term splits

### Compression

Density is recomputed at current `y`. Only active compression states
`rho_i>rho0` contribute. For every active density center `i` and each neighbor
`j`, use physical `w=dW/dr`,

```text
a_ij = kappa*dt^2/rho0 * w(r_ij)/r_ij   <= 0
J_i  = rho_i/rho0
```

and the SISPH positive/negative split `J_i + (-1)`. The center contribution
and its equal/opposite reaction are both accumulated. Every directed density
state is evaluated exactly once even though geometric pairs are stored
uniquely.

### Viscosity

For each unique reference pair, the exact full-pair quadratic block is

```text
L_ij = dt*omega/rho0 * (2*mu*P_t + lambda*P_n).
```

`L_ij` is implicit on the owner diagonal and the current neighbor position
plus reference displacement is explicit on the right-hand side. This is the
FCR1 full unique-pair coefficient, not the stopped directed/full mixture.

### Surface

For each unique current pair,

```text
l_ij = 2*gamma*m*dt^2*c(r/r0)/r.
```

Positive `l_ij` (attraction) is implicit; negative `l_ij` (repulsion) is
explicit. Both endpoints receive equal/opposite contributions.

## Overshoot safeguard

The raw local solution is never published directly. Let `p=y_sissm-y`.
Require `dot(gradient(E),p)<0`, then run the unchanged FCR2 Armijo rule from
`alpha=1`, halving at most `40` times. Every accepted iterate must lower the
exact objective. No Chebyshev or Anderson acceleration is present in v1.

## Corpus and gate

Run the exact FCR2 active cases with the same maximum `80` iterations:

- compressed pair;
- normal/shear viscosity pairs;
- repulsive/attractive surface pairs;
- combined tetrahedron.

For every case:

- finite, monotonic and internal momentum residual `<=1e-12`;
- physical direction preserved;
- final objective no worse than FCR2 by relative `1e-10`;
- final gradient no worse than `2x` FCR2 with `1e-8` absolute floor;
- no non-descent direction or exhausted line search.

Aggregate objective evaluations (iterations plus rejected trials) must be at
least `4x` lower than FCR2 on the three previously stiff cases. Two reports
must be byte-identical and FCR0–FCR2/frozen old controls unchanged.

Passing authorizes a separately frozen Chebyshev A/B and then FCR3-C profile
reclosure. Failure allows at most one term-localized discriminator; it does
not authorize coefficient tuning or profile sweeps.
