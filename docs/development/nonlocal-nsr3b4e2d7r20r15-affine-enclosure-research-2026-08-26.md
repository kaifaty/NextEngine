# NSR3-B4E2D7R20R15 affine direction-enclosure research

Status: `ANALYTIC LEMMA CLOSED / SHADOW REPLAY SELECTED`.

## Lemma

Let exact vectors satisfy componentwise

```text
|x - x_hat| <= e_x,   |z - z_hat| <= e_z,   0 <= alpha <= 1.
```

For the affine update `y=(1-alpha)x+alpha z` and central update
`y_hat=(1-alpha)x_hat+alpha z_hat`, the triangle inequality gives

```text
|y-y_hat| <= (1-alpha)e_x + alpha e_z.
```

The current implementation instead carries

```text
e_old = e_x + alpha(e_z + e_x) + e_round.
```

With the same nonnegative rounding enclosure,

```text
e_old - e_affine = 2 alpha e_x >= 0.
```

Thus the affine recurrence is rigorously no weaker under the frozen
`alpha in [0,1]` invariant. This is an analytic real-arithmetic enclosure
lemma; executable binary128 correspondence still requires a shadow trace with
the existing rounding term.

## Hypotheses

| ID | hypothesis | discriminator |
|---|---|---|
| A1 | repeated double-counting of `e_x` causes the remaining corner ambiguity | affine-shadow `e_x` plus R14 refined candidate error makes both final ratio tests strict |
| A2 | even correlation-aware scalar error is too coarse | corner remains ambiguous under the shadow bound |
| A3 | the shadow observer perturbs or fails to correspond to the old trajectory | any R13/R14 root changes or affine bound exceeds old bound |

## Selected experiment

Carry an unused affine-error shadow beside the current scalar error through the
unchanged NNQP trajectory. On a full passive replacement set it to the same
candidate error; on a boundary interpolation use the proved recurrence plus
the exact existing rounding term. At the original ratio rejection, combine
the shadow current error with the already certified R14 inverse-refined
candidate error and evaluate a report-only ratio.

No branch, direction, active set, alpha or solver state may consume the shadow.
If A1 holds, a subsequent stage may formulate the combined candidate solver;
it must still re-establish complete NNQP and outer KKT certificates.

