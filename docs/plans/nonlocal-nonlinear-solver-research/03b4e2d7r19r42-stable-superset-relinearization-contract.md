# NSR3-B4E2D7R19R42 -- stable-superset relinearization contract

Date: `2026-08-24`

Status: `FROZEN / IMPLEMENTATION NEXT / ROLLBACK ONLY`.

Parent: `c0fa38fb`, R41 stdout SHA-256
`0237a380e86732e1ef78c50ca390ef2f93b1a82ac47338488403d9c5bb00d81c`,
semantic `3462fdcbc39d821c51fd4fdfbf9d9141ca7fd192fa53e9fb2885162c110f2ad8`.

## Frozen identity

```text
nextengine.nonlocal.nsr3b4e2d7r19r42-stable-superset-relinearization|v1|parent=c0fa38fb:0237a380e86732e1ef78c50ca390ef2f93b1a82ac47338488403d9c5bb00d81c:3462fdcbc39d821c51fd4fdfbf9d9141ca7fd192fa53e9fb2885162c110f2ad8|source=r41-route-nonzero-support-crossing-requires-relinearization;position-e0f7ba79637171012b11750b752ab862b7b24174f4d79e2ec115e06bc8b93828;trial-60e793aff919fada6b49c1b98be7ee5cc4a2c74a4254fa762cdece5700e1e804;physical-step-424c84b72409f5b6665874f17afc18cf2784ae84593e30be41eaeec0d25e514b;current-topology-070ab5209d8b32c942a244d9d5ae5442baba6f281c9010f65a5f0d1cf184c2e5;trial-topology-25d6148fc3f460ba6c324a539499deebabb86e88267ab6bdf3fe700dc44c7402;current-pairs-58ce6532f180f9f2803a069f8cfeb25278fbc3930796d7cf29700db08ce4fac3;trial-pairs-2e74673c3ae59c099c91902d7eac31b97b1c5a9496497128174d9a899f2bae06|ownership=one-source-anchored-canonical-verlet-superset;skin=0.04h=0.006;list-radius=h+skin=0.156;physical-h-unchanged;exact-r<=h-state-mask;lexicographic-pairs|certificate=actual-max-fluid-displacement-squared;4*dmax2<=skin2*(1-2e-12);analytic-global-l2-upper-bound;static-support-fixed|coverage=ordered-current-and-trial-subset;state-mask-roots;no-missing-pair;candidate-degree-cap|correspondence=independent-canonical-current-and-trial;filtered-pairs;flat-csr;radii;weight-first;weight-second;density;constraint;bit-exact|trial-operator=Atrial=spacing*Jc-at-trial;fresh-trial-radii-and-weight-first;pair-once-jvp;directed-jvp;superset-centered-finite-difference-epsilon2^-16;vjp-adjoint;constant-translation-interior|bounds=pair-directed-componentwise-gamma(16*max-degree+64);finite-difference-relative-l2<=1e-6;adjoint-relative<=1e-12;translation-interior-exact-zero|residual=trial-psi;violation;active;zero-origin-mapping=norm(AtrialT*max(cTrial,0));no-stationarity-tolerance|attribution=deterministic-current-trial-jvp-difference;nonzero-not-gate|controls=certificate-pass-fail;missing-pair;correspondence-mutation;operator-mutation;route-precedence;parent;source;dense;binding;superset;certificate;coverage;correspondence;operator;work;rollback|routes=stable-superset-parent-rejected;stable-superset-source-rejected;stable-superset-dense-rejected;stable-superset-binding-rejected;stable-superset-build-rejected;stable-superset-certificate-rejected;stable-superset-coverage-rejected;stable-superset-correspondence-rejected;trial-relinearization-rejected;stable-superset-work-rejected;stable-superset-relinearization-candidate|precedence=parent,source,dense,binding,superset,certificate,coverage,correspondence,trial-operator,work,candidate|work=parent-r41-replays1;static-index1;superset-builds1;canonical-workspaces2;masked-workspaces2;workspace-releases4;maximum-live2;filters2;jvp-pair2;jvp-directed1;vjp1;finite-difference-superset-evaluations2;translation-jvp1;new-hvp0;new-model0;new-trial0;new-precision0;new-outer0|runs=2-clean-release-builds;1-process-each;byte-exact|r40-r41-gates=unchanged;support-expansion=none;contact-projection=none;correction=none;candidate-commit=none;tolerance=none;state-mutation=none;timing=none;runtime=none;production=none|credit=one-private-stable-superset-trial-relinearization-classification-only
```

SHA-256: `14ad489cbf6985aea25f7c697e2cb6cb27c979883a2a27898331c96d6dcd78df`.

## Hard gates

1. Exact R41 parent bytes, semantic and selected route plus exact R40 source,
   trial, physical-step, current/trial topology and pair roots.
2. Dense controls exercise certificate pass/fail, missing-pair detection,
   correspondence/operator mutation and every frozen route in precedence.
3. One canonical source-anchored `H+0.04H` superset passes capacity and the
   exact B4EP3 displacement certificate using the actual R40 maximum fluid
   displacement. The physical horizon remains bit-exact `H`.
4. Ordered inclusion covers every current and trial active pair exactly. Root
   the complete superset and both state-local active masks; no observed pair
   union substitutes for the coverage certificate.
5. Superset-filtered current and trial workspaces independently reproduce
   canonical pairs/order, flat CSR, radii, `W'`, `W''`, density and constraints
   bit-for-bit.
6. Trial pair-once and directed JVPs satisfy the inherited componentwise
   rounding bound. Exact-superset centered finite difference at `2^-16` has
   relative L2 error at most `1e-6`; JVP/VJP adjoint error is at most `1e-12`;
   fixed-support translation is exactly zero on interior rows.
7. Report trial hinge objective, violation, active rows and zero-origin
   projected mapping with deterministic roots. No observed value becomes a
   stopping tolerance. Current/trial deterministic JVP difference is
   attribution only.
8. Work is one R41 replay, one static index and superset build, two canonical
   plus two masked workspace builds/releases with at most two live, two
   filters, exactly the frozen JVP/VJP/finite-difference passes and zero new
   HVP/model/trial/precision/outer work. Rollback is exact.

Require two clean Release builds and byte-exact outputs. Do not change R40 or
R41 gates, expand physical support, project contact, alter or apply the normal
step, accept the trial, select a tolerance, mutate state, time the solver or
claim runtime/production authority.

Rationale:
[R42 research](../../development/nonlocal-nsr3b4e2d7r19r42-stable-superset-relinearization-research-2026-08-24.md).
