# Physical sound PS-2 — corpus pre-registration and power plan

Date: 2026-08-27
Status: `PS_2_CONTRACT_COMPLETE / REAL_ACQUISITION_OPEN / PASS_DISABLED`

## Question

Can the research loop freeze exact acquisition axes, leakage-resistant splits,
numeric grouped-risk/coverage targets and unavailable-component behavior before
a new shadow is opened, while keeping all recordings and manifests external and
without creating corpus-admission authority?

## Implemented boundary

`xtask physical-sound-registry corpus-plan` now validates a current-only,
external manifest with:

- exact material, object, geometry revision, support, excitation,
  impact-position and listener axes;
- controlled-real source declarations and hash-closed acquisition/provenance
  records;
- planned independent object, family, source, generator-revision, mutation
  parent and in-domain coverage groups;
- deterministic non-zero dev/calibration/holdout/shadow partitions grouped by
  the exact canonical keys, with calibration-only threshold selection and a
  sealed shadow;
- pre-registered false-pass, unsafe-alternative, detection-power and useful
  coverage targets;
- grouped mutation rejection, severity monotonicity and mandatory
  `FallbackOutOfDomain` for OOD, missing evidence, specialist disagreement or
  unsupported schemas.

Repository-local inputs and outputs, unknown fields, malformed hashes, missing
controlled-real sources, split leakage and incomplete policies reject before a
report is published. The report can only say `PlanPowerSufficient` or
`PlanPowerInsufficient`; it explicitly has no corpus or formula admission
authority.

The analysis is deterministic and bounded. Validation and hashing cost is
linear in declared domains, axes and referenced bytes. Sample sizing searches
at most 1,000,000 groups. It combines a two-sided 95% Wilson interval with the
exact zero-failure detection probability
`1 - (1 - unsafe_risk)^n`; no random fitting or learned inference occurs.

## Frozen planning result

The external revision is under:

`/home/kaifaty/.codex/experiments/nextengine/physical-sound/ps2-corpus-plan-v1/`

It targets controlled impacts on thin-walled open soda-lime-glass vessels and
declares:

- 40 physical objects across four independently acquired object families;
- four acquisition sources/sessions and four generator revisions;
- measured geometry, a versioned 20 mm foam-annulus support fixture and an
  instrumented impact hammer;
- rim and wall-midpoint impacts, two fixed half-metre listener conditions and
  at least three repeats per condition;
- 40 reject mutation parents and 40 in-domain coverage groups;
- maximum grouped false-pass risk `0.10` at 95% confidence;
- unsafe alternative `0.20` detected with power at least `0.95`;
- useful-coverage Wilson lower bound at least `0.80`.

The deterministic analysis produced:

| Quantity | Required | Planned |
| --- | ---: | ---: |
| Reject parents for Wilson false-pass bound | 35 | 40 |
| Reject parents for unsafe-risk detection power | 14 | 40 |
| In-domain groups for useful-coverage lower bound | 16 | 40 |

The result is `PlanPowerSufficient`. It means the declared acquisition would be
large enough if executed exactly; it does not mean the recordings exist or the
validator passes them.

Frozen hashes:

- manifest SHA-256:
  `024cc2c06affb7796130892af751113bd7994cf98c73a83550057a94ff46bcd1`;
- report SHA-256:
  `e082610c90dabff3c7a328df94671dca4f84f46cd629952c3e914ce600a3ea01`;
- research protocol SHA-256:
  `3561135e21676d08f3487a17d1d8ec90a337ba8b372a8c5654ec58c035a6c4ad`;
- acquisition-source plan SHA-256:
  `d59ad7a623184e8020207e886b83ef83cc49bf4cfdd18cc53ef9f901e7f9a435`;
- provenance review SHA-256:
  `c241b6d0153ebc40e2545a9e6659db23b7bd9e3038015c8dc310fa4e7053fd92`.

Two complete reports are byte-identical.

## Dataset research decision

[REALIMPACT](https://samuelpclarke.com/realimpact/) is the strongest current
comparison/import candidate: its authors describe 150,000 real recordings of
50 objects, repeated automated hammer impacts, contact-force profiles and 600
listener positions. The official
[repository](https://github.com/samuel-clarke/RealImpact) publishes preprocessing
and download tooling, but states that the remainder/raw dataset is still being
packaged. The GlassGoblet preprocessed archive is approximately 2.31 GB. Those
facts justify bounded archive inspection, not automatic admission: exact
support/position metadata and recording redistribution terms still need direct
verification.

[ObjectFolder 2.0](https://github.com/rhgao/ObjectFolder) remains a useful
generator/OOD comparison, not matched-real evidence: its official format is an
implicit multisensory object representation queried by surface coordinate and
force vector.

The first admitted-family candidate is therefore a new independently governed
controlled acquisition. REALIMPACT may become a second source only after its
actual archive metadata passes the same exact manifest and provenance review.

## Verification and next action

Focused corpus-plan tests cover known sample sizes, sufficient/insufficient
decisions, split leakage, missing real source, external path enforcement and
byte-repeat. All 44 focused physical-sound tests, Clippy with warnings denied,
format check and boundary scan pass.

PS-2 is not complete. The next evidence boundary is the smallest controlled
real-family pilot that exercises the frozen manifest without opening shadow:
one acquisition source/session, one object family and enough repetitions to
validate geometry/support/force/listener ingestion. Pilot data can debug the
pipeline but cannot reduce the pre-registered 40-object/35-parent release
requirement. Automatic `Pass` and AV-P0D remain disabled.
