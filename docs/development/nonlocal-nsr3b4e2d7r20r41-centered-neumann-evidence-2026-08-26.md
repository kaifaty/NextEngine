# NSR3-B4E2D7R20R41 centered Neumann evidence

Status: `PASS / CENTERED_NEUMANN_SOLUTION_CANDIDATE / 65 OF 65 RESOLVED`.

Implementation `e92a7b8c` emits reproducible semantic:

```text
0ff6f9cc597166ca962e3fa3c97b29ed6350f62b22c6bb2768fa2a95daedf3cb
```

R40 and every earlier parent reproduce exactly. The materialized left matrix
and directional-vector roots are `45237f7a...013b` and `4db7e3e6...1a6b`;
all `4225+65` exact dyadic entries are contained without underflow.

| depth | certified radius | signs `+/-/?` | minimum separation |
|---:|---:|---:|---:|
| 1 | `4.2908247950e10` | `24/35/6` | negative |
| 2 | `6.3963555629e6` | `24/35/6` | negative |
| 4 | `1.4213997320e-1` | `24/41/0` | `1.0542439169e3` |
| 8 | `4.5753008642e-14` | `24/41/0` | `1.0543860568e3` |
| 16 | `4.5682812285e-14` | `24/41/0` | `1.0543860568e3` |
| 32 | `4.5682812285e-14` | `24/41/0` | `1.0543860568e3` |

The first predeclared resolving depth is four. The six R39/R40-ambiguous
components are all rigorously negative; none of the 59 previously certified
signs conflicts. At depth four the interval is already separated from zero by
more than `1054`, so this is not a marginal sign decision.

At depth eight the exact fixed-point residual has reached the arithmetic input
enclosure floor. Further iterations cannot materially improve the issued
radius: the maximum bound remains about `3.9663e-14`, while the maximum actual
residual-evaluation error is only `6.266e-19`. This plateau is expected and is
not a convergence failure.

All six checkpoint residuals and all six compensated centers contain their
independent exact dyadic oracles. Two R41 executions are byte-exact; R40 and
R39 remain at semantics `a999b65a...f7e` and `83e1f738...1bac`.

R41 performs no factorization, inverse column, solve, correction apply, NNQP
decision/transition, trial or state update. It proves that a four-step centered
refinement is a valid passive-solution candidate. It does not yet prove that
installing that solution makes the torsion NNQP trajectory terminate or
generalizes to another matrix.
