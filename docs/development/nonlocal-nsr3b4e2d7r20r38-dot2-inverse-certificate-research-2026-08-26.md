# NSR3-B4E2D7R20R38 Dot2 inverse-certificate research

Status: `RESEARCH COMPLETE / COMPENSATED RESIDUAL CERTIFICATE SELECTED`.

## Question

Can a bounded, same-binary128 compensated dot product certify the exact R37
torsion inverse residual without relying on the arbitrary-size integer oracle
in the candidate path?

## Basis

R37 proves exact contraction (`rho_exact <= 0.004444`) and localizes the
remaining problem to cancellation in 65-term matrix products. Ogita, Rump and
Oishi's Algorithm 5.3 (`Dot2`) combines error-free `TwoProduct`/`TwoSum`
transformations and proves, absent underflow,

```text
abs(dot2(x,y) - x^T y)
  <= u * abs(x^T y) + gamma_n^2 * abs(x)^T abs(y).
```

Primary source: https://doi.org/10.1137/030601818 and the authors' manuscript
https://www.tuhh.de/ti3/paper/rump/OgRuOi05.pdf, Algorithm 5.3 and Proposition
5.5. Their verified-linear-system work motivates using such residual bounds in
a nonsingularity/error certificate:
https://www.tuhh.de/ti3/paper/rump/OgRuOi05z.pdf.

For our residual entry, include identity directly in a length-66 dot product:

```text
(I - AX)_ij = [identity_ij, A_i,:] dot [1, -X_:,j].
```

This avoids a separately rounded subtraction. Use `TwoProductFMA`, `TwoSum`
and round-to-nearest binary128 only. Bound construction uses upward one-ULP
wrappers for every nonnegative multiply/add/divide and conservative
`u=2^-112`. Any nonzero subnormal intermediate rejects the lane instead of
using an unimplemented underflow term.

## Hypotheses

| ID | hypothesis | discriminator |
|---|---|---|
| C1 | Dot2 is sufficient | all 4225 exact residuals are contained and row-sum `rho_bound < 1` |
| C2 | Dot2 value is accurate but theorem bound remains too wide | containment passes, `rho_bound >= 1` |
| C3 | implementation or no-underflow assumption fails | controls, error-free identities, containment or normality reject |

## Decision

Implement one report-only R38 shadow over the captured R37 matrix/inverse.
Keep exact dyadic residuals only as an independent oracle. If C1 passes,
research a separately bounded original-solution residual/refined-error audit;
do not yet replace the inverse verifier or continue torsion. If C2 holds,
research `DotK` or a componentwise Krawczyk inclusion. C3 rejects the apparatus.
