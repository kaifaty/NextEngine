# NSR3-B4E2D7R20R20 projector-breakpoint geometry evidence

Status: `PASS / SIMPLE_BREAKPOINT_OFFSET`.

Implementation `4ca6de6e` preserves R19 semantic exactly and emits:

```text
4ca89c1ffa825d8a09422f460c40886b7e5e60ac9992602726eb7d0a5dcc1de5
```

All 455 fixed samples and all 14 existing endpoints pass exact evaluation and
root parity. Every one of the seven brackets contains:

- exactly one projector-mask transition;
- exactly one Armijo sign transition;
- no mask return and no ball-activity change;
- exactly one changed scalar;
- no mask-stable sample with a negative Armijo margin.

The mask/Armijo transition cells are `12/14`, `39/44`, `15/21`, `4/4`,
`7/7`, `56/59` and `50/54`. Thus two brackets are grid-aligned and five have
a small crossing region that remains rigorously Armijo-positive. The changing
scalar is 168 in steps 25--27 and 189 in steps 29--32; the accepted crossing
at step 28 separates those two locally persistent events.

This excludes both a hidden same-face rejection layer at the frozen resolution
and a multi-event path inside the local brackets. It also shows why simply
stopping immediately before the boundary would be unnecessarily conservative:
in five cases an admissible interval exists on the new face. The next smallest
research step is to derive the boundary directly from the current fixed-face
projector KKT equations and validate that predictor against all seven observed
cells before it is allowed to generate a solver trial.

No sampled multiplier was applied. Solver iterations, cap, tolerances and all
runtime/production authority remain unchanged.

