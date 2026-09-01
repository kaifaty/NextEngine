# Edge-aware presentation surface discriminator — revision 1

| Field | Value |
| --- | --- |
| Research ID | `NGQ5` revision 1 |
| Status | `FROZEN / PRESENTATION_ONLY / TOOL_ONLY` |
| Negative controls | NGQ4 revision 1 area gate; NGQ4 revision 2 isotropic height blur |

## Observation

NGQ4 revision 2 preserves the raw simulation roots and closes mask topology,
but its ordinary 3x3 binomial height average fails 4k step 48:

```text
depth RMSE         0.057320653 m
depth p95 change   0.148925127 m
maximum change     0.275410086 m
```

The same filter passes steps 0/24 at about `2.5 mm` RMSE. The first known
failure is therefore height smoothing at a developed moving front, not
simulation, mask closure, connectivity, capacity or mesh construction.

## Competing hypotheses

| ID | Causal hypothesis | Prediction |
| --- | --- | --- |
| HG5A | isotropic averaging crosses a real depth discontinuity | range-aware weights restore the frozen depth gates while the isotropic control repeats its failure |
| HG5B | the closed-pixel depth assignment already creates the error | range-aware weights remain near the isotropic failure |
| HG5C | a single top-down height field is inadequate for the developed front | finite/error gates or connected mesh gates still fail after edge preservation |

## Frozen discriminator

Retain the exact NGQ4 revision-2 component selection, binary close, filled
pixel assignment, geometry, frames and gates. For every wet pixel compute in
one pass:

```text
spatial weight = [1,2,1] x [1,2,1]
range sigma    = particle radius = 0.025 m
range weight   = exp(-0.5 * (neighbor_height-centre_height)^2 / sigma^2)
output         = weighted mean / weight sum
```

Also compute and root-bind the old isotropic result on the same `base_depth`
input as the causal control. The selected presentation surface uses only the
bilateral result. No additional iteration or parameter lane is allowed.

The frozen NGQ4 revision-2 mask/locality gates and NGQ4 revision-1 depth gates
remain unchanged. A PASS requires both 4k and 16k, all five frames each.

## Source boundary

The edge-preserving principle follows Carlo Tomasi and Roberto Manduchi,
“Bilateral Filtering for Gray and Color Images,” ICCV 1998,
DOI `10.1109/ICCV.1998.710815`:
<https://projects.iq.harvard.edu/sites/projects.iq.harvard.edu/files/imagenesmedicas/files/tomasi1998kg.pdf>.
The source supports the local domain-plus-range weighting principle only; all
water-specific parameters and acceptance gates above are frozen here.

## Stop and interpretation

- bilateral passes while isotropic repeats the moving-front failure: HG5A is
  supported bounded and the surface prototype may advance;
- bilateral and isotropic both fail similarly: HG5B remains live;
- bilateral is numerically local but topology/mesh fails: HG5C is supported;
- any raw simulation/root/control change: `APPARATUS_INCONCLUSIVE`.

Do not tune range sigma or add more passes after the result. CPU extraction
time remains diagnostic and outside the GPU physics budget.
