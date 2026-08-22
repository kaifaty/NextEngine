# NSR3-B4E2D1 frame-zero binary64 research

Date: `2026-08-22`

Status: `COMPLETE / INDEPENDENT_RAW_BIT_DISCRIMINATOR_SELECTED`

## Why topology changed

The B4E0 alignment compared canonical micrometre coordinates, not the exact
binary64 values consumed by neighborhood construction. Its local lattice is
formed as `0.025 + index*0.05`; canonical publication/decode instead forms
`integer_micrometres / 1,000,000`. They denote the same decimal coordinates
but differ by one ulp for many samples.

This usually has negligible physical magnitude, but a compact-support query
contains a discrete `r <= h` admission boundary. Pairs at exactly `h=0.15 m`
can enter or leave even though the cubic kernel value/derivatives at the
boundary are zero. Thus topology roots and capacity facts are sensitive before
energies visibly differ.

## Authority question

The correct first-step state must follow the external reference payload, not
whichever construction preserves older Nonlocal counters. A direct diagnostic
read indicates the external Dam frame zero matches micrometre division for all
6,000 vectors, but that result was produced outside a frozen evidence target.

B4E2D1 therefore extends the already independent standalone reference-slice
tool with a separate `--initial-binary64` mode. It retains complete hash/path/
layout admission, captures only Dam frame zero, and hashes stable IDs plus the
six raw IEEE754 u64 payloads. It constructs two independent candidate streams:

- integer micrometres divided by `1,000,000`;
- `0.025 + index*0.05` addition-built lattice;

with zero velocity and identical ID order. Exactly one complete candidate root
must equal the external root. Vector-level equality/mismatch counts explain
the result but do not replace complete-root equality.

No Nonlocal source is linked and no trajectory, neighborhood, canonical
publication or timing code runs. Payload-bit, candidate-bit and stable-ID
mutations must each change/reject the relevant root.

PASS selects only the binary64 source of truth and authorizes a separate
topology reclosure. It cannot retroactively change B4E0/SIRDI evidence or
resume B4E2D physics.

