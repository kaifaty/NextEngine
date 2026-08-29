# NSR3-B4E2D7R19R41 -- support-crossing and contact audit contract

Date: `2026-08-24`

Status: `FROZEN / IMPLEMENTATION NEXT / ROLLBACK ONLY`.

Parent: `52eb514d`, R40 stdout SHA-256
`6db946e6c6e327520b60f98d34e60f346f0b02bfe84e88767fc1e36d7db9489d`,
semantic `d8ea97108543603e511c68632a8993eb010f0a7f8342f034a55c8d53555e98ff`.

## Frozen identity

```text
nextengine.nonlocal.nsr3b4e2d7r19r41-support-crossing-contact-audit|v1|parent=52eb514d:6db946e6c6e327520b60f98d34e60f346f0b02bfe84e88767fc1e36d7db9489d:d8ea97108543603e511c68632a8993eb010f0a7f8342f034a55c8d53555e98ff|source=r40-route-nonlinear-topology-rejected;position-e0f7ba79637171012b11750b752ab862b7b24174f4d79e2ec115e06bc8b93828;trial-60e793aff919fada6b49c1b98be7ee5cc4a2c74a4254fa762cdece5700e1e804;endpoint-a73caf5baa6708ca89693077bad943dcdbf1f5069d50270ccc0f6b184a102e3a;physical-step-424c84b72409f5b6665874f17afc18cf2784ae84593e30be41eaeec0d25e514b;current-topology-070ab5209d8b32c942a244d9d5ae5442baba6f281c9010f65a5f0d1cf184c2e5;trial-topology-25d6148fc3f460ba6c324a539499deebabb86e88267ab6bdf3fe700dc44c7402;current-pairs-58ce6532f180f9f2803a069f8cfeb25278fbc3930796d7cf29700db08ce4fac3;trial-pairs-2e74673c3ae59c099c91902d7eac31b97b1c5a9496497128174d9a899f2bae06|crossings=exact-sorted-pair-symmetric-difference;lost;entered;fluid-fluid;fluid-support;source-and-trial-radius-bits;signed-horizon-margin;binary64-ulp-distance;weight;gradient;second;density-delta;record-root|shell=material-zero-only-if-both-state-weight-gradient-second-exact-zero-and-crossing-density-delta-exact-zero;no-fitted-tolerance|contact=all-particle-axis-lower-upper;source-and-trial-penetration;new;resolved;worsened;normal-step-component;worst-owner;record-root|selection=nonzero-support-crossing-first;new-contact-second;existing-contact-inward-third;material-topology-equivalence-last|controls=exact-horizon-zero;interior-to-outside-nonzero;existing-contact-worsened;new-contact;route-precedence;parent;source;mapping;workspace;crossing-partition;kernel-fold;density-ledger;contact-ledger;work;rollback|routes=crossing-parent-rejected;crossing-source-rejected;crossing-dense-rejected;crossing-workspace-rejected;crossing-partition-rejected;crossing-kernel-rejected;crossing-contact-ledger-rejected;crossing-work-rejected;nonzero-support-crossing-requires-relinearization;new-contact-crossing-requires-contact-solve;contact-tangent-normal-step-required;material-topology-equivalence-candidate|precedence=parent,source,dense,workspace,partition,kernel-ledger,contact-ledger,work,nonzero-crossing,new-contact,existing-contact-inward,material-equivalence|work=parent-r40-replays1;diagnostic-workspaces2;releases2;maximum-live2;crossing-merges1;contact-particle-axis-tests36000;new-jvp0;new-vjp0;new-hvp0;new-model0;new-trial0;new-precision0;new-outer0|runs=2-clean-release-builds;1-process-each;byte-exact|r40-gates=unchanged;correction=none;candidate-commit=none;contact-projection=none;support-expansion=none;tolerance=none;public-state-mutation=none;timing=none;runtime=none;production=none|credit=one-private-support-crossing-contact-classification-only
```

SHA-256:
`7abcc0e2a406580265a84dec051fcb5745ccf1cbb76c9cfd5ac57136fc32a3a2`.

## Hard gates

1. Exact R40 parent bytes/semantic route and exact source, trial, endpoint,
   physical-step, topology and pair-membership roots.
2. Dense exact-horizon zero, interior nonzero, existing-contact-worsened and
   new-contact controls, plus every route in frozen precedence.
3. Exactly two fresh normalized workspaces reproduce both R40 topology and
   pair roots.
4. One ordered merge partitions the pair symmetric difference exactly into
   lost/entered and fluid/fluid-support records; intersection plus both
   partitions reconstruct both parent pair counts.
5. Every crossing records source/trial radius bits, signed horizon margin,
   positive binary64 ULP distance, `W/W'/W''`, density delta and a complete
   record root. Direct recomputation must match workspace-owned member radii.
6. Material-zero shell requires exact zero for both-state `W/W'/W''` on every
   crossing and an exactly zero per-center crossing-density ledger. No
   numerical tolerance is admitted.
7. Exactly 36,000 ordered contact face tests root current/trial penetration,
   normal step, new/resolved/worsened sets and worst owner. Direct maximums
   reproduce R40 values exactly.
8. Work is one R40 replay, two workspace builds/releases, maximum live two,
   one pair merge, 36,000 face tests and zero new JVP/VJP/HVP/model/trial/
   precision/outer work. Source/trial/endpoint rollback is exact.

Require two clean Release builds and byte-exact outputs. Do not change R40,
expand support, project contact, form another trial, select a tolerance, mutate
state, time the solver or claim runtime/production authority.

Rationale:
[R41 research](../../development/nonlocal-nsr3b4e2d7r19r41-support-crossing-contact-research-2026-08-24.md).
