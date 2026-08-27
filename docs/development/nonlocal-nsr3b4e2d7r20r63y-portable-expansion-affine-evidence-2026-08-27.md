# NSR3-B4E2D7R20R63Y portable expansion-affine evidence

| Field | Result |
|---|---|
| Status | `PASS` |
| Route | `TWOFOLD_AFFINE_IMAGE_CANDIDATE` |
| Semantic SHA-256 | `ad3d84abaecc5142f05d369d0e653533c5e152cac2a9112efb2ad8277d70fb9d` |
| Stdout SHA-256, both final executions | `7427ecdb3c4c7c95900bc6046545d298a674be42865936bad543c7e0191f4372` |
| Parent R63X semantic | `904614d78c1948f4af836167936bdc528a0c69a0ded7e85cfe9ae264be395a65` |
| Controls root | `1feb766b6f0ac162e09515d49afc68bc9a7ddb85d032ab39d8d73e3762261894` |
| Selected width | two binary64 components plus outward radius |
| Authority | offline research only |

## Observation

R63Y rebuilt the exact R63X affine profile once (`10,404` right-hand-side
products and `1,061,208` matrix products), then projected every coefficient
and both immutable candidate lanes into fixed onefold and twofold binary64
expansions. All twelve finite certificates executed; no pass caused an early
exit or a post-run width change.

Both widths contained all `1,224` independent exact R63V images. The finite
classifier itself has zero binary128/exact fields and zero exact-oracle
classification uses. Its image centers are one flattened strict-binary64
`Dot2Err` reduction per row, followed only by outward binary64 representation
radii and the frozen contractive denominator.

## Results

| Lane | Width | State 0 | State 1 | State 2 |
|---|---:|---:|---:|---:|
| retained-wide candidate | 1 | `12+/24-/66?`, `4.11438e15` | `12+/24-/66?`, `1.68289e15` | `24+/77-/1?`, `48.4459` |
| exported-factor candidate | 1 | `12+/24-/66?`, `4.08658e15` | `12+/24-/66?`, `3.46642e15` | `24+/77-/1?`, `48.4445` |
| retained-wide candidate | 2 | `12+/24-/66?`, `4.11438e15` | `12+/24-/66?`, `1.68289e15` | `24+/78-/0?`, `4.43861e-4` |
| exported-factor candidate | 2 | `12+/24-/66?`, `4.08658e15` | `12+/24-/66?`, `3.46642e15` | `24+/78-/0?`, `1.01045e-3` |

The onefold profile remains a valid enclosure but fails the state-2 semantic
gate by exactly one unresolved sign in each lane. The twofold profile restores
the R63X state-2 scale and both passing sign roots equal the frozen
`89b2908b...6094` root. All `10,506` profile entries have a nonzero second
component; their maximum remaining radii fall from `15.7759` to `8.34e-16`
for the affine vector and from `1.09e-16` to `6.03e-33` for the matrix.

## Controls and regression

The control suite passes exact reconstruction for both widths, a nonzero low
component, high cancellation, coefficient/candidate radii and dropped-radius
negatives, `TwoSum`, FMA `TwoProduct`, subnormal/overflow/nonfinite rejection,
noncontractive `rho`, a finite nonsymmetric orientation discriminator, exact-
oracle independence, mutation and classifier precedence. The control root
binds the positive and negative artifacts, not only their Boolean outcomes.

The focused build passed. R63B through R63X were then executed sequentially;
all 23 stdout SHA-256 values equal their frozen evidence. R63X remains
`9e218dbd...bf7`. No CPU/wall performance A/B was run on the shared host.

## Conclusion and next boundary

Twofold binary64 expansion arithmetic is the minimum tested portable finite
verifier for this affine-image theorem. This removes native binary128 from the
certificate decision path, but not from the independent offline exact audit.
It does not make the candidate producer portable or sparse: the current exact
profile is still dense, and both candidate solutions still originate from the
retained/exported wider factor lanes.

The next research gate must separate a factorized/sparse candidate producer
from an immutable twofold-verifier artifact. It must preserve the final affine
cancellation, prove producer/operator correspondence and reject mutation or
stale identity before any corpus, timing, GPU, runtime or production claim.
