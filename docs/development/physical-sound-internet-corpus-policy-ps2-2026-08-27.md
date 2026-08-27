# Physical sound PS-2 — internet corpus acquisition policy

Date: 2026-08-27
Status: `ACTIVE_CONSTRAINT / FIVE_E3_PROJECT_GROUPS / GLASS_7_OF_16 / REJECT_PARENTS_8_OF_35 / TWO_REALIMPACT_E2_OBJECTS / E3_AND_REJECT_EXPANSION_NEXT / PASS_DISABLED`

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
   when an official page contains dynamic operational fields, freeze a
   source-specific bounded canonical identity projection rather than pretending
   that volatile HTML is an immutable artifact;
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

See the measured [internet source/cache pilot](physical-sound-internet-source-pipeline-ps2-2026-08-27.md),
[AV-MSF multi-object E3 pilot](physical-sound-av-msf-e3-multiobject-pilot-ps2-2026-08-27.md),
[independent YCB Impact E3 pilot](physical-sound-ycb-independent-e3-pilot-ps2-2026-08-27.md),
[independent Heller Impact E3 pilot](physical-sound-heller-independent-e3-pilot-ps2-2026-08-27.md),
[Greatest Hits discriminator](physical-sound-greatest-hits-discriminator-ps2-2026-08-27.md),
[REALIMPACT E2 adapter](physical-sound-realimpact-e2-adapter-ps2-2026-08-27.md),
[GreenGoblet bounded-range E2 pilot](physical-sound-realimpact-green-goblet-range-pilot-ps2-2026-08-27.md),
the [Freesound glass-bowl E3 pilot](physical-sound-freesound-glass-bowl-e3-pilot-ps2-2026-08-28.md),
the [Freesound wine-glass cached E3 increment](physical-sound-freesound-wine-glass-e3-pilot-ps2-2026-08-28.md)
and the [explicit reject-parent import](physical-sound-explicit-reject-parent-import-ps2-2026-08-28.md).

## Current consequence

REALIMPACT GlassGoblet and GreenGoblet are useful `E2 transfer response`
evidence because their published archives expose force-deconvolved 48 kHz
responses, meshes, impact vertices and listener coordinates. A typed adapter
now freezes the official repository and one exact row per object instead of
trusting generic metadata. GreenGoblet is retrieved by a frozen 1.61 MB
bounded-range path rather than a 2.31 GB full download. Their unavailable raw
force, material composition, repeat identity and fixture revision still
prohibit `E1` or corpus-admission credit.
The complete frozen AV-MSF demo-card surface supplies ten typed-adapter-backed
`E3 identified recording` object groups and twenty impacts. It contains two
Glass objects/four recordings, but every card shares one
publisher/project/revision source group and therefore remains in one `dev`
partition. ObjectFolder remains `E4` synthetic/generated comparison evidence
unless a separate published real recording supplies the missing real-response
claims.

The frozen YCB Impact robot component now supplies that first independent E3
project group: Wineglass and Skillet lid contribute eight exact repeated Glass
recordings. Its OSF-specific fetch policy validates one hash-bound redirect;
generic redirects remain disabled. The upstream `train`/`test` labels are
preserved as source metadata and do not become NextEngine holdout assignments.

The CMU Heller Impact Events item supplies a third project group. Its explicit
`Marbles Dropped in Glass Vase` directory contributes one Glass object/event
group and five repeated PCM16 recordings through a bounded archive adapter.
Mirror, red-vase and different-impactor variants are excluded rather than
receiving inferred material/object identity.

Freesound pack 14905 supplies a fourth project group and one explicitly named
medium-pitched Glass bowl. Eight wood-strike HQ MP3 previews are exact-hash
bound and validated as gapless 44.1 kHz stereo streams. A bounded canonical
page-identity projection removes volatile CSRF/download fields while retaining
the author, pack, description, sound-card and license identities. The source
is CC BY-NC 4.0 and remains external research only; it receives no geometry,
support, force, position, listener or transfer-response credit.

Freesound pack 41981 supplies a fifth publisher/project/revision group and one
pack-specific wine-glass family. Three numbered knife-strike HQ previews are
exact-hash bound and validated as gapless 44.1 kHz stereo streams. The page has
no authored pack description, so its canonical identity freezes an empty
description plus the author, pack and all twelve sound-card identities. CC0
permits redistribution with notice, but bytes remain external. Current-host
direct bounded fetch receives HTTP 403; two imported cache roots validate
byte-identically without weakening proxy/endpoint policy, so this is cached E3
and not an online-repeat claim.

The generic registry, bounded HTTPS fetch/cache, capability matrix, E2/E3
adapters and leakage-safe identified-corpus normalizer are implemented.
The combined measurement has five publisher/project/revision groups, fifteen
objects and forty-four recordings. Glass coverage is now `7/16` required
object/family groups and twenty-eight recordings. Explicit role mode
cross-checks every `target` or `reject_parent` assignment against adapter
material evidence: eight non-Glass object groups/sixteen recordings are now
development-only reject parents. Twenty-seven of the required thirty-five
reject parents and nine Glass groups remain open, so the immediate blocker is
`REMAINING_9_INDEPENDENT_E3_GROUPS_AND_27_REJECT_PARENTS`, not force
hardware, generic networking, OSF/Figshare redirects, REALIMPACT adapter
existence, bounded REALIMPACT row retrieval or unqualified variants from the
same source revisions. Greatest
Hits range inventory found 382 Glass-labelled actions across 31 videos, but the
published labels have no stable object identity and ordinary TLS verification
for the archive host currently fails; it therefore receives no E3 credit and
must not be downloaded or grouped by video. The GreenGoblet ranged ZIP package
is now validated and adds a second E2 object but no E3 group. The next bounded
package resumes internet search for stable object-level Glass identities and
repeats plus hard non-Glass parents from another published E3 project. It must
not move the current small development set into calibration/holdout/shadow or
treat parent availability as measured validator rejection. ObjectFolder-Real remains a strong
candidate but its official 34–39 GB per-batch gzip tar streams do not expose a
bounded path to a second repeat after large embedded media; do not retry
growing archive prefixes unless an official per-object audio-only or seekable
route appears. If published
evidence cannot satisfy a domain's required claims and grouped sample sizes,
that domain remains fallback-only. The project does not resolve the gap by
asking the user to make physical recordings.
