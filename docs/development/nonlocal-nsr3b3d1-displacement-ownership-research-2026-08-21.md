# NSR3-B3D1 displacement-ownership research -- 2026-08-21

Status: `COMPLETE / OWNED_SUBSTEP_DISPLACEMENT_SELECTED_FOR_TEST`

## Observation

B3D proves two independent facts:

1. active reaction accuracy can be recovered cheaply by an impulse-aware
   nonlinear stop;
2. inactive reconstruction remains correctly bounded, but its conservative
   error certificate grows like `|x|/h` when velocity is recomputed from two
   nearby world positions.

The second effect is a numerical representation problem, not continuum
physics or contact order.

## Alternatives

### Keep `(y-x)/h` and relax the cumulative bound

Rejected. The bound is doing its job: it exposes worsening conditioning under
refinement. Relaxing it would make a finer oracle less trustworthy.

### Use extended precision for the report oracle

Rejected as the selected direction. It would not transfer to the eventual GPU
path and would hide the representation issue rather than remove it.

### Shift to local world origins

Potentially useful later, but origin choice would become another state/cache
boundary and still reconstruct a difference after rounding both endpoints.

### Own the accepted displacement

Selected for a bounded test. Prediction already creates `delta*=h*v*`; every
accepted trust step is itself a displacement. Accumulating those values avoids
throwing the increment away and subtracting positions to recover it.

Contact operates on the same displacement: for a lower face hit, the accepted
normal component is `delta_n=(a+R)-x_n`; tangential components retain their
smooth values. The accepted position is then rebuilt once as `x+delta` and
velocity is `delta/h`.

## Ownership rule

Within one report-only substep:

```text
delta owns the transition increment
y is a reconstructible objective-evaluation coordinate
v_next is derived from delta
```

The displacement is transient and is discarded after the committed
position/velocity pair. This does not add public state or a second durable
authority. Any later runtime promotion would require a separate architecture
decision about floating origin and replay semantics.

## Decision

Execute the frozen
[B3D1 contract](../plans/nonlocal-nonlinear-solver-research/03b3d1-displacement-ownership-contract.md).
Only if displacement ownership closes the inactive certificate while retaining
the B3D active reaction-aware result may a B3R composition retry be designed.
