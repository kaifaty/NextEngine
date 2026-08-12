# CPU/Isaac correspondence contract

The canonical plane is the engine-owned CPU PhysX/headless route. Isaac Lab is
an optional accelerated mirror. Correspondence admits a bounded use of the
mirror; it does not transfer replay authority to GPU simulation.

## Version and identity closure

Read `lab/profiles/isaac-lab-physx-stage0.v1.json` for the active pins. The
auditor intentionally contains matching pins and fails when the tracked profile
changes, forcing this skill and its release-specific advice to be reviewed
together.

Before simulation, close:

1. generation ID, canonical embedded hash and admitted inputs;
2. environment profile file hash and exact BodySchema identity;
3. compiled descriptor file hash and body/compiled-descriptor fields;
4. derived USD file hash;
5. corpus embedded manifest hash and TRAIN-4 gate when reference data is used;
6. tracked Isaac Lab, Isaac Sim, engine PhysX, Python and backend versions;
7. clean repository commit for newly generated evidence.

An upstream Isaac example from another release is design input only. In
particular, advice for a newer Isaac Sim major or for direct-torque actuation
does not override the tracked profile.

## MODEL-MIRROR-P1

The current repository producer evaluates at least 256 episodes with at least
600 motor steps per episode.

Exact fields are command bytes and every declared profile/layout/hash identity,
including reward-component order. Exact mismatch is a lineage/schema defect,
not a numeric tolerance problem.

| Tolerant metric | Pass condition |
| --- | ---: |
| joint position RMSE | `<= 0.02 rad` |
| root position RMSE | `<= 0.03 m` |
| root velocity RMSE | `<= 0.05 m/s` |
| contact occupancy agreement | `>= 0.98` |
| done-tick agreement | `>= 0.95` |
| reward total MAE | `<= 0.05` |

Audit the report against both original trajectory files. A report whose input
hashes were not rechecked is useful for inspection but not promotion-ready.

## MODEL-MIRROR-P2 for the reference tracker

The biomechanics reference environment needs more than the current P1 scalar
envelope. P2 must close the exact descriptor generation from BodySchema V2 and
correspondence for:

- reference clip selection, frame ordinal and phase;
- ordered reference observation features and transforms;
- ordered Q16 reward components and total;
- termination/truncation facts and reason;
- post-safety applied targets at the action-channel/motor-tick boundary.

The repository does not yet expose a product-owned P2 report producer/schema.
Until it does, report P2 as `NotRun` or unsupported. Never accept a hand-edited
`check: MODEL-MIRROR-P2` label, reinterpret a P1 report, or substitute the
reference sanity smoke.

## Failure routing

| Failure | Inspect first |
| --- | --- |
| exact ID/hash/order | stale artifact, translator, feature order, schedule/profile binding |
| joint/root drift | axes, frames, inertia, solver, drive parameters, action timing |
| velocity drift | sampling cadence, frame transform, substeps, committed tick |
| contact drift | collider roles, filters, material, sole definitions, contact cadence |
| reward/done drift | same committed facts, Q16 order/rounding, termination versus truncation |
| applied-target drift | residual scale, safety projection, PD ownership, channel permutation |
| non-finite/sample floor | reject the corpus; do not post-align or discard frames |

For direct-torque profiles, imported drives must not oppose the agent. The
current reference tracker is different: its action is reference target plus an
actuator-owned residual, followed by engine safety and fixed PD. Those drives
are part of the frozen action contract.
