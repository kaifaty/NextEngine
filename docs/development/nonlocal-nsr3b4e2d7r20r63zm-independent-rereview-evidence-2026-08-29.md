# NSR3-B4E2D7R20R63ZM revision-5 independent re-review evidence

Status: `GO / FIXED_CACHE_TANGENT_BOUNDARY_ONLY`.

## Verdict

The single formal revision-5 re-review returned `GO` with no load-bearing
finding. The reviewed claim is deliberately narrow: the exact frozen parent
cache can be parsed once, six direct-index tangent products can be published
in the fixed binary artifact, and a separately implemented checker can
reconstruct the selected semantics, products, work, events and route.

This does not authorize recurrence, representation-family generalization,
dynamic building, corpus work, timing, runtime integration, Rust, GPU,
cross-target or production claims. SPEC-38 and ADR-076 remain `Proposed`;
ADR-081 guardrails remain binding and later `CONTINUUM-*` ProductChecks remain
`NOT_RUN`.

## Frozen identity

- Revision-5 manifest SHA-256:
  `a04ad03a7bf3d3f22068080e757c46215c9e7988c3f1911584c6c70070052996`.
- Research snapshot: `fc8bcc53eb888e708be94ba0a08097b5169f87f2`;
  parent: `9a5a01db558821c5f73d20a8281d752f259e08f1`;
  tree: `7da4ff59ff8ca058ad3cda4080572ba1696471a9`.
- Parent-to-snapshot binary diff SHA-256:
  `863ded9a3a8f5d3cce7f3b1fdd272ab7ad388dc9372def0d45782ee974068e24`.
- Frozen contract SHA-256:
  `e7e92a8e40699b31215b6c5571263940f73621433186411e9b606e2ba0768770`.
- Parent cache: `1033625` bytes, SHA-256
  `23dbf605ad7b6ae12c4cf6a80404ead9617354ff2848bd010b52c7fa7f83bb84`.
- Selected semantic root:
  `dc1b5328d2999dadafa8915ebe10d36029cc5b5ba13f2657670de75ee4a466a5`.

All manifest-listed source hashes matched. The producer and checker remained
independent of the formula-probe core, R63ZL reader/validator and the earlier
R63ZJ/R63ZK schedulers/classifiers.

## Closure of revision-4 findings

The re-review repeated the revision-4 counterexamples rather than relying on
the author report. Invalid certificate/profile booleans at cache offsets
`5690`, `775040` and `1032659`, plus invalid skipped string, collection and
profile-vector lengths, all produced producer route `2`, first-failure ordinal
`1`, and independently verified checker route `2`.

The reviewer confirmed that:

- every skipped typed field is validated by the producer and independent
  checker parsers;
- the independent parser/admission result owns the expected route on every
  readable-cache path;
- the producer model is comparison-only and must agree with that independent
  result;
- producer-model parsing/reconstruction and unused-slot scans are present in
  the sealed 26-field checker receipt;
- the complete artifact, product and 420-byte audit offset chains are
  compile-time asserted;
- after the producer/checker terminal seals, only patch/write/mechanical exit
  work remains.

## Independent reproduction

- Fresh Release binaries were byte-identical to the frozen binaries: raw
  `4b0a4d088c1c2a1f4cd0ec1a10275c305d05306aa2b754e4b87a06f48bfd1894`,
  product
  `20c359ebd15f1e9d492e3ec70c3c06795e9e63fd88ab131bd9866df383fc3095`,
  producer
  `42e934fa10aaa3a4720e46863ef7cc1df46fa682c142df5b4eb078ec041292f8`
  and checker
  `73296d7358c6c8522149b0c76209397ff968884e9067f65e6423575547c605fe`.
- Two baseline runs reproduced raw
  `4bcd13b3a1129042c90b572c04c06a3675586a529ff2719ea80b74c6057f6bd2`,
  product
  `672a81639de00c323d847f65f211dc6b4091c5c46348e9e536a5504578c38710`,
  artifact
  `ac6946e872799baef366d8a6648e7bb5cd70c6f2acc326747fdf153c471f0b87`
  and audit
  `fd4bcf0094e9f6ab8c64080c3a22566e3a23d2544e187641075350519a281f80`.
- The expanded controls ran twice at report SHA-256
  `83c42c288e6bab63fab299a1569556536533201ee33543d04c13b899ee11739a`:
  `56/56` controls passed and all `57/57` baseline/control audit identities
  were distinct.
- All fresh allocation receipts reported
  `allocation_probe_calls=0 allocation_probe_bytes=0`.

The accepted roots are product set
`6ac66c134a3b8eea010de7e66bab209aa995071089eb8b96bd14804e2722a06b`,
bundle `e1ec9d48e74630ea1698bc7ea4cccd84ecd3fd65e300048f47055e505e72970c`,
trace `8b6e3d1ebd261767aa6722440b9e61975ef49a00cd272a1b75dfc01364b93ce6`,
producer result
`7eaedbd3775bac423c4e5dfadc25c18f0ffe2ffe181e5080675ff0a457a37da3`
and checker
`b4a7969c6676b30e87fff470f95e9c098f8951b2a477314213c359c6164c80b4`.

## Focused integration verification

The repository CMake integration configured successfully with GNU C++ 15.2.0
and built all four `nonlocal-formula-r63zm-*` targets in both Dev and Release
profiles. Running both profiles against the frozen external cache reproduced
the exact raw, product, artifact and audit hashes above. Producer and checker
stdout were empty, and all allocation receipts remained `0/0`. Running the
integrated Release `artifact_controls.py` reproduced report SHA-256
`83c42c288e6bab63fab299a1569556536533201ee33543d04c13b899ee11739a`,
status `PASS`, `56` controls and `57` distinct baseline/control audits.

The CMake-built executable identities are not added to the frozen revision-5
manifest; only their exact outputs and behavior are the integration check.
The reviewer's fresh direct Release binaries above remain the closed binary
identity evidence for the formal verdict.

## Integration boundary

The exact reviewed sources are retained together under
`crates/continuum-water/tools/nonlocal-feasibility/r63zm/`. Producer and
checker remain separate executables and do not link
`nonlocal-formula-reclosure-core`. The parent cache remains generated evidence
outside Git; no cache, build output or control artifact is repository content.

The next admissible action is a separately frozen recurrence-consumer
contract over this fixed binary boundary. It may not alter or silently absorb
the reviewed producer/checker, and it receives no correctness credit from the
earlier inconclusive R63ZJ/R63ZK/R63ZL packages.
