# Physical sound V24 T0 — analytic modal teacher protocol

| Field | Value |
| --- | --- |
| Date frozen | `2026-09-01` |
| Status | `FROZEN_BEFORE_IMPLEMENTATION / SYNTHETIC_VALUES_UNOPENED / CONTRACT_ONLY` |
| Roadmap package | V24 `T0` |
| Data contract | V24 D0 `synthetic_teacher` / `sorted-modal-contact-field-v1` |
| Product effect | None; external synthetic research only, with authored clips still authoritative |

## Question and claim

Can one small deterministic teacher produce a multi-object modal/contact corpus
whose physical inputs and exact outputs are known, whose remesh twins preserve
the same truth, and whose records can traverse the V24 V3 evidence plane?

T0 may claim only analytic representation recovery, causal sensitivity and
deterministic rendering. It cannot claim real Metal, Glass or Wood identity,
air-radiation truth, perceptual quality, validator admission or runtime use.

## Selected teacher

Use two closed-form linear-elastic families:

1. a simply supported isotropic Kirchhoff-Love rectangular plate;
2. an Euler-Bernoulli rectangular cantilever beam clamped at `u = 0`.

The plate modes are

```text
phi_mn(u,v) = sin(m*pi*u) * sin(n*pi*v)
D = E*h^3 / (12*(1-nu^2))
omega_mn = pi^2 * sqrt(D/(rho*h)) * ((m/a)^2 + (n/b)^2)
```

The beam uses the first ten positive roots of
`cos(beta)*cosh(beta) = -1`, the standard cantilever mode shape, and

```text
omega_n = beta_n^2 / L^2 * sqrt(E*I/(rho*A))
A = width*height
I = width*height^3/12
```

The implementation stores the exact root table as decimal strings and checks
the characteristic residual independently. No random solver, optimizer, FEM
mesh or learned parameter participates in T0.

For both families, a fixed off-node pickup multiplies the impact mode shape.
The signed contact gain is normalized once per object over the continuous
analytic domain. Damping is the declared synthetic law
`decay_per_second = base + slope*frequency_hz`; it is not inferred from audio.

## Frozen synthetic parameter sets

The material IDs are deliberately non-real labels:

| ID | Density kg/m3 | Young's modulus Pa | Poisson ratio | Damping base 1/s | Damping slope 1/Hz |
| --- | ---: | ---: | ---: | ---: | ---: |
| `elastic-a` | `2700` | `69000000000` | `0.33` | `8` | `0.0015` |
| `elastic-b` | `7850` | `200000000000` | `0.29` | `12` | `0.0010` |
| `elastic-c` | `8900` | `110000000000` | `0.34` | `18` | `0.0008` |

Exactly twelve physical recipes are frozen before modal generation:

| Role | Object | Family | Material | Dimensions metres |
| --- | --- | --- | --- | --- |
| train | `t-plate-a` | plate | `elastic-a` | `a=.24, b=.18, h=.0030` |
| train | `t-plate-b` | plate | `elastic-b` | `a=.20, b=.15, h=.0020` |
| train | `t-beam-a` | beam | `elastic-b` | `L=.30, w=.040, h=.0040` |
| train | `t-beam-b` | beam | `elastic-a` | `L=.26, w=.035, h=.0030` |
| development | `d-plate` | plate | `elastic-c` | `a=.22, b=.16, h=.0025` |
| development | `d-beam` | beam | `elastic-c` | `L=.28, w=.045, h=.0035` |
| calibration | `c-plate` | plate | `elastic-a` | `a=.18, b=.13, h=.0018` |
| calibration | `c-beam` | beam | `elastic-b` | `L=.24, w=.030, h=.0028` |
| method_holdout | `h-plate` | plate | `elastic-b` | `a=.27, b=.17, h=.0033` |
| method_holdout | `h-beam` | beam | `elastic-a` | `L=.32, w=.050, h=.0045` |
| admission_shadow | `s-plate` | plate | `elastic-c` | `a=.19, b=.145, h=.0022` |
| admission_shadow | `s-beam` | beam | `elastic-c` | `L=.34, w=.038, h=.0032` |

Each object retains exactly ten lowest positive modes after canonical sort by
`(frequency_hz, family_index_a, family_index_b)`. Every damped frequency must
be finite, positive and below `18,000 Hz`; decay must remain positive and
underdamped. Failure rejects the complete run.

