# NSR3-B4E2D7R20R10 v3 operator preflight research

Status: `RESEARCH COMPLETE / INPUT-ONLY PREFLIGHT SELECTED`.

## Purpose

R9 freezes source diversity but does not prove that the local TRQP is
nontrivial. The v1 experience showed that plausible contact scenes can project
to zero positive density rows and therefore provide no solver evidence.

R10 independently materializes the exact R64 sparse operator, contact box,
global trust-ball projection, row scales and projected target. It measures
only whether each frozen v3 source excites at least one density inequality.

## Admission rule

Every source must satisfy:

- exact workspace/operator slot, entry and incidence ownership;
- `source_positive=0`, preserving the known zero-step feasibility witness;
- `projected_positive>0` with a positive value separated from its binary64
  outward bound;
- positive finite diagonal/row scale and scale-invariance controls at `2^-8`
  and `2^8`;
- distinct exact problem roots.

All four are blind holdouts, so even one quiet case rejects v3 corpus
admission. No replacement source can be added after this observation under the
R9 identity.
