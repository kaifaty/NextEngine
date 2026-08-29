# NSR3-B3D5 owned-gradient residual trajectory evidence -- 2026-08-21

Status: `PASS / OWNED_RESIDUAL_TRAJECTORY_CANDIDATE / B3R_DESIGN_AUTHORIZED`

The frozen
[D5 contract](../plans/nonlocal-nonlinear-solver-research/03b3d5-owned-residual-trajectory-contract.md)
completed all six fixed boundary trajectories twice byte-identically. This is
the first candidate in the B3 diagnostic chain that simultaneously closes
active reaction accuracy, inactive reconstruction arithmetic and the full
face/corner contact horizon.

## Correctness result

All rows pass:

- all `384/768/1536` substeps complete with active support and hard contact;
- maximum active stationarity/reaction-limit ratio is `0.436--0.964`;
- maximum inactive defect/bound ratio is `0.0086--0.0630`;
- cumulative displacement bounds are `3.76e-12--2.28e-11`;
- virtual translation remains below `1.78e-17 kg m/s` and contact defect is
  bit-zero;
- signed cumulative and direct terminal momentum checks pass;
- every floor trial retains exact active/pair topology and strictly decreases
  residual;
- no trust trial is rejected.

Final correspondence to the old non-aborting B3D trajectories stays inside
the frozen `1e-5 dx/c` gate:

| Fixture | substeps/frame | position difference | velocity difference |
|---|---:|---:|---:|
| face | 96 | `8.52e-6 dx` | `6.68e-7 c` |
| face | 192 | `2.18e-6 dx` | `1.76e-7 c` |
| face | 384 | `2.66e-6 dx` | `1.24e-6 c` |
| corner | 96 | `4.31e-7 dx` | `4.27e-7 c` |
| corner | 192 | `3.51e-7 dx` | `1.25e-7 c` |
| corner | 384 | `1.26e-6 dx` | `1.62e-7 c` |

## Bounded work

No smooth solve needs more than two floor-merit accepts, below the frozen cap
of four. The finest rows need at most one. Aggregate work is:

| Fixture | level | ordinary HVP | candidate HVP | ratio | floor accepts |
|---|---:|---:|---:|---:|---:|
| face | 96 | 300 | 822 | `2.74x` | 156 |
| face | 192 | 588 | 1364 | `2.32x` | 387 |
| face | 384 | 1140 | 2302 | `2.02x` | 566 |
| corner | 96 | 106 | 304 | `2.87x` | 77 |
| corner | 192 | 208 | 504 | `2.42x` | 123 |
| corner | 384 | 412 | 828 | `2.01x` | 205 |

This is still expensive and report-only. The PASS proves bounded numerical
closure, not production performance.

## Selected numerical architecture

```text
owned delta
  -> inertia energy/gradient from delta-delta*
  -> unchanged pressure gradient + analytic HVP
  -> ordinary trust-energy globalization
  -> at unresolved energy scale only:
       capped decreasing ||h*grad F|| merit
  -> unchanged reaction gate
  -> swept hard contact
  -> velocity = delta/h
```

D5 authorizes a fresh B3R composition retry with this exact candidate. It does
not retroactively change B3/D1/D3 failures and does not authorize the physical
corpus.

## Repeatability

```text
D5 raw SHA-256 (two identical runs):
38845883a1f689aa1f126f58633d94605d8662e914c2165211c3b52577ec4261

D5 JSON-without-newline SHA-256:
fd7627b22734b6be1183e0bbd53f03a99170bda35d2f37585c4da066516c2b91

D5 semantic SHA-256:
c716675fd75c4c7ecaa2410eb2f26154d7c31f36264a1d41bd9ad5b3f0aee40c

D4 raw preserved:
60a53643f7a587cbfce8fb4d12f7c774b945742e1a66ca04ea4e53a375517946

D3 raw preserved:
aa16501e6402c5a4170cec7f58d030506d564362dd928d91ca5d6c36463ca792

D1 raw preserved:
f500187b0fb02c36d4383e2e9fe1003677afc5b2744c02b587fe9f9d86b8587a
```

No physical corpus, CUDA, performance, runtime or production authority is
granted.
