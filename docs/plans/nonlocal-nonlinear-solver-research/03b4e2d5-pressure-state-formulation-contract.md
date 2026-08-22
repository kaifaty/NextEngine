# NSR3-B4E2D5 -- pressure-state formulation contract

Status: `FROZEN / NOT_RUN / NO_TRAJECTORY`

Identity projection (exact bytes, no final LF):

```text
nextengine.nonlocal.nsr3b4e2d5-pressure-state-formulation|v1|parent=30f958a25899ba511313eb5d7d8621470a03dcb69b960af9febdb95893a97b75:c2c9ee08717b3c790aef35810122cb8a1bf866f5b13922fcc384d822f5d0fa5f:6dd02ec3b2fde108990e83218f09cca6e5d14fee51b3e716837f8e43e43a16b4|profile=kappa=1226.25;rho0=1000;mass=0.125;keff=9810000;strain320=0.0011739237712489192;strain-limit=0.001|penalty=phi=0.5*kappa*max(c,0)^2;p=keff*c;pressure320=11516.192195951897;head=1.1739237712489192;minimum-kappa=1439.524024493987;stiffness-factor=1.0834776284025984;head-scaling=1,10,100:1,sqrt10,10|phr=([max(0,lambda+beta*c)]^2-lambda^2)/(2*beta);dphi/dc=max(0,lambda+beta*c);lambda-next=max(0,lambda+beta*c);constraint=c<=0;lambda>=0;complementarity=lambda*c=0|controls=penalty-special-case;active-inactive-derivative;force-equivalence-at-zero-strain;pressure-map;zero-lambda-negative;scaling|selection=augmented-lagrangian-pressure-state;fallback=semismooth-primal-dual-if-outer-stalls|runs=2-release-builds;2-processes;byte-exact;timing=none|trajectory=none;physics-mutation=none|credit=tiny-al-oracle-contract-research-only
```

Identity SHA-256:
`5e3fc734c04294bd2705ce9ad96c371bb6423523521479f71dd6fba44049b9f8`.

## Required command

Add `--nonlocal-pressure-state-formulation`. It must run no neighborhood,
solver, trajectory or timing measurement. In binary64 it must:

1. reconstruct `K_eff=kappa*rho0/mass`, the B4E2D4 pressure/head, the minimum
   cap-sufficient penalty coefficient and square-root stiffness factor;
2. verify one-, ten- and hundred-metre head scaling from the immutable SI
   profile;
3. verify PHR at `lambda=0` equals the unilateral quadratic penalty in active
   and inactive scalar controls;
4. verify analytic derivative against a central difference away from the kink
   and active/inactive curvature against the declared `beta/0` values;
5. set `lambda_star=kappa*strain320`, require that at `c=0` its pressure map
   equals the B4E2D4 inferred pressure, and require exact primal feasibility,
   dual feasibility and complementarity;
6. prove the zero-`lambda`, zero-strain mutation cannot supply that pressure;
7. select `AUGMENTED_LAGRANGIAN_PRESSURE_STATE` and publish no other authority.

Derivative relative error must be at most `1e-7`; scalar identity, sign,
scaling and complementarity controls use finite binary64 values and their
declared exact/algebraic comparisons.

## Execution and authority

Build Release twice. Run two fresh processes without a timing wrapper and
require exit zero, empty stderr and byte-identical stdout. PASS authorizes only
research and contract design for a tiny dense augmented-Lagrangian oracle. It
does not authorize a `kappa` change, nominal trajectory, persistent pressure
schema, runtime/GPU/PhysX integration, performance or production claim.
