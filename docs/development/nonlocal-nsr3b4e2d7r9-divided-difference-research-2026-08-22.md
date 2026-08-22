# NSR3-B4E2D7R9 computational divided-difference research

Date: `2026-08-22`

Status: `RESEARCH_COMPLETE / CONTRACT_FROZEN / NOT_RUN`

## Question

D7R8 proves that the current binary64 actual-reduction calculation reverses
the sign of at least eleven failed trials. Can the same binary64 state and
the same physical objective yield an accurate trust-region numerator if the
current-to-trial difference is propagated directly instead of subtracting
independently rounded absolute values?

## Relevant numerical method

Vavasis describes computational divided differencing specifically for
optimization sufficient-decrease and trust-ratio tests. The transformation
propagates `f(x+s)-f(x)` through each operation, cancelling common terms
before they are rounded. The paper gives the exact forms needed here:

- multiplication and squaring use factored deltas;
- square-root delta is the squared-argument delta divided by the sum of the
  two roots;
- a squared `max(0,x)` penalty uses a factored square when the branch is
  unchanged and direct non-cancelling values across the zero branch;
- piecewise cubics use factored within-segment deltas and telescope through
  knots when a step crosses a segment.

Those rules map directly to radius, the compact cubic kernel and the PHR
penalty in our objective. They address evaluation of the actual reduction;
they do not substitute a Taylor/model prediction for it. See
[Vavasis, *Some notes on applying computational divided differencing in optimization*](https://optimization-online.org/wp-content/uploads/2013/07/3957.pdf).

The remaining sums are still susceptible to accumulation error. Ogita, Rump
and Oishi show that error-free transforms and compensated summation can
approach twice-working-precision accuracy using only working-precision
operations. D7R9 does not yet implement their full accurate-sum algorithm,
but uses a deterministic compensated accumulator as the smallest candidate
and retains stronger expansion arithmetic as a later route. See
[Ogita, Rump and Oishi, *Accurate Sum and Dot Product*](https://ogilab.w.waseda.jp/ogita/math/doc/2005_OgRuOi.pdf).

Noise-aware trust-region rules are a separate fallback family. Sun and
Nocedal relax the acceptance ratio using a known error bound, while a recent
LDL trust-region implementation switches to gradient-norm decrease when the
objective difference reaches machine precision. Those results confirm that
the observed failure is a known optimizer boundary, but neither is the first
choice here: our error is deterministic, the objective is algebraically
structured, and a physical energy solve still needs a meaningful actual
reduction. See [Sun and Nocedal](https://arxiv.org/abs/2201.00973) and the
[LDL trust-region method](https://epubs.siam.org/doi/10.1137/23M1623380).

## Candidate algebra

For a current pair displacement `d`, representable trial step `s`, current
radius `r` and trial radius `r'`, compute:

```text
delta_r2 = s dot (2 d + s)
delta_r  = delta_r2 / (r' + r)
```

The kernel coordinate is `q=2r/h`. Within either cubic segment, compute the
polynomial difference from `delta_q` by factoring the difference of squares
and cubes. If the step crosses `q=1` or `q=2`, telescope through that exact
knot. This also handles the continuous zero-support transition without
subtracting two nearly equal full kernel values.

Accumulate `mass * delta_weight` directly into per-centre density deltas.
Build the current density once with compensated binary64 summation, then
propagate:

```text
delta_constraint = delta_density / rest_density
delta_active     = kappa * delta_constraint       # unchanged active branch
delta_active_sq  = delta_active * (2 active + delta_active)
```

Across the PHR zero branch, one square is zero and the direct expression does
not cancel. The inertia delta similarly uses
`2 * current_displacement dot step + step dot step`. A compensated final sum
combines PHR and inertia contributions and changes sign to obtain
`E(current)-E(trial)`.

## Discriminator design

Run three binary64 measurements on all 23 exact D7R8 replay trials:

1. retained D7R7 direct reduction;
2. independently recomputed compensated absolute energies followed by
   subtraction;
3. the complete divided-difference candidate above.

Only the eleven D7R8-resolved trials score a candidate. A candidate must match
every resolved sign, repair at least one causative false ascent and stay
within 50% of the compensated extended magnitude on every scored trial. The
wide magnitude bound is deliberate: D7R8 fixed-order and compensated
long-double reductions differ by up to about 25% at the smallest resolved
case. The twelve unresolved trials are emitted but never called correct,
accepted or used to tune the gate.

## Decision tree

1. Select `COMPENSATED_ABSOLUTE_REDUCTION_CANDIDATE` if the simpler absolute
   evaluator closes every scored trial.
2. Otherwise select `DIVIDED_DIFFERENCE_REDUCTION_CANDIDATE` if the propagated
   delta closes every scored trial.
3. If all remaining mismatches occur only on a kernel/PHR branch crossing,
   select `BRANCH_RECLOSURE_REQUIRED`.
4. Otherwise select `STRONGER_ARITHMETIC_REQUIRED`; the next candidate would
   use error-free expansions, not wider particle state.

Every route is private evidence only. A passing reduction formula would still
need a separately frozen private-inner integration, derivative-consistency
checks at newly reachable states, complete pressure-state convergence and
the original physical pilot before any performance or production claim.

