# NSR3-B4E2D7R19R42 stable-superset relinearization evidence

Date: `2026-08-24`

Status: `PASS / STABLE_SUPERSET_RELINEARIZATION_CANDIDATE / ROLLBACK EXACT`.

## Outcome

One source-anchored canonical Verlet superset covers both exact R40
compact-support graphs and supports a freshly evaluated trial operator. The
skin is only an ownership domain: physical `H` and exact compact-support
evaluation remain unchanged. Both masked workspaces reproduce independent
canonical current/trial workspaces bit-for-bit.

This resolves R40's topology-ownership problem. It does not accept the R40
trial: already-active contact faces and negative complete merit remain open.

## Coverage and correspondence

| Quantity | Result |
|---|---:|
| Superset pairs / maximum degree | `386402 / 122` |
| Current / trial active pairs | `340340 / 340262` |
| Maximum particle displacement squared | `1.2762005182107088e-17` |
| Maximum particle displacement | about `3.5724e-9` |
| Physical global step L2 | `6.1414252615669614e-8` |
| Skin | `0.006` |
| Current / trial canonical correspondence | exact / exact |

The conservative `4*d_max^2 <= s^2*(1-2e-12)` certificate passes. Removing
one required pair from the superset makes the inclusion control reject.

```text
superset root     355ce6fa5ccaf7524e2f28fedf74c68baf0e85a60429a5442d3a4bd707130f01
current mask root ca978c11d71d315ecb826c3fdde4cbec64bbc9ba1b98df6baf6417b4b7927f75
trial mask root   6a4fa0678189621fc4cce7e53041741364e0480a2801fed0c1547b44d80d7879
```

## Trial relinearization

| Control | Result | Gate |
|---|---:|---:|
| Pair/direct JVP failures | `0` | `0` |
| Maximum JVP rounding-bound ratio | `0` | `<=1` |
| Exact-superset centered FD relative error | `3.9624001932496727e-10` | `<=1e-6` |
| Adjoint relative error | `2.9457467349214905e-14` | `<=1e-12` |
| Interior translation | exact zero | required |
| Current/trial deterministic JVP L2 difference | `2.857890576320178e-7` | attribution |

Finite difference evaluates the complete superset with exact `W(r)` at both
perturbed states, including horizon crossings. Its agreement confirms that
the closed `C2` kernel supports smooth first-order relinearization without a
shell epsilon.

The freshly relinearized trial remains nearly feasible but not stationary:

```text
hinge psi           9.169374035643007e-27
violation           1.354206338461241e-13
active rows         53
zero-origin mapping 2.966049437035721e-14
mapping root        8ddf0976be7423a21a568a9af9dd946207470f6ac1bed3921d0c3bfca3e966cb
```

No observed value is promoted to a stopping tolerance.

## Reproducibility

Implementation commit: `84129597`.

```text
/home/kaifaty/.cache/nextengine/build-nonlocal-b4e2d7r19r42-a.CRdW4I
/home/kaifaty/.cache/nextengine/build-nonlocal-b4e2d7r19r42-b.ZenT8V
binary SHA-256 3745fce636536142dfbb9567f30ff391c7ad88bd4569b0756172453b7611a601
size           7622568
ELF build-id   f8cbde15e07791dbd3724f4adca648906738456a

/home/kaifaty/.cache/nextengine/run-nonlocal-b4e2d7r19r42-a.cFaUjm
/home/kaifaty/.cache/nextengine/run-nonlocal-b4e2d7r19r42-b.UtP6w8
stdout bytes   2422
stdout SHA-256 f7ae3a0c60efce0a78d3d1821e51b12c97f2fb8557724e5f3082bf493bb45106
semantic       fed11eefb017fce471b41fa25a20f5db9ea93e563b90fe5d02123380d52784d2
stderr bytes   0 / 0
```

R41 also retains its exact stdout SHA
`0237a380e86732e1ef78c50ca390ef2f93b1a82ac47338488403d9c5bb00d81c`
after the internal workspace-construction refactor. No timing was measured or
interpreted on the shared host.

## Decision

Retain stable superset ownership plus exact state-local masks and fresh
coefficients as the topology-admission candidate. Research a contact-tangent
projection of the R40 normal step next, then rebuild/relinearize and reevaluate
nonlinear feasibility and complete merit. Do not change physical support,
accept R40 retroactively or claim production readiness.
