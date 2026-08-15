# 02 — Boundaries and rigid coupling

## Outcome

Add leak-resistant static/moving boundaries and prove one two-way water–rigid
body vertical without creating a second rigid-state writer.

## Boundary profiles

Start with analytical plane, box, sphere and capsule signed-distance
boundaries. Each has stable boundary/body identity, exact transform input and
finite velocity. Add a sampled density-map or MLS boundary only after the
analytical corpus passes and authored curved geometry demonstrates the need.
Boundary preprocessing is immutable content/cache, bounded and hash-identified.

## Coupling contract inside the lab

For each substep, freeze a quantized/exact rigid projection, solve fluid
constraints, accumulate each sample-boundary reaction, and reduce by
`(body PersistentId, boundary feature key, SampleId)` into:

```text
BodyReaction { body_id, linear_impulse, angular_impulse, application_epoch }
```

This is an implementation-local record until a production consumer exists.
The PhysX adapter validates the closed batch and remains the only writer of
body transforms/velocities. A fixed profile selects one, two or four coupling
iterations; wall time never selects the count.

## Scenarios and failures

- static tank leak and thin-feature tunneling;
- moving piston and rotating paddle;
- neutrally buoyant/offset-center floating body;
- body dropped into water at low and high speed;
- two bodies sharing boundary particles/features;
- stale body revision, missing body, capacity overflow and nonfinite impulse.

The whole coupled substep publishes or nothing does. Unknown/stale body input,
reaction collision, conservation violation or PhysX rejection retains the
prior physical generation and returns a stable diagnostic.

## Exit

`CONTINUUM-COUPLING-P1` passes bounded mass and linear/angular momentum error,
penetration/leak thresholds, repeat/worker permutations and a saveable lab
checkpoint continuation. Production integration still requires an Accepted
ADR narrowing ADR-058 and a consumer-backed public record.
