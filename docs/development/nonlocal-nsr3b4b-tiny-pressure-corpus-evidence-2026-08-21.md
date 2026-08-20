# NSR3-B4B tiny pressure corpus evidence -- 2026-08-21

Status: `FAIL / P1_REFERENCE_RUN / P2_NOT_EXECUTED`

## Reproduction

```text
nonlocal-formula-reclosure --tiny-pressure-corpus-self-test
```

Two complete executions are byte-identical:

```text
raw JSON plus LF  a8fdaaf0d2ac358783b13f6beb02c506e67cb023bdd50a0bf2739e6d900624d8
JSON without LF   fef0035a7e0d005358d08d16ed134a656d73127c5ba215d2663f9feb95bdb5e3
semantic result   59d7f2436f8bbbea974b47f4109d038ed09251012340accd3b0e0664b6ee2b16
```

The first failure is
`P1_SUPPORTED_COLUMN:REFERENCE_RUN`. Per the frozen stop policy, the released
block P2 was not executed.

## What passed before the blocker

The adaptive P1 path completed all eight frames. It accepted 276 of 414
executed substeps, activated pressure and contact, and remained inside the
individual physical scales:

| Observable | Result | Frozen limit |
|---|---:|---:|
| maximum positive density strain | `5.9589474864774061e-4` | `1e-3` |
| maximum speed | `0.21735630999856018 m/s` | `0.01c` |
| maximum momentum-ledger residual | `7.0177316182344445e-10` | `1e-9` |
| maximum support-reaction closure | `1.1949536398097553e-15` | `1e-10` |
| maximum penetration | `0 m` | `1e-12 m` |

The terminal bottom-to-top mean density ratios were
`[1.0003872102490321, 0.99697412582836908, 0.83109918065664534]`.
These diagnostics do not override the failed independent reference.

## Exact refinement failure

Fixed 48 substeps/frame completes. Fixed 96 and 192 both stop on frame zero,
substep zero with `REACTION_BELOW_ENERGY_RESOLUTION`:

| Quantity | fixed 96 | fixed 192 |
|---|---:|---:|
| reaction defect, N s | `5.9689936631574676e-7` | `7.4612397140137767e-8` |
| mixed reaction limit, N s | `2.5546920380366131e-12` | `1.2773460190183065e-12` |
| predicted reduction, J | `1.1151393208186398e-13` | `1.7437192547456636e-15` |
| inherited energy floor, J | `2.2737367544323206e-13` | `2.2737367544323206e-13` |
| proposed step / dx | `1.1589205474866491e-9` | `7.2487174912679562e-11` |
| trial/current residual ratio | `1.2778566964207925e-4` | `3.1986895733209341e-5` |
| pressure topology exact | false | false |

The correction is numerically useful but crosses the unilateral compression
active set. B3R's floor merit is deliberately restricted to unchanged
topology, so accepting it would silently expand the validated globalizer.

## Architectural diagnosis

The P1 bottom layer begins simultaneously on an analytical nonpenetration
plane and beside complete ghost pressure support. The current composition is:

```text
unconstrained gravity predictor
  -> smooth pressure minimization with ghost support
  -> analytical wall sweep
```

As the fixed step shrinks, the predictor enters the wall by `O(h^2)`. That
creates a tiny compression branch during the smooth solve even though the
hard wall will remove penetration afterward. The requested model signal then
falls below the endpoint energy resolution exactly at an active-set change.

The published Nonlocal method represents solid boundaries with ghost
particles or signed-distance fields, while its public algorithm solves the
unconstrained position update; its limitations explicitly leave unified
fluid-solid collision handling as future work. See the
[paper](https://peridynamics.com/publications/2026-Liu-NUV.pdf) and the
[pinned public solver](https://github.com/peridyno/peridyno/blob/1aa892bb296fe766d2f9249c881b8605af23a69b/src/Dynamics/Cuda/ParticleSystem/SIUnifiedFluid/SemiImplicitUnifiedFluidSolver.cu).
The stricter NextEngine ledger therefore exposes a composition question not
closed by the source implementation.

## Decision

- Preserve B4B FAIL and every frozen threshold.
- Do not accept the adaptive P1 trajectory without the fixed ladder.
- Do not remove ghost support or relabel the floor event as convergence.
- Research a bound-constrained contact KKT formulation in which wall
  multipliers and pressure support jointly close stationarity and the impulse
  ledger before any new full-corpus identity.

## Regression hashes

The post-change raw outputs remain exact:

```text
B4A  6e46cb7650e48b156bf5d837d3590b4019792787d5f7fbd4ca1af9eeea5afd48
B3R  89ede039a67b8e0f7f4a27d5ecd412f4691245ee8acaec612dd0894b2153edad
D5   38845883a1f689aa1f126f58633d94605d8662e914c2165211c3b52577ec4261
B3   c64ad0b8d73f7ada62364a3daa2d3bed66a8bfc5a1fe6c0148b7c1f8f2e5fb2f
B2   d6ba5f8e802966c25283d0c8384ed01beec20b347acb343cf5c7c2bf360d69d9
```
