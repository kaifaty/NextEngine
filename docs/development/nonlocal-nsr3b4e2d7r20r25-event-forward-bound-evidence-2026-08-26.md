# NSR3-B4E2D7R20R25 event-input forward-bound evidence

Status: `PASS / EVENT_FORWARD_BOUND_CANDIDATE`.

Implementation `87dab448` reproduces R24 semantic exactly and emits:

```text
d3544d4255d066245d6055b71baa56e67d70e44fce001ba1da83844a4e62d449
```

All seven local bounds contain all 455 actual-versus-affine projector-input
differences. Containment, certified-side and suffix rejection counts are zero.
Each predicted component has exactly 12 contributing sparse transpose entries.

The derived bounds are `9.24e-33` for scalar 168 and `1.12e-32` for scalar
189. Observed maximum discrepancies are only `4.35e-36..9.81e-36`, but those
observations did not enter the bound. The first powers whose analytic input is
strictly above the bound are:

```text
22, 25, 26, 15, 19, 23, 24
```

From every such power through 64, the actual projector stays on exactly the
predicted new face and every rigorous Armijo margin remains positive. Thus the
componentwise forward bound owns a stable-side implication on the frozen
events. It is not yet solver policy and remains binary128 research evidence.