## Geometry, contacts and remesh identity

Every object has nested top-surface triangle grids `17 x 13` and `33 x 25`.
The refined grid contains every coarse normalized `(u,v)` vertex. Triangles
use the same lower-left then upper-right diagonal rule. Stable vertex order is
`v-major, then u-major`; triangle order follows cells in that order.

The twelve canonical contacts are continuous rational coordinates:

```text
context: (1/8,1/8) (7/8,1/8) (1/8,7/8) (7/8,7/8)
         (1/2,1/4) (1/4,1/2) (3/4,1/2) (1/2,3/4)
query:   (3/8,3/8) (5/8,3/8) (3/8,5/8) (5/8,5/8)
```

Plate contacts map to `(a*u, b*v, h/2)`; beam contacts map to
`(L*u, w*(v-1/2), h/2)`. The pickup is plate `(0.37,0.61)` and beam
`u=0.83`. The same physical object, role, contact and analytic target bind both
mesh resolutions. V3 lane rows use only the refined mesh; the coarse twin is a
hash-bound control in teacher evidence. Remesh is not a new object or
partition. The common generator revision is lineage, while every V3 role uses
a distinct frozen synthetic source-group namespace so no partition parent
crosses roles.

## Canonical artifacts

T0 emits only to a fresh external directory. Large/generated artifacts stay
outside Git.

`mesh.bin` is little-endian:

```text
8-byte ASCII magic NEMESH01
u32 version=1, u32 vertex_count, u32 triangle_count
vertex_count * f64[3] positions
triangle_count * u32[3] indices
```

`modal-parameters.bin` is little-endian:

```text
8-byte ASCII magic NEMODT01
u32 version=1, u32 mode_count=10
mode_count * (u32 ordinal, u32 family_a, u32 family_b,
              f64 frequency_hz, f64 decay_per_second)
```

`contact-gain-field.bin` is little-endian:

```text
8-byte ASCII magic NEGAIN01
u32 version=1, u32 vertex_count, u32 mode_count=10
32-byte raw SHA-256 of mesh.bin
vertex_count * mode_count * f64 signed gains, vertex-major
```

Every canonical contact also emits a three-second, mono, `48 kHz`, float32 WAV
from the existing second-order damped modal equation at one declared canonical
impulse. One object-global scale is derived from the analytic full-domain gain
bound and is reused for every contact; per-clip peak normalization is forbidden.

`teacher-evidence.json` binds the recipe, formulas, material/support IDs,
contact/pickup policy, code/environment hashes, artifact hashes and all control
results. `lane-records.json` contains V3-ready row fragments, not a standalone
V3 manifest: D0 intentionally requires all three lanes in the final combined
manifest.

## Gates and mutations

A `Pass` requires all of the following twice byte-exactly:

- twelve objects, ten modes and twelve contacts per object;
- finite/bounded binaries and WAVs with no clipping or per-contact scaling;
- plate frequency scaling under isolated `E`, `rho`, `h`, `a` and `b` changes;
- beam frequency scaling under isolated `E`, `rho`, `h` and `L` changes;
- cantilever characteristic residual at most `1e-12` for every stored root;
- exact plate boundary zeros and cantilever clamp displacement/slope zeros;
- bit-identical modal bytes and common-vertex gains across remesh twins;
- non-constant contact participation for every object;
- exact force-linearity controls at impulses `0.5`, `1.0`, `2.0`;
- object/source/recording/condition groups disjoint across V3 roles;
- full owning CLI A/B, including every writer, before real or model values.

Tests must corrupt formula ID, root, material scalar, support, dimensions,
mode order/count, mesh hash, contact role, remesh vertex, gain, WAV finiteness,
artifact hash and output occupancy. A same-output rerun, partial publication or
repository-local output rejects.

## Resource and stop rules

The implementation may use Python standard library plus NumPy, but no network,
GPU, optimizer, PyTorch, SciPy eigensolver or runtime dependency. One complete
run is limited to `300 s`, `1 GiB` peak RSS and `256 MiB` output.

Any analytic/control/remesh failure closes T0 without materializing X0 or M0.
The protocol may be revised only before teacher values; after a value is
opened, a failed role cannot select a nearby threshold, recipe, contact or
mode-count retry.
