# Physical sound synthesis — current task state

| Field | Value |
| --- | --- |
| Status | `ROADMAP_V14 / N0_REBASELINE_COMPLETE / N1_DATASET_CONTRACT_NEXT / MEASURED_TRANSFER_OPTIONAL_SOURCE_BLOCKED / FALLBACK_REQUIRED / P1_BLOCKED` |
| Updated | `2026-08-31` |
| Task key | `physical-sound-synthesis` |
| Scope | External multi-object neural canonical-impact research, deterministic clip cooker and independent automatic validation |
| Definition of done | One frozen generator/cooker and independent validator process untouched Glass/Wood/Metal shadows exactly once; every decision bakes byte-identical clips or selects complete fallback |
| Authority | Working context only; Accepted SPEC/ADR, roadmap and exact evidence outrank this file |

## Resume in 60 seconds

- **Current conclusion:** V14 removes recovery of one perfect exact source from
  the critical path. The next useful target is a multi-object, offline neural
  canonical-impact prior followed by deterministic clip cooking.
- **Exact evidence:** [M2c result](../physical-sound-v13-m2c-realimpact-five-impact-control-result-2026-08-31.md),
  manifest `c003ace2…b3a8`, report `afb9ae5c…e8af`; object 92 remains closed by
  [M2b](../physical-sound-v13-m2b-object92-raw-force-inventory-result-2026-08-31.md).
- **Rebaseline:** [Roadmap V14](../../plans/physical-sound-synthesis-roadmap-v14.md)
  preserves the canonical/measured claim split and makes measured transfer an
  optional upgrade rather than a prerequisite for neural authoring.
- **M1a result:** [Research Record V0](../physical-sound-v13-m1a-research-record-v0-result-2026-08-31.md)
  passes 12 focused guards, repeat-exact synthetic builds and exact
  current-version roundtrip with all access counters zero.
- **M1b result:** [Exposure Ledger V0](../physical-sound-v13-m1b-exposure-ledger-v0-result-2026-08-31.md)
  passes 13 focused guards, including contact/mutation leakage, and two
  byte-identical builds with zero build-time source/signal access.
- **M1c result:** [historical census](../physical-sound-v13-m1c-historical-census-and-shortlist-result-2026-08-31.md)
  closes M1 and leaves fresh glass candidates `59/82/92/93`.
- **Next action:** N1a Dataset Contract V1 schema and signal-blind source-quality,
  exposure and object-disjoint split invariants; sample decode remains forbidden.
- **Spend rule:** object `41` becomes a permanent acquisition-OOD fixture. Do
  not lower force coverage, choose favorable contacts or open its microphone,
  development, holdout, validator or shadow roles.
- **Object-92 rule:** do not drop contact `35`, reduce roles or fetch `12 GiB`;
  the next-object header proves the missing member is not later in the archive.
- **Deployment rule:** accepted research output bakes ordinary clips offline.
  Runtime inference remains unauthorized and every query has an authored clip
  fallback.
- **Product boundary:** SPEC-45 remains `Proposed`; no public schema, model,
  dataset, WAV, checkpoint, validator release or runtime promotion exists.

## Current program state

| Stage | State | Exact consequence |
| --- | --- | --- |
| R2 listener field | `REJECTED` | Direct/phase, separable and low-rank coordinate fields are closed. |
| R3A V3B codec | `REJECTED / REPRODUCIBLE` | NDAC preserves coarse envelope but loses spectrum and modes. |
| R3A V5/V8/V9 | `REJECTED / REPRODUCIBLE` | Bounded codec/residual/modal variants fail real modal transfer. |
| V11 B1/B1R2/B1R3 | `REJECTED / REPRODUCIBLE` | Known-truth iterations separate noise, local modal support and acquisition coverage. |
| V12 C1 | `COMPLETE / REPEAT_EXACT_PASS` | Coverage-certified oracle recovers `7/7` truth poles with calibrated OOD. |
| V12 C2–C3 | `COMPLETE / REPEAT_EXACT_PASS` | ObjectFolder object `41` and six signal-blind roles are hash-frozen. |
| V12 C4a | `CLOSED / REPEAT_EXACT_DATA_INSUFFICIENT` | Force coverage fails before response decode; protected roles remain sealed. |
| V13 M1 | `COMPLETE / REPEAT_EXACT_ZERO_SIGNAL` | Record, builder and real historical census/shortlist pass. |
| V13 M2 | `CLOSED_BY_V14_REBASELINE` | RealImpact control is exact; object 92 stays rejected and source recovery no longer blocks model feasibility. |
| V14 N1–N5 | `N1_NEXT / N2_N5_BLOCKED` | Freeze the multi-object dataset contract, then prove known-truth, real representation, few-shot contact field and cross-object prior in order. |
| V14 N6–N8 | `VALIDATOR_CAN_DEVELOP / ADMISSION_BLOCKED` | Validator structure may advance after N1; admission and cooker require a held generator winner. |
| V14 N9–N12 | `BLOCKED / ADR_REQUIRED / TRANSFER_OPTIONAL` | Domain growth and production wait for shadow evidence; measured transfer is a separate source-gated upgrade. |

