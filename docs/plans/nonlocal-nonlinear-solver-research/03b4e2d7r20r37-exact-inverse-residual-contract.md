# NSR3-B4E2D7R20R37 exact inverse-residual contract

Status: `FROZEN / REPORT-ONLY EXACT DYADIC AUDIT AUTHORIZED`.

## Parent

- R36 implementation `cc1d40fc`, semantic
  `448a9b8b64a015d0dcbceac82555216ad6421e22a5b4a8b62a95fcfe42eab90f`;
- torsion case/step roots `a543bc06...1d6d` / `d274e20e...4cd9`;
- sole inverse/norm roots `5fe4d71e...5e7c` / `c2d53190...6dfd`;
- passive dimension and completed inverse columns `65 / 65`.

## Frozen audit

Replay only the exact torsion case through the unchanged R35 policy. Add an
observer to the already executed verified-inverse calls. The first apparatus
run exposed two such calls although only one is retained in final
`inverse_audits`; require exactly two captured call roots and a unique match to
the frozen R36 inverse root. Capture full matrix/factor/inverse payload only for
that selected 65-column audit. Require the parent case, step, inverse and norm
roots. The observer may not execute an additional factorization or
right-hand-side solve.

Represent every finite binary128 operand canonically as signed
`integer * 2^exponent`. With arbitrary-size integers, compute each entry of

```text
R = I - A X
```

and each row sum `sum_j abs(R_ij)` exactly. Select the largest exact row by
value, then lowest row index. Publish the exact norm/root, worst row, outward
binary128 value, exact comparison with one, and current R36 norm reproduction.

The dyadic apparatus must pass predeclared identity, diagonal power-of-two,
cancellation and noncontractive controls, exact binary128 operand round-trip,
and a deterministic permutation repeat. Every exact residual must lie inside
the corresponding reported outward conversion.

Routes in precedence order:

1. `EXACT_INVERSE_PARENT_REJECTED`;
2. `EXACT_DYADIC_APPARATUS_REJECTED`;
3. `EXACT_INVERSE_CAPTURE_REJECTED`;
4. `INVERSE_CANDIDATE_NONCONTRACTIVE` when exact norm `>=1`;
5. `ARITHMETIC_ENCLOSURE_DOMINATES` when exact norm `<1`.

R37 executes one parent torsion solver replay and its existing 65 inverse
columns, zero additional inverse/factor/solve, zero counterflow replay, trial,
state update, tolerance/cap/Armijo change or timing. It cannot install Dot2,
change the inverse audit, continue torsion, refine counterflow, claim runtime
or production readiness.
