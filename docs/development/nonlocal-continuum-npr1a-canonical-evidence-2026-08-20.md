# Nonlocal NPR1-A canonical evidence — 2026-08-20

Status: `REPORT_ONLY / NPR1_A_PASS / NPR1_B_NEXT`

## Outcome

The isolated `nonlocal-npr1-canonical` binary implements the frozen binary64
publication, frame-root and trajectory-root contract without changing the
NPR0 CUDA executable. All bounded self-tests pass twice with byte-identical
reports.

## Passed controls

- nearest-ties-to-even at positive and negative half values;
- negative-zero normalization and minimum-subnormal handling;
- NaN/infinity and checked multiplication/shift/i64 overflow rejection;
- exact negative i64 boundary and positive overflow;
- +/-16 m position and +/-64 m/s velocity publication bounds;
- N-1/N/N+1 sample-capacity decisions;
- storage-order canonicalization, strict canonical order and duplicate-ID
  rejection;
- invalid root encoding and empty trajectory rejection;
- independently computed frame and trajectory golden roots.

The target is compiled separately with `-ffp-contract=off` and
`-fno-fast-math`. It uses checked unsigned 128-bit integer arithmetic for the
binary64 decoder. This is Linux research evidence, not a Windows or production
authority claim.

## Exact roots

| Artifact | SHA-256 |
| --- | --- |
| frame golden | `07cb5e151046c6e51c6c680d146bb430a6ecdac91715d8ba07daa502e453d075` |
| trajectory golden | `2791ab24ef76a6466b8548c1a0b3e39076146609d0678914c3c46a2570976bfd` |
| bounded result | `f5e5f1787ddbfe35980d45e4f256dc38b898454a13e1c7b2f2b86752d6f8e700` |
| raw report, both runs | `464a55bc33741f18ddc4b3c1cda6b46b5248bb653789a5e31585482f871ee207` |
| executable | `674c4781178eadccf2d2ce9a44fc223a29bc2f6900d3ccd5ce2414ee3c8c6225` |

The NPR0 v3 corpus root remains
`d63188a4117a4cd496abfadfe7555b1e239b7e2edbfd44478decc1c57272797a`,
the v4 corpus root remains
`6b11101d6c73ebf7efbfe56f2d588ab3efffe11a056e5cc5acae6f5952fee304`,
and the retained CUDA self-test passes after adding the target.

## Consequence

NPR1-A closes only the serialization boundary and golden arithmetic. No
trajectory yet feeds its next step from canonical integers. NPR1-B term and
directional-derivative controls are the next admitted implementation.