## Material transition: V12 closes on force coverage

- **Observation:** all sixteen fit impacts start at samples `48,000…48,004`
  and have strong input SNR, yet twelve contacts contribute no target bins at
  the frozen relative-power gate. Of `4,631` singly supported bins, only three
  have two supporters and none have three or four.
- **Evidence:** two full force-only executions emit byte-identical certificate
  and report hashes; force PCM count is `4,608,000`, while microphone,
  protected and network counts are all zero.
- **Conclusion:** the source is spectrally narrow/non-overlapping for a shared
  arbitrary-force FRF claim. Threshold tuning cannot create the missing
  excitation information.
- **Decision:** close V12 before response fitting. Preserve object `41` as an
  OOD regression fixture and leave `MeasuredTransferField` blocked until a
  different broadband paired source passes a fresh certificate.
- **New direction:** V13 tests a narrower exact-object mapping—canonical
  normalized impact plus surface contact to global frequencies/damping and
  contact-dependent complex modal gains—then bakes deterministic clips.
- **External basis:** [V13 research](../physical-sound-v13-canonical-modal-field-research-2026-08-31.md)
  records RealImpact, AV-MSF, DiffSound and NeuralSound evidence and the limits
  on what each can support.

## Material transition: M1a freezes the research lifecycle

- **Observation:** prior work had exact experiment evidence but no single
  machine-enforced object lifecycle or claim/axis boundary.
- **Evidence:** [M1a result](../physical-sound-v13-m1a-research-record-v0-result-2026-08-31.md)
  records two byte-identical five-state fixture branches, exact successor
  roundtrip and zero network/source/signal counters.
- **Conclusion:** current-V0 records can preserve source, claim, axes, parent
  hashes, independent admission evidence and authored fallback without gaining
  product authority.
- **Decision:** M1a is complete. Unknown schema/axis, lifecycle skips, claim
  widening and terminal reopening fail closed; no synthetic migration is
  promised. M1b now supplies the real prior-exposure ledger hashes.

## Material transition: M1b makes exposure machine-checkable

- **Observation:** 271 external experiment directories contain 1,356 JSON
  files across incompatible schemas; key-name inference cannot safely recover
  sample semantics.
- **Evidence:** [M1b result](../physical-sound-v13-m1b-exposure-ledger-v0-result-2026-08-31.md)
  records exact artifact/pointer binding, 13 guards and repeated zero-source-
  signal builds.
- **Conclusion:** a declarative hash-closed catalog can normalize historical
  identities and detect contact/mutation leakage without opening waveforms.
- **Decision:** M1b is complete. M1c must populate the exhaustive
  `historical_union`; no source is clean merely because an adapter is absent.

## Material transition: M2a freezes the fresh exact object

- **Observation:** of fresh glass candidates `59/82/92/93`, only object `92`
  has the complete selected compact audio/contact/point-cloud/scale/split chain.
- **Evidence:** two real-source executions emit byte-identical manifest/report,
  bind `36` WAV headers and coordinates, and retain zero PCM/force decode.
- **Conclusion:** object `92` can support the canonical exact-contact lane
  without selecting on sound; its six roles are now immutable.
- **Decision:** M2a is complete. M2b may only inventory raw force under the
  frozen `1 GiB` increment and `12 GiB` ceiling, then bind RealImpact control.

## Material transition: M2b rejects incomplete raw lineage

- **Observation:** the 8-GiB prefix reaches object 93 with 143/144 selected
  object-92 members; only `92/audio/35/metadata.yaml` is absent.
- **Evidence:** two final inventories are byte-identical; 36/36 raw microphone
  hashes equal compact hashes and all signal/numeric-force counters are zero.
- **Conclusion:** more prefix bytes cannot repair the source. The complete-
  quartet contract fails before fitting.
