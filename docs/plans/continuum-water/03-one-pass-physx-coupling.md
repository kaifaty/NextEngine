# W3 — One-pass PhysX coupling

## Outcome

Prove a one-pass, two-way water/rigid substep in which CPU water owns only its
region state, PhysX remains the sole writer of rigid state and the composite
PhysicalStep publishes completely or not at all.

## Fixed substep

For each 240 Hz substep:

1. bind the ADR-081 namespace/owner/world/revision/root/tick/substep/edge/
   participant/operation identity tuple and exact rigid canonical snapshot;
2. freeze the selected rigid body poses, linear/angular velocities, centre of
   mass and boundary material mapping;
3. solve the water candidate against analytical static and moving primitive
   boundaries;
4. accumulate sample reactions in stable
   `(PhysicsBodyIdV1, feature key, SampleId)` order;
5. reduce one fixed-point linear/angular impulse record per body;
6. validate the complete water candidate and reaction batch;
7. let the PhysX adapter apply the batch and integrate exactly once;
8. validate the resulting rigid projection and publish both owner states as
   one PhysicalStep transaction.

There is no fixed-point iteration between solvers, delayed application on the
next substep or runtime async-completion admission. Device execution may be
asynchronous internally in a future mirror, but the authoritative logical
step is synchronous at `PhysicalStep`.

## Implementation-local reaction record

```text
WaterBodyReactionV1 {
  world_id,
  expected_world_revision,
  expected_rigid_snapshot_root,
  expected_body_revision,
  physics_tick,
  substep,
  region_id,
  region_revision,
  water_profile_hash,
  prior_water_state_root,
  body_id: PhysicsBodyIdV1,
  linear_impulse_fixed[3],
  angular_impulse_fixed[3],
  torque_reference_com_fixed[3],
}

WaterBodyReactionBatchV1 {
  key: (world_namespace, source_owner_id, destination_owner_id, world_id,
        expected_source_revision/root, expected_destination_revision/root,
        physics_tick, substep, edge_profile_id, region_id, operation_slot),
  sorted_reactions[],
  resulting_water_state_root,
  batch_root,
}
```

The lab record stays private. Maximum coupled bodies for the selected V1
fixture is `32`; `N-1/N/N+1` tests enforce it. Exactly one record exists per
body after reduction. Exact duplicate batch keys and conflicting roots under
one key both return a typed collision and apply no impulse.

Linear/angular impulses use existing SPEC-26 fixed-point numeric/unit rules.
The torque reference is the frozen canonical body centre of mass; an implicit
application point or backend-local origin is forbidden.

## Boundary material lineage

Keep `PhysicsMaterialDescriptorV2` unchanged. The lab uses a private exact
mapping from rigid shape/material revision to a water-boundary response
profile. A production consumer later promotes the smallest equivalent
engine-owned mapping; missing or stale mapping fails before the substep.

## Scenario corpus

- sealed static basin leak/penetration regression;
- moving piston and rotating paddle under prescribed rigid motion;
- the selected `0.5 m`, `50 kg` crate float equilibrium;
- crate drop/impact at frozen low and high velocities;
- offset centre-of-mass body for angular impulse sign/reference;
- two bodies sharing the region and reduction ordering;
- stale world/body/water revision, missing body, duplicate/conflicting batch,
  capacity, nonfinite, non-convergence and PhysX rejection.

Reaction closure compares the negative water boundary impulse with the exact
batch before PhysX conversion. Post-step momentum/energy diagnostics account
for gravity, prescribed body work and PhysX integration; their normalized
residual must remain `<= 1%`.

## Failure and checkpoint semantics

Any water or PhysX fault rejects the complete composite candidate, retains the
previous complete physical generation and stops the run with the first stable
diagnostic. The implementation cannot commit PhysX while retaining old water,
commit water without PhysX, retry with fewer iterations, freeze water or load
the dry fallback after activation.

The runtime composition profile owns the complete DAG and merges every rigid
input before one PhysX integration. Worst-case batch/participant capacity is
admitted before freeze; post-freeze exhaustion beyond reservation is an
invariant. A primary-gameplay fault uses the ADR-081 whole-session domain.

A private composite lab checkpoint is allowed for continuation evidence. It
must contain exact water state plus the existing engine-owned rigid canonical
checkpoint closure; native PhysX serialization and independent sidecars are
forbidden.

## Exit

`CONTINUUM-COUPLING-P1 = PASS` requires all positive/failure scenarios,
one-pass closure, fixed-point batch/root repeatability and atomic checkpoint
continuation. W3 does not itself authorize a public reaction contract or
production runtime integration.

## Non-goals

Multiple water regions, iterative coupling, articulation-fluid interaction,
generic gameplay queries, terrain, public schemas, presentation quality and
GPU authority.
