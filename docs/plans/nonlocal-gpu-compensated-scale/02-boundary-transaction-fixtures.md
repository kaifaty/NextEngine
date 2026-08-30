# NCGP3 boundary and transaction fixture addendum

| Field | Value |
| --- | --- |
| Research ID | `NCGP3` revision 3 |
| Status | `FROZEN / CORRECTNESS_FIRST / REPORT_ONLY` |
| Extends | NCGP3 revisions 1 and 2 without changing their claim ceiling |

## Pair-aware swept-contact fixtures

The inset lower plane is the exact binary32 value
`0x3ccccccd = 0.02500000037252902984619140625`. For every contacted axis,
freeze the canonical origin and proposal bytes:

```text
origin.hi = 0x3cccccce = 0.0250000022351741790771484375
origin.lo = 0xb04ccccd = -7.450580707946130624623969793e-10
proposal  = 0xb1000000 = -1.86264514923095703125e-9
```

The logical origin is `0.02500000149011610828253537416458`, which is inside
the basin. Its logical endpoint is `0.02499999962747095905244350433350`,
which crosses the plane. The high-only endpoint rounds to the plane exactly
and therefore misses the crossing under the endpoint `< lower` test.

Use an interior exact binary32 value and zero proposal/low part on every
uncontacted axis. The correct pair-aware sweep must produce these masks and
exact contacted coordinates with zero low part:

| Fixture | Contacted axes | Expected mask |
| --- | --- | --- |
| single face | x | `0x01` |
| edge | x, y | `0x05` |
| corner | x, y, z | `0x15` |

The high-only-boundary mutation must produce mask zero for all three while
consuming the same input and proposal bytes. The work/result receipt seals
logical-origin reconstructions, six plane tests per sample, hit count, face
mask, contacted-component canonicalizations, input pair bytes and output pair
bytes. Merely changing a variant ID or expected root is not evidence.

## Executable transaction fixture

Use the retained NCGP2 translated pair at center `x=0.75 m`, canonical device
high/low state, total HVP budget 32 and the unpreconditioned profile. Execute:

1. seal the uploaded high/low workspace state;
2. run the NCGP3 post-finalize-failure variant, which must corrupt at least one
   high and one low device component after finalization and then return the
   typed `DeviceFailure` route;
3. seal the restored high/low workspace state and require byte equality with
   step 1;
4. retry the correct NCGP3 variant in the same workspace;
5. compare its complete state/work/result roots with a correct run in a fresh
   workspace from the same input.

The failure receipt seals the injected corruption work and the rollback work.
Empty failure state or a root mutation without executable device corruption is
insufficient. All other NCGP3 revision-1 and revision-2 rules remain unchanged.
