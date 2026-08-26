# NSR3-B4E2D7R20R32 terminal certificate trajectory contract

Status: `FROZEN / ONE-SHOT 12-CASE TERMINAL TRAJECTORY AUTHORIZED`.

## Parent

- R31 implementation `7bb3c886`, semantic
  `3425772c1e7480d690a31b52e2456857b17feb15f9967892e3830b01c907de5b`;
- exact R31 power-0 trial root `e36e8d5e...26d7` and metrics root
  `05b71f6f...9b53`;
- R29 semantic `3c4f5d51...ba7b`, shear root `3f4798c8...e473` and exact
  21-step accepted trajectory with one exhausted-line recovery;
- exact KKT tolerance `2^-70`, Armijo rule and 32-step cap remain unchanged.

## Frozen candidate

Execute the complete ordered 12-case R29 corpus under a separate default-false
candidate. Preserve every ordinary line-search decision. Only when all 21
existing trials reject may the candidate scan those already evaluated trials
in inherited power order for the first complete `metrics.certified` result.

The sole permitted intervention is exact R29 shear iteration 22. Require
power 0, `alpha=1`, exact R31 dual/metrics/trial roots, exact same-face and
same-ball lineage, a negative rigorous Armijo margin, and bit-consistent
recomputation of every finite/primal/dual/complementarity/stationarity/gap
predicate. Select that trial's existing primal and multiplier state, run one
fresh KKT audit, classify the exit as terminal certificate rather than Armijo
acceptance, and execute no following iteration.

PASS requires 12/12 certification, exact R29 roots for all eleven non-shear
cases, exact shear prefix/recovery/final-trial correspondence, one terminal
selection and zero other trajectory changes. The report must distinguish
terminal success, no certified rejected trial, correspondence failure and any
historical regression.

No new trial, second recovery, ordinary Armijo/tolerance/cap change, search
extension, timing, runtime/GPU, generalization or production authority.
