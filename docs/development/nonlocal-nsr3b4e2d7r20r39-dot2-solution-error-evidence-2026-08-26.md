# NSR3-B4E2D7R20R39 Dot2 solution-error evidence

Status: `PASS / DOT2_SOLUTION_SIGN_UNRESOLVED / 59 OF 65 RESOLVED`.

Implementation `9ade7a70` emits reproducible semantic:

```text
83e1f7387a1ff2675d3899af904666734fd6b71f3e3079b34617d46f63d21bac
```

R38 and every captured parent root reproduce. The selected RHS/solution roots
are `464263ae...42f8` and `dbf5c26c...b26e`; their exact residual root is
`8a0e41dd...79c4`.

| field | value |
|---|---:|
| stored solve residual / old arithmetic bound | `9.06689e-18 / 1.87140e-14` |
| exact residual infinity norm | `7.134357676811010e-18` |
| Dot2 residual infinity bound | `7.134357676811010e-18` |
| represented inverse norm | `1.442926194748943e33` |
| certified inverse norm bound | `1.449366651428182e33` |
| old cheap error | `7.109566050753423e38` |
| new uniform solution error | `1.034030009613052e16` |
| old/new improvement | `6.875589668247558e22` |
| minimum absolute solution component | `1.054228865543863e3` at row 59 |
| signs | `24 positive / 35 negative / 6 unresolved` |
| solution audit root | `19a90db0...4713` |

All 65 exact residual components are contained without underflow. Maximum
actual Dot2 error is `9.45e-51`, only `2.96e-4` of its issued bound. Residual
arithmetic is therefore not the remaining blocker.

The failure comes from applying a worst-case global inverse norm to this
specific residual. It improves the legacy estimate enormously and resolves 59
signs, but remains thirteen orders of magnitude wider than the smallest
component. R39 does not establish whether the six signs are genuinely
ill-conditioned or merely lost by the direction-agnostic norm.

Two repeats are exact and R38 remains `6898dcbe...06ae`. R39 executes no new
solve/factorization/inverse column or NNQP decision. It selects only a
directional `Xr`/left-defect discriminator.
