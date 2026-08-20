# NSR3-B4A closed-box/free-surface eligibility evidence -- 2026-08-21

Status: `PASS / CLOSED_BOX_FREE_SURFACE_ELIGIBLE / B4B_DESIGN_AUTHORIZED`

The frozen
[B4A contract](../plans/nonlocal-nonlinear-solver-research/03b4a-closed-box-eligibility-contract.md)
passes twice byte-identically. It executes no physical trajectory and changes
no solver coefficient.

## Static topology

The box-owned `0.3 m` cube produces the exact frozen shell counts:

| Shell | Support samples |
|---|---:|
| two layers | 784 |
| three layers | 1,512 |

On the compressed filled-box control, two and three layers agree exactly for
density, pressure energy, fluid gradient, fluid HVP and virtual support
reaction. The selected two-layer closure therefore survives all six box faces;
the third layer remains an exact zero-contribution oracle.

## Free-surface separation

The same complete box shell is retained while the upper half of the fluid is
removed:

| Observable | Result |
|---|---:|
| fluid / support samples | `108 / 784` |
| missing interior air lattice samples | 108 |
| missing air incorrectly used as support | 0 |
| support samples inside the open box | 0 |
| active pressure centres | 0 |
| maximum density ratio | `0.999999999999998` |
| bottom-layer rest-density error | `3.33e-15` |
| minimum top-layer density ratio | `0.7337166400` |

This is the required topology result: support belongs to the container, not
to the current fluid bounding box. The free surface is under-dense and does
not acquire a fictitious pressure cap.

## Analytical contact

All eleven controls pass: six individual faces, lower and upper corners,
exact graze, moving-away and a complete-box crossing. Feature IDs are exactly
`0..5` for `x-/x+/y-/y+/z-/z+`; maximum penetration and impulse/reaction
closure are both zero. The full-crossing TOI is `0.3125` within binary64
roundoff.

## Nominal cost boundary

The exact all-pairs candidate checks per objective evaluation are:

| Scenario | Outer support | Checks/evaluation |
|---|---:|---:|
| hydro | 5,824 | 52,941,000 |
| dam break | 16,384 | 116,301,000 |
| orifice outer box only | 9,344 | 74,061,000 |
| sealed product | 24,704 | 2,337,768,000 |

These counts are before multiple trust HVPs and embedded refinement. B4A
therefore selects `JOINT_CELL_NEIGHBORHOOD_REQUIRED_BEFORE_NOMINAL`. It does
not forbid the next bounded tiny B4B oracle.

## Repeatability and regression

```text
B4A raw SHA-256 (two identical runs):
6e46cb7650e48b156bf5d837d3590b4019792787d5f7fbd4ca1af9eeea5afd48

B4A JSON-without-newline SHA-256:
7282f6ab736d4bea423cc20a1dd20092a156afb3eb5d61741d6b8b4e10674c51

B4A semantic SHA-256:
5e069b7e7c86da39aef944d31fc184a564eaa8e794c54f9e63d0e855852567b1

B3R raw preserved:
89ede039a67b8e0f7f4a27d5ecd412f4691245ee8acaec612dd0894b2153edad

D5 raw preserved:
38845883a1f689aa1f126f58633d94605d8662e914c2165211c3b52577ec4261

original B3 raw preserved FAIL:
c64ad0b8d73f7ada62364a3daa2d3bed66a8bfc5a1fe6c0148b7c1f8f2e5fb2f

B2 raw preserved:
d6ba5f8e802966c25283d0c8384ed01beec20b347acb343cf5c7c2bf360d69d9
```

PASS authorizes B4B tiny pressure-only corpus contract design. It grants no
hydro/dam-break correctness, nominal execution, internal aperture, viscosity,
surface tension, CUDA, performance, runtime or production authority.

