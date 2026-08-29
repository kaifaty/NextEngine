# NSR3-B4E2D7R19R35 -- equal-work generalized-Hessian contract

Date: `2026-08-24`

Status: `FROZEN / REPORT ONLY`

Parent: `7db0857e`, exact R34 stdout SHA-256
`5150f1f697072a21a81ce7d6fc7aa4548ce603e7e27796143483fa0ad796f345`,
semantic result
`00dcf602a9266d8170a8c84af1f338903650f4c767a24d7cf00249a9837583b7`.

## Frozen identity

```text
nextengine.nonlocal.nsr3b4e2d7r19r35-equal-work-generalized-hessian|v1|parent=7db0857e:5150f1f697072a21a81ce7d6fc7aa4548ce603e7e27796143483fa0ad796f345:00dcf602a9266d8170a8c84af1f338903650f4c767a24d7cf00249a9837583b7|source=v8-from-r33;prefix-response-7dd519932782f9177be354418c9ecaaa1fed3baac1e8255305675da7cb59dff3;baseline-checkpoint-35db4ad52c1fbd8d99e4ada888ecc5589df1cdfb39e3f9a73cd1323494326dc3;baseline-response-ed311a3272f1cd21049db2fa57cb40d6a9b3dd23a9e629b0cfd9f5f514d62193;baseline-gradient-49edafddd1de7d26262831752dc14269a6c6a72b317983c5b75c626939b66996;baseline-mapping-f536d409dc5679e2343d05a418d0a33637f9ec1db65c52e25f8eb49a1e0d1c07;particles6000;pairs340340;max-degree113|model=phi(v)=0.5*norm(max(c+A*v,0))2;A=SPACING*Jc;all-rows;support-fixed;H=At*Dactive*A|trust=global-l2;delta0.25;fixed|curvature=outer4;cg-max5;unpreconditioned;undamped;zero-initial-direction;consistent-range;hvp=jvp-mask-vjp;zero-or-nonpositive-curvature-stop|globalization=z=project-ball(v+p,delta);d=(z-v)/norm(z-v);exact-r32-piecewise-line;all-rows|comparison=same-v8;pair-pass-cap51;strict-terminal-phi-violation-mapping-dominance-over-r34|terminal=fresh-jvp-v;fresh-vjp-positive;maintained-direct-response-relative<=1e-12;projected-mapping|controls=dense-spd-newton;dense-singular-consistent;dense-active-switch;dense-trust-projection;parent;source;workspace;prefix;cg;globalization;direct;work;rollback|observations=outer-cg-residual-curvature-step-active;terminal-roots;baseline-candidate-ratios;trust-use|nullspace-stops|routes=curvature-parent-rejected;curvature-source-rejected;curvature-dense-rejected;curvature-workspace-rejected;curvature-prefix-rejected;curvature-cg-rejected;curvature-globalization-rejected;curvature-direct-rejected;curvature-work-rejected;all-inequality-projected-stationary;first-order-reference-retained;equal-work-generalized-hessian-candidate|precedence=parent,source,dense,workspace,prefix,cg,globalization,direct,work,classification|runs=2-clean-release-builds;1-process-each;byte-exact|work=control-parent-r34-once;diagnostic-workspaces1;prefix-passes1;outer4*(gradient1+cg5*hvp2+line1)=48;terminal-passes2;new-pair-passes<=51;new-hvp<=20;new-model0;new-trial0;new-outer0|correction=none;nonlinear-evaluation=none;floor-classification=none;state-mutation=none;following-outer=none;substep=none;macro=none;trajectory=none;timing=none;public-schema=none;runtime=none;production=none|credit=one-private-equal-work-generalized-hessian-classification-only
```

SHA-256:
`9462749f5b1f1113537e9bc1b57883198571721e88e4a2ddaf8f255ebedc4708`.

## Command and recurrence

Add `--nonlocal-al-equal-work-generalized-hessian`. Reproduce exact R34 and
capture its exact R33 `v8` prefix plus R34 terminal baseline. Build one new
read-only sparse workspace and validate `v8` with a fresh JVP.

Execute four curvature outer iterations. At each, form `g=A^T[r]+`; run at
most five zero-start unpreconditioned CG iterations on `H=A^T D A`; project
`v+p` to the fixed trust ball; then use exact all-row hinge line globalization.

## Hard gates

1. Identity, exact R34 bytes/semantic, source and frozen baseline roots.
2. Four independent dense curvature/globalization controls.
3. Exact 6,000-particle/340,340-pair/degree-113 workspace.
4. Fresh `v8` prefix JVP reproduces the frozen response root and maintained
   response within relative `1e-12`.
5. Every CG recurrence is finite. Positive curvature uses the exact standard
   CG alpha/beta recurrence; zero residual or nonpositive curvature stops
   without fitted damping. At most five HVPs occur per outer.
6. Every nonstationary CG result is descent. Projected chord remains feasible;
   exact all-row line has strict reduction and inherited direct line KKT.
7. Fresh terminal JVP/VJP pass; response defect is `<=1e-12`; projected
   mapping and comparison ratios are finite.
8. New work is at most 51 pair passes and 20 generalized HVPs, with zero
   nonlinear models/trials/outers. One workspace is built/released.
9. Position, dual, constraint and violated-mask roots roll back exactly.

Hard-gate PASS selects projected stationarity, retains the R34 first-order
reference, or selects the generalized-Hessian candidate. Candidate requires
strictly smaller direct terminal objective, violation norm and projected
mapping than R34.

## Proof and stop boundary

Require two clean Release builds and one fresh process from each with
byte-exact binary/stdout equality. These are not timing runs.

Do not apply either iterate, evaluate nonlinear moved state, change
penalty/trust/cap policy, execute another outer/substep/macro/trajectory,
mutate public schema/runtime or claim production readiness.

Rationale is frozen in the
[D7R19R35 research](../../development/nonlocal-nsr3b4e2d7r19r35-equal-work-curvature-research-2026-08-24.md).
