# Physical sound PS-2 — internet corpus acquisition policy

Date: 2026-08-27
Status: `ACTIVE_CONSTRAINT / AV_MSF_E3_CATALOG_MEASURED / INDEPENDENT_SOURCE_EXPANSION_NEXT / PASS_DISABLED`

## Decision

The physical-sound research loop MUST NOT depend on the product owner or another
local operator striking, holding or recording glass or any other test object.
Microphones, instrumented hammers, force transducers and capture rigs are not
user prerequisites and are not roadmap blockers.

PS-2 obtains real evidence from already published internet-accessible datasets,
papers, project archives and their official metadata. Research tooling downloads
or imports those artifacts into an external cache, verifies them and converts
them into the current external corpus inventory. Network access remains an
offline research-tool capability; production runtime and cooked playback never
access the network or research corpus.

The complete synchronized microphone/force/metrology bundle remains a supported
import shape when an external publisher supplies it. It is an evidence tier, not
an instruction to build a local rig. The frozen PS-2 V1 acquisition plan and its
hashes remain historical power-analysis evidence; its instrumented-hammer setup
is no longer the active execution plan.

## Claim-scoped evidence tiers

An internet source receives credit only for the claims supported by its actual
bytes and metadata. Missing fields are never inferred from a paper, filename,
nearby object or another dataset.

| Tier | Minimum published evidence | Allowed use | Not established |
| --- | --- | --- | --- |
| `E1 synchronized response` | Raw microphone and calibrated force traces, repeat identity, geometry, support, impact/listener coordinates and calibration revisions | Absolute excitation-response, modal/spatial calibration and the lower tiers | Nothing outside the declared axes |
| `E2 transfer response` | Force-deconvolved or normalized impulse response with object geometry and impact/listener coordinates | Mode frequency, decay, relative participation, radiation/spatial fitting | Absolute force-to-amplitude mapping and missing material/support/repeat facts |
| `E3 identified recording` | Real recording with stable object/family/material, source and repeat provenance | Material/object identity, envelope, spectral evolution and validator coverage | Force, impact-position, geometry or radiation causality |
| `E4 synthetic/generated` | Reproducible solver or generator revision with exact lineage | Numeric controls, mutations, OOD and source-model regression | Real identity or perceptual-naturalness evidence |

A domain admission may combine independent tiers for separate specialist claims,
but every required claim must retain its source and capability identity. A
missing capability yields `FallbackOutOfDomain` for that claim/domain; it does
not upgrade weaker evidence and does not erase legitimate lower-tier research
use. Grouped holdout/shadow isolation and the pre-registered risk/coverage bounds
remain mandatory.

## Internet-data pipeline

The external source pipeline now implements these stages:

1. discover candidate sources through official dataset pages, papers and
   repositories;
2. freeze canonical source URL, publisher/project identity, retrieval date,
   declared revision, attribution/license review and expected hashes/sizes;
3. fetch into an external content-addressed cache with bounded archive and
   payload validation; recordings, models and generated features never enter
   the repository;
4. adapt each published format into normalized audio/force/geometry/condition
   components without filling absent axes;
5. emit an auditable capability matrix and assign only claim-scoped evidence
   credit;
6. construct object/family/source-disjoint partitions and re-run the frozen
   power analysis before opening a new shadow.

Unknown or incompatible redistribution terms keep the artifact external and
non-distributable; they do not permit copying it into the repository or shipped
content. Provenance, attribution and source identity remain attached to every
derived feature and report.

See the measured [internet source/cache pilot](physical-sound-internet-source-pipeline-ps2-2026-08-27.md)
and [AV-MSF multi-object E3 pilot](physical-sound-av-msf-e3-multiobject-pilot-ps2-2026-08-27.md).

## Current consequence

REALIMPACT GlassGoblet is useful `E2 transfer response` evidence because its
published archive exposes force-deconvolved 48 kHz responses, mesh, impact
vertices and listener coordinates. Its unavailable raw force, material
composition, repeat identity and fixture revision still prohibit `E1` credit.
The complete frozen AV-MSF demo-card surface supplies ten typed-adapter-backed
`E3 identified recording` object groups and twenty impacts. It contains two
Glass objects/four recordings, but every card shares one
publisher/project/revision source group and therefore remains in one `dev`
partition. ObjectFolder remains `E4` synthetic/generated comparison evidence
unless a separate published real recording supplies the missing real-response
claims.

The generic registry, bounded HTTPS fetch/cache, capability matrix, E3 adapter
and leakage-safe identified-corpus normalizer are implemented. Measured Glass
coverage is `2/16` required object groups, so the immediate blocker is
`INDEPENDENT_PUBLISHER_COVERAGE_AND_REALIMPACT_E2_ADAPTER`, not force hardware,
generic networking or more cards from the same AV-MSF revision. If published
evidence cannot satisfy a domain's required claims and grouped sample sizes,
that domain remains fallback-only. The project does not resolve the gap by
asking the user to make physical recordings.