- **Decision:** close object 92 without role reduction. Continue independent
  M2c control freeze, then choose a different preregistered canonical source.

## Material transition: V14 removes the perfect-source bottleneck

- **Observation:** two independently frozen exact-source lineages failed before
  real model fitting for different reasons, while synthetic and multi-object
  real evidence can test representation feasibility without those axes.
- **Evidence:** [V14 research](../physical-sound-v14-data-first-neural-rebaseline-2026-08-31.md)
  cross-checks current ObjectFolder, ObjectFolder 2.0, RealImpact, AV-MSF and
  ModalSound publisher surfaces against the existing exact failures.
- **Conclusion:** a perfect raw-force object is necessary for measured transfer,
  not for a learned canonical-impact authoring prior.
- **Decision:** close V13-M2d as a sequential prerequisite. N1 freezes a
  multi-object Dataset Contract before N2 known-truth neural work.

## Durable negative knowledge

- R2 direct/phase, separable and low-rank listener fields collapse or lose
  held endpoints; coordinate kernels without physical mode-shape evidence are
  not a successor.
- R3A V3B cleanly rejects a general neural codec as modal-fidelity evidence.
- V5 optimization variants and V8/V9 residual/modal capacities fail real
  spectrum/modal transfer; nearby capacity or postfilter tuning is closed.
- V11 proves source excitation, response observability and query relevance are
  different evidence domains. A smooth impact is not broadband by assumption.
- V12 object `41` has valid impacts but cannot support the intended shared FRF;
  its microphone must not be opened by weakening the preregistered gate.
- Static random-phase magnitude residual and the retired V9 residual are not
  admissible shortcuts. Unexplained energy remains uncertainty/fallback until
  a separately preregistered residual family passes held evidence.

## Stable decisions

| ID | Decision | Reconsideration condition |
| --- | --- | --- |
| D-001 | Physical audio is presentation-only; gameplay hearing remains deterministic `AcousticFactV1`. | A superseding Accepted ADR. |
| D-002 | Impact is first; rolling, scraping, fracture, cloth, fluid, fire and biological sound are separate work. | An admitted impact vertical and separately scoped consumer. |
| D-003 | Acoustic profiles stay separate from physics-material authority. | A consumer proves a shared physical source-of-truth field. |
| D-004 | Production waits for an engine-owned committed contact projection; no raw PhysX callback path. | Projection and ProductCheck exist. |
| D-005 | Neural inference is offline; runtime receives deterministic cooked clips. | A measured need and separate ADR define runtime model/fault/fallback. |
| D-006 | Real evidence is internet-sourced; the user performs no local impact recording. | Explicit product-owner reversal. |
| D-007 | Validation is an independent frozen ensemble with calibrated OOD, not a per-sound human queue. | A simpler policy proves equal bounded risk and coverage. |
| D-008 | Missing geometry/support/force/listener axes narrow the claim and are never inferred from labels. | A hash-closed source supplies the axis. |
| D-009 | First ML task is exact-object contact variation at one canonical listener; radiation is later. | A visible consumer proves radiation must precede contact variation. |
| D-010 | A neural waveform decoder is report-only and cannot satisfy cooker admission. | A future Accepted ADR changes the deterministic product boundary. |
| D-011 | Geometry/contact ML predicts bounded modal gains around object-global frequency/damping and must beat classical controls. | Fresh held evidence rejects this factorization. |
| D-012 | `CanonicalImpactField` and `MeasuredTransferField` are distinct claims in every record and validator decision. | A paired broadband source and accepted evidence safely unify them. |
| D-013 | `FallbackOnly` and `FallbackOutOfDomain` are valid terminal object states, not invitations to tune. | A new preregistered method family revision supplies new evidence. |
| D-014 | Research Record V0 remains external and experimental until a concrete product consumer and ADR justify a public contract. | Accepted architecture promotion. |
| D-015 | A learned canonical-impact prior may claim bounded plausibility, not true material recovery or arbitrary-force transfer; network output is cooked offline. | Independent exact-object and measured-transfer evidence justifies a stronger record. |

## Open hypotheses

| Hypothesis | Evidence for | Evidence against | Next discriminator |
| --- | --- | --- | --- |
| H1: hybrid neural modes plus spatial gains represent canonical impacts | Physical factorization, AV-MSF evidence and multi-object contact data | No fresh Next Engine real held-contact pass | V14 N2–N4 |
| H2: shared mesh/material prior beats interpolation on unseen objects | ObjectFolder spans many objects/materials | Prior coordinate-only fields failed | V14 N5 object-disjoint tournament |
| H3: automatic validator reaches useful bounded risk | Hard/acoustic/physics/corpus components are defined | No independent release or shadow result | V14 N6 then N8 |
| H4: baked atlas meets product cost | Offline clips preserve current deterministic fallback boundary | Whole-mixer/voice cost is unmeasured | V14 N7 then visible consumer |

