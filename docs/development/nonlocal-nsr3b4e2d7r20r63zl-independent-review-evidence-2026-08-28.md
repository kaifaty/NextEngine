# NSR3-B4E2D7R20R63ZL initial independent review evidence

Status: `INITIAL_REVIEW_NO_GO / ONE_BATCHED_REPAIR_AUTHORIZED`.

## Verdict

Independent review of frozen implementation snapshot
`f6475040616d0f4f55f933ffd4b7c12ec0068c2f` returned `NO-GO`. The immutable
admitted API and its dot arithmetic were confirmed, but the work boundary is
not complete and cannot yet be consumed by a new recurrence checker. R63ZJ and
R63ZK remain `INCONCLUSIVE`.

The reviewer verified parent `de277c8d...a2d`, complete diff
`127248d1...61aa`, contract/source blobs and cache
`23dbf605...bb84`. A clean detached Dev and Release build reproduced stdout
`329ee21f...f28c`; the Release binary matched the author at
`6e3425b0...b4d7e`. R63ZI/R63ZG/R63ZH/R63ZJ/R63ZK regressions remained exact
at `98736993...0a1e`, `ca2a0f80...29c9`, `181ac246...6722`,
`b9ded7d7...7f8a` and `73df9f1b...2755`.

## Load-bearing findings

1. Main used the full parent cache reader. It materialized every parent field
   and called `formula_probe_parent_fixture_valid` before admission, executing
   unrelated solution/certificate/factor/common/profile hashes and six
   verifier reconstructions outside every R63ZL receipt.
2. Malformed serialized input escaped through stderr, while an in-memory
   failed admission was dereferenced through `context()` before the declared
   structured rejection route could be emitted.
3. Candidate and legacy products were interleaved. The first legacy
   validation/hash ran after one candidate product rather than after all six.
4. Control and result seals were incomplete: control roots contained only
   aggregate counters, digest controls did not enter classification,
   `RouteInput::Seal` was unreachable from top-level logic, checker work was
   tested before final seal work, and the printed result root was never
   independently validated.
5. The six inputs were authenticated only by the forbidden parent validator.
   Main copied 612 input components, and candidate plus reference kernels
   performed 771,120 temporary operand assignments, none owned by a receipt.

## Positive boundary retained

The review confirmed that `FormulaProbeAdmittedTangent` has a private
constructor, immutable shared payload and no fabricating public API. Its
factory executes the ten admission predicates in the frozen order, with one
tangent hash, two scalar parses and 32,130 tangent copies. Its dot arithmetic
matches legacy operation-for-operation and omits only already established
size guards and unconsumed witness-root material.

## Batched repair decision

Revision 3 consumes the contract's only repair. It introduces a selective
fail-closed reader, direct authentication of all six input identities,
non-interleaved candidate/reference phases, direct-index candidate dots,
complete copy/allocation receipts, per-control outcome seals and a validated
top-level result. The single re-review must reject the package if any
load-bearing ownership or sealing defect remains.

No numerical-correspondence, representation, timing, corpus, runtime, GPU,
ProductCheck or production authority exists.
