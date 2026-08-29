# NSR3-B4E2D7R19R50 -- active-face closure contract

Date: `2026-08-25`

Status: `PASS / ACTIVE_FACE_HILDRETH_CLOSURE_CANDIDATE / CLOSED /
ROLLBACK ONLY`.

Parent: `8789cc16`, R49 stdout SHA-256
`7e1dde137b6164f67a448ab704eb713148e08c3a3a6e693614cb2645b01fa656`,
semantic `bbdfae6fbd301b74dbdbfde8bc416f814cf7e06ec74b8b54c8abd7718dac6544`.

## Frozen identity

```text
nextengine.nonlocal.nsr3b4e2d7r19r50-active-face-hildreth-closure|v1|parent=8789cc16:7e1dde137b6164f67a448ab704eb713148e08c3a3a6e693614cb2645b01fa656:bbdfae6fbd301b74dbdbfde8bc416f814cf7e06ec74b8b54c8abd7718dac6544:RESTORATION_HZ_PRIMAL_ACCELERATOR_CANDIDATE|source=r49-witness-7716e30c95b802a36f8c7d62dc993c5d912e678ec4fcb1617e364fba7898ded5;checkpoint-b19e6e64eaf68e9bd8eccf748d89cda661b42850118d355846b92f329975caba;steps-ec1fcee08dd724d98b0ce4e5506fdc75e90d654e122f65dcb2afcb8436aaade7;active366;maximum-upper1.9771445876661678e-11|problem=directed-active-majorant-minimum-norm-halfspace-projection;unchanged-r48-A-contact-box-normal-ball;working-rows=current-directed-upper-positive;u=current-directed-upper;B=restrict-working-A;min0.5*norm2p;u+Bp<=0|capacity=unique-row-cache512;next-power-of-two-above-parent366;overflow-reject;never-truncate|basis=row-gradient=At-ei-one-vjp;gram-column=A-row-gradient-one-jvp;cache-fixed-operator-across-outers;positive-diagonal;relative-symmetry<=1e-12|solver=hildreth-dual-coordinate;lambda0=0-each-outer;correction0;stable-ascending-row-order;coordinate=lambda-new=max(0,lambda+predicted/diagonal);exact-eight-sweeps;checkpoints1,2,4,8;no-tolerance-stop|globalization=target=current+correction;project-r48-ball-box;feasible-chord;dyadic-alpha1-through2^-8-largest-first;fresh-directed-audit-each;accept-first-primal-certified-or-strict-psi-h-maximum-upper-decrease|max-outers4;rebuild-positive-upper-active-set-after-accept;no-fifth-outer|certificate=unchanged-r48-fresh-all-row-directed-jvp;zero-certified-positive-only;parent-dual-unchanged;no-infeasibility-claim|selection=certificate-first;else-active-face-closure-candidate-if-at-least-one-accepted-outer-and-terminal-strict-psi-h-maximum-upper-dominance;else-reference-retained|controls=r49-parent-bytes;dense-hildreth-projection;dense-dual-release;dense-active-addition;dense-capacity;dense-symmetric-gram;dense-dyadic-globalization;directed-rounding;source;workspace;geometry;active-cache;gram;hildreth;projection;globalization;certificate;work;rollback;route-precedence|routes=active-face-parent-rejected;active-face-source-rejected;active-face-workspace-rejected;active-face-geometry-rejected;active-face-capacity-rejected;active-face-gram-rejected;active-face-hildreth-rejected;active-face-projection-rejected;active-face-globalization-rejected;active-face-primal-audit-rejected;active-face-work-rejected;restoration-next-trqp-compatible-candidate;active-face-hildreth-closure-candidate;active-face-reference-retained|precedence=parent,source,workspace,geometry,capacity,gram,hildreth,projection,globalization,primal-audit,work,certificate,selection|work=parent-r49-replays1;static-index1;superset-build1;moved-workspace1;releases1;unique-rows<=512;row-vjp<=512;gram-jvp<=512;initial-directed-jvp1;candidate-directed-jvp<=36;pairpasses<=1061;outers<=4;sweeps<=32;projection-calls<=4;new-support-audits0;new-hvp0;new-model0;new-nonlinear-trial0;new-outer0|runs=2-clean-release-builds;1-process-each;byte-exact|r40-r41-r42-r43-r44-r45-r46-r47-r48-r49=unchanged;normal-witness-apply=none;r43-restoration-commit=none;filter-runtime-commit=none;restoration-exit=none;switching=none;trust-update=none;following-outer=none;tolerance=none;penalty-change=none;support-expansion=none;state-mutation=none;timing=none;runtime=none;production=none|credit=one-private-active-face-closure-classification-only
```

SHA-256: `ab745dfcfede239ed739d5bec80f1bc6c9e74be36b3bd5daa6887f830f263c75`.

## Hard gates

1. Reproduce exact R49 bytes, semantic, route, witness/checkpoint/step roots,
   source operator, contact/trust geometry and rollback.
2. Run a fresh directed audit at the exact R49 witness. Working rows are exactly
   `upper>0`; no residual tolerance or row truncation is admitted.
3. Cache at most 512 unique rows. Each row owns one basis VJP and one Gram-column
   JVP under the fixed operator. Overflow rejects before partial selection.
4. Require finite row data, strictly positive Gram diagonal and relative
   symmetry at or below `1e-12`. A Gram failure cannot be reclassified as
   unresolved closure.
5. Run exactly eight stable-order Hildreth sweeps per executed outer, with
   nonnegative dual coordinates and checkpoints `1/2/4/8`. No fitted convergence
   tolerance or early sweep stop is allowed.
6. Project the full correction with the unchanged R48 Euclidean ball-box
   projector. Every dyadic candidate lies on the resulting feasible chord.
7. Audit `alpha=1,1/2,...,2^-8` largest first. Accept only the first independently
   certified point or the first strict simultaneous decrease in `psi`, `h` and
   maximum directed upper. If none passes, stop exactly.
8. Rebuild the positive directed active set after each accepted outer. Stop on
   certificate or after four outers; no fifth retry exists.
9. Compatibility requires zero positive all-row directed uppers. Hildreth dual
   stationarity, predicted halfspace feasibility or raw hinge is insufficient.
10. Exact work, release and rollback. No correction/state/filter commit,
    restoration exit, trust update, following outer or timing.

Require two clean Release builds and byte-exact outputs. Any PASS is one private
active-face closure classification only, not a completed restoration transaction,
runtime permission or production evidence.

## Closed result

R50 closes `PASS / ACTIVE_FACE_HILDRETH_CLOSURE_CANDIDATE`. Four accepted
outers and 991 pair passes lower `h` to `2.9894663306152676e-12` and maximum
directed upper to `1.0297972392379541e-12`, but 488 rows remain positive and
the cache reaches 492/512. Both clean Release binaries and outputs are
byte-exact; see the
[dated evidence](../../development/nonlocal-nsr3b4e2d7r19r50-active-face-closure-evidence-2026-08-25.md).

Rationale:
[R50 research](../../development/nonlocal-nsr3b4e2d7r19r50-active-face-closure-research-2026-08-25.md).