## Do not retry

- Any per-object threshold, seed, capacity, checkpoint, contact or model
  selection after a role is opened.
- Lowering V12-C4 force coverage, selecting favorable object-41 contacts or
  opening its response/protected roles.
- Claiming arbitrary-force response from canonical normalized impacts or
  publisher-deconvolved response without raw paired provenance.
- Prompt-to-waveform, a universal codec, stationary random-phase residual or
  nearby V9 residual as the engine formula.
- Another coordinate/listener kernel without surface mode-shape evidence.
- Local microphone/hammer capture, manual validation of every sound, raw
  PhysX-callback mixing or runtime neural inference first.
- Dropping object-92 contact `35`, treating 35 complete quartets as a pass or
  fetching the already-proven irrelevant 12-GiB checkpoint.

## Required context

Read in precedence order:

1. [Agent routing](../../architecture/agent-routing.md), SPEC-00 and SPEC-01.
2. SPEC-08/24/26/30, ADR-027/046/058/071 and
   [SPEC-45](../../architecture/45-physical-sound-synthesis-and-acoustic-presentation.md).
3. [Roadmap V14](../../plans/physical-sound-synthesis-roadmap-v14.md),
   [V14 research](../physical-sound-v14-data-first-neural-rebaseline-2026-08-31.md),
   [Roadmap V13](../../plans/physical-sound-synthesis-roadmap-v13.md),
   [M2c protocol](../physical-sound-v13-m2c-realimpact-five-impact-control-protocol-2026-08-31.md),
   [M2c result](../physical-sound-v13-m2c-realimpact-five-impact-control-result-2026-08-31.md),
   [M2b protocol](../physical-sound-v13-m2b-object92-raw-force-inventory-protocol-2026-08-31.md),
   [M2b result](../physical-sound-v13-m2b-object92-raw-force-inventory-result-2026-08-31.md),
   [M2a protocol](../physical-sound-v13-m2a-object92-source-role-freeze-protocol-2026-08-31.md),
   [M2a result](../physical-sound-v13-m2a-object92-source-role-freeze-result-2026-08-31.md),
   [M1a protocol](../physical-sound-v13-m1a-research-record-v0-protocol-2026-08-31.md),
   [M1a result](../physical-sound-v13-m1a-research-record-v0-result-2026-08-31.md),
   [M1b protocol](../physical-sound-v13-m1b-exposure-ledger-v0-protocol-2026-08-31.md),
   [M1b result](../physical-sound-v13-m1b-exposure-ledger-v0-result-2026-08-31.md),
   [V13 research](../physical-sound-v13-canonical-modal-field-research-2026-08-31.md),
   [C4a result](../physical-sound-r3a-v12-c4a-object41-real-frf-fit-result-2026-08-31.md),
   [C4 protocol](../physical-sound-r3a-v12-c4-object41-real-frf-fit-protocol-2026-08-31.md)
   and [Roadmap V12](../../plans/physical-sound-synthesis-roadmap-v12.md).
4. [C3 result](../physical-sound-r3a-v12-c3-object41-source-role-freeze-result-2026-08-31.md),
   [C2 result](../physical-sound-r3a-v12-c2-internet-source-zero-decode-inventory-2026-08-31.md),
   [C1 result](../physical-sound-r3a-v12-c1-acquisition-coverage-oracle-result-2026-08-31.md)
   and [B1R3 result](../physical-sound-r3a-v11-b1r3-local-modal-support-result-2026-08-31.md).
5. [Main product roadmap](../../roadmap.md) for scheduling/promotion facts.

## Handoff

- **Workspace:** V12 C1–C4a code/evidence are in Git; datasets, PCM, arrays,
  reports and generated audio remain external.
- **Isolation:** object-92 PCM/force/YAML numeric values remain sealed; object
  41 microphone and every protected role also remain sealed.
- **Quality:** no real formula, neural field, validator release, baked atlas,
  admitted domain or runtime integration exists. Clip fallback is authoritative.
- **Next commit boundary:** V14 N1a Dataset Contract V1 and zero-signal
  source-quality/split invariants.
