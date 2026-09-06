# SPEC-45: Proposed physical sound synthesis and acoustic presentation

| Field | Value |
|---|---|
| ID | SPEC-45 |
| Status | Proposed |
| Version | 1.02 |
| Last verified | 2026-08-31 |
| Normative dependencies | [SPEC-00](00-product-contract.md), [SPEC-01](01-system-architecture.md), [SPEC-08](08-audio-navigation-and-world-services.md), [SPEC-12](12-vertical-slice-conformance.md), [SPEC-24](24-content-catalog-bundle-and-neutral-asset-schemas.md), [SPEC-26](26-physics-world-collision-constraints-queries-and-canonical-snapshots.md), [SPEC-30](30-presentation-extraction-and-render-content.md), [ADR-027](adr/027-physics-motor-and-animation-layering.md), [ADR-046](adr/046-consumer-driven-contracts-and-current-only-alpha-formats.md), [ADR-058](adr/058-physx-only-deterministic-humanoid-training-substrate.md), [ADR-071](adr/071-canonical-physics-material-lineage.md) |
| Related research | [Physical sound synthesis research, 2026-08-26](../development/physical-sound-synthesis-research-2026-08-26.md), [quality evaluation](../development/physical-sound-quality-evaluation-research-2026-08-26.md), [automated validation](../development/physical-sound-automated-validation-research-2026-08-27.md), [AV-P0B corpus benchmark](../development/physical-sound-corpus-benchmark-av-p0b-2026-08-27.md), [AV-P0C controlled mutations](../development/physical-sound-validator-av-p0c-2026-08-27.md), [steel calibration](../development/physical-sound-steel-calibration-2026-08-26.md), [wood/glass calibration](../development/physical-sound-wood-glass-calibration-2026-08-26.md), [controlled glass corpus](../development/physical-sound-controlled-glass-corpus-2026-08-27.md), [PS-2 internet corpus policy](../development/physical-sound-internet-corpus-policy-ps2-2026-08-27.md), [PS-2 internet source/cache pilot](../development/physical-sound-internet-source-pipeline-ps2-2026-08-27.md), [PS-2 AV-MSF E3 pilot](../development/physical-sound-av-msf-e3-pilot-ps2-2026-08-27.md), [PS-2 AV-MSF multi-object E3 coverage pilot](../development/physical-sound-av-msf-e3-multiobject-pilot-ps2-2026-08-27.md), [PS-2 independent YCB Impact E3 pilot](../development/physical-sound-ycb-independent-e3-pilot-ps2-2026-08-27.md), [PS-2 independent Heller Impact E3 pilot](../development/physical-sound-heller-independent-e3-pilot-ps2-2026-08-27.md), [PS-2 Greatest Hits discriminator](../development/physical-sound-greatest-hits-discriminator-ps2-2026-08-27.md), [PS-2 typed REALIMPACT E2 adapter](../development/physical-sound-realimpact-e2-adapter-ps2-2026-08-27.md), [PS-2 Freesound glass-bowl E3 pilot](../development/physical-sound-freesound-glass-bowl-e3-pilot-ps2-2026-08-28.md), [PS-2 Freesound wine-glass cached E3 increment](../development/physical-sound-freesound-wine-glass-e3-pilot-ps2-2026-08-28.md), [PS-2 explicit reject-parent import](../development/physical-sound-explicit-reject-parent-import-ps2-2026-08-28.md), [PS-2 declarative Freesound adapter](../development/physical-sound-declarative-freesound-adapter-ps2-2026-08-28.md), [PS-2 ObjectFolder-Real interactive-demo E3 pilot](../development/physical-sound-objectfolder-real-demo-e3-pilot-ps2-2026-08-28.md), [PS-2 YCB vertical reject-parent expansion](../development/physical-sound-ycb-vertical-reject-expansion-ps2-2026-08-28.md), [PS-2 REALIMPACT Blue Bowl cross-tier E2 increment](../development/physical-sound-realimpact-blue-bowl-cross-tier-ps2-2026-08-28.md), [PS-2 REALIMPACT Shell Plate bounded-range E2 pilot](../development/physical-sound-realimpact-shell-plate-range-pilot-ps2-2026-08-28.md), [PS-2 Kronland Glass E3 expansion](../development/physical-sound-kronland-glass-e3-expansion-ps2-2026-08-28.md), [PS-2 REALIMPACT Skull Cup bounded-range E2 pilot](../development/physical-sound-realimpact-skull-cup-range-pilot-ps2-2026-08-28.md), [PS-2 SoundPacks Glass E3 and split audit](../development/physical-sound-soundpacks-glass-e3-and-split-audit-ps2-2026-08-28.md), [PS-2 Kronland reject expansion and split freeze](../development/physical-sound-kronland-reject-split-freeze-ps2-2026-08-28.md), [PS-2 exact-domain E2/E3 claim matrix](../development/physical-sound-domain-claims-matrix-ps2-2026-08-28.md), [PS-2 internet-source feasibility and transfer route](../development/physical-sound-internet-source-feasibility-ps2-2026-08-28.md), [PS-2 REALIMPACT transfer calibration](../development/physical-sound-realimpact-transfer-calibration-ps2-2026-08-28.md), [PS-2 REALIMPACT multi-listener acquisition](../development/physical-sound-realimpact-multilistener-acquisition-ps2-2026-08-28.md), [PS-2 REALIMPACT vertical spatial calibration](../development/physical-sound-realimpact-spatial-calibration-ps2-2026-08-28.md), [PS-2 REALIMPACT multi-object spatial-axis extension](../development/physical-sound-realimpact-spatial-extension-ps2-2026-08-28.md), [PS-2 REALIMPACT shape-conditioned spatial calibration](../development/physical-sound-realimpact-shape-spatial-calibration-ps2-2026-08-28.md), [PS-2 REALIMPACT frequency-conditioned spatial calibration](../development/physical-sound-realimpact-frequency-spatial-calibration-ps2-2026-08-28.md), [PS-2 REALIMPACT modal-radiation representation diagnostic](../development/physical-sound-realimpact-modal-radiation-representation-ps2-2026-08-28.md), [PS-2 analytical boundary-solver control](../development/physical-sound-bem-analytical-control-ps2-2026-08-28.md), [PS-2 Pitcher causal audit](../development/physical-sound-pitcher-causal-audit-ps2-2026-08-28.md), [PS-2 Iron selector/tail diagnostic](../development/physical-sound-realimpact-selector-tail-diagnostic-result-ps2-2026-08-28.md) |
| Neural strategy | [Offline neural acoustic field strategy, 2026-08-30](../development/physical-sound-neural-acoustic-field-strategy-2026-08-30.md) |
| Replaces | SPEC-45 1.01; records the reproducible R3A V4 fit-representation rejection and selects the already-permitted offline neural authored-asset route without authorizing quality, admission or runtime model use |
| Latest evidence | [R3A V4 fit probe and neural rebaseline](../development/physical-sound-r3a-v4-fit-probe-and-neural-rebaseline-2026-08-31.md) |

## Status and decision boundary

This SPEC defines a candidate presentation layer that synthesizes source audio
from committed physical excitations. It does not add a current public schema,
runtime crate, save segment, `WorldDynamics` owner, roadmap work package or
shipping claim. The accepted clip-based `AudioSceneSnapshotV1` and
`AudioMixerV1` path remains the production baseline and the mandatory
fallback.

The candidate is intentionally narrower than a universal procedural-audio
system. Its first useful consumer is rigid-body contact sound: impact first,
then bounded rolling and scraping. Cloth, fluids, fire, fracture and biological
sound production are separate source-model tracks. They may reuse the same
bounded excitation and mixer interfaces after their own owners and evidence
exist, but they are not implied by the rigid modal vertical.

### Candidate offline neural field and cooker boundary

The primary research successor is an external neural acoustic field, not a
runtime waveform generator. It may learn from published, hash-closed geometry,
impact, listener and audio evidence and predict:

- bounded object-global modal frequencies and damping;
- impact- and listener-conditioned modal gains;
- a compact coloured residual descriptor;
- calibrated coverage and OOD evidence.

The first vertical cooks this output into canonical sorted/quantized modal,
gain and residual coefficients and renders it through the deterministic
reference path. Its first learned axis is contact position at one declared
canonical listener condition. Detailed object radiation/listener variation is
a later independent capability claim; environmental attenuation,
spatialization and propagation remain SPEC-08 responsibilities. Training data,
model weights, optimizer state, feature caches, generated WAVs and validator
inference remain external. Runtime loads neither the neural model nor the
research registry; invalid, unavailable and OOD conditions select the authored
clip fallback.

The existing Q30 modal renderer, synthetic FEM/BEM controls and frozen DCT
coloured-residual path remain the classical baseline, synthetic teacher,
compact output representation, exact runtime reference and negative controls.
An object-specific few-shot field precedes any shared zero-shot claim. A direct
waveform model may be evaluated only as a report-only perceptual upper bound or
an authored-asset source.

This v1.00 change refines a `Proposed` research route. It adds no current public
schema, runtime dependency, ProductCheck or roadmap activation and therefore
does not supersede an Accepted architecture decision. Production promotion
still requires a concrete consumer and the ADR-046 workflow.

Non-normative implementation note (2026-08-26): an isolated P0/P0.5
laboratory now exists in `next_presentation::physical_sound_lab`. The `xtask
physical-sound-lab` command emits external 48 kHz audition WAVs, and the
reference demo can mix committed `Begin` contacts behind the explicit Cargo
feature `physical-sound-lab`. It uses 12-mode steel/dry-hardwood banks and one
four-mode small-glass clink with a fused sub-1.5 ms non-fracture onset, plus a
same-excitation external audition set for thin-goblet, bottle and thick-jar
body hypotheses. The latter is deliberately separated diagnostic content, not
three promoted material classes or a replacement for the current glass
profile. A second off-by-default feature, `physical-sound-selected-glass`,
explicitly routes the reference demo's committed `Begin` contacts to the
product-owner-accepted 16-mode, 48 kHz Q30 thin-container calibration with a
144-sample cooked onset. The base feature still selects Glass-H, and no
generated WAV, DiffSound dependency or external model state enters the
repository. The laboratory also uses a
provisional adjacent-snapshot speed estimator because the current contact
record lacks impulse/effective-mass and material fields. The small
heterogeneous CC0 screens
improved bounded descriptor sets but are not a controlled corpus, universal
material profiles or substitutes for human audition. This experiment does not
implement the candidate content records, does not satisfy a P1 ProductCheck and
does not relax the production block below. Disabling the feature preserves the
ordinary clip baseline.

The same isolated experiment now includes `xtask physical-sound-eval`. Its
current-only external AV-P0A manifest declares one generator/source domain,
an exact authored fallback, expected impact/silence probes and bounded
`exact_wav_repeat`, `force_response` and `position_continuity` relations. The
command runs deterministic hard-signal mutation ladders and emits only `Pass`,
`Reject` or `FallbackOutOfDomain`. `Pass` means that the complete rigid-impact
control matrix and hard gates passed; it does not claim subjective naturalness,
material identity, real-corpus fidelity or P1 readiness. Missing controls,
unsupported source families and invalid reference evidence cannot request a
person to approve the candidate; they select `FallbackOutOfDomain`. A seeded
blind A/B bundle remains an explicit opt-in diagnostic and never affects the
decision.

A later external P0 checkpoint adds one exact-geometry synthetic glass-vessel
corpus with 15 train/force/position holdout conditions. The clean-room offline
FEM solve, source arrays and WAVs stay outside the repository; only its recipe,
validators and engine-owned recurrence are retained. Q30 numeric transfer and
declared force/position controls pass, while a simple spatial-IDW baseline is
measurably insufficient and all perceptual judgments remain human-gated. This
adds controlled evidence only; it does not identify real glass, implement the
candidate content records or satisfy a P1 ProductCheck.

Follow-up automated-validation research rejects per-candidate human audition as
the target promotion workflow. AV-P0A now implements the deterministic first
layer, but learned acceptance remains disabled. Automatic acoustic-quality
admission still requires grouped real-corpus holdouts, mutation/OOD calibration
and bounded selective risk in AV-P0B/P0C. Historical `NeedsHumanAudit` reports
remain accurate descriptions of the retired v0 evaluator, not current
content-cooker state.

The isolated `xtask physical-sound-benchmark` checkpoint now implements the
AV-P0B external manifest and frozen-feature-matrix boundary. It verifies exact
WAV/provenance-review/feature hashes, enforces object/family-disjoint
development, calibration, holdout and shadow partitions, and measures
classical or external features against a real-only development gallery.
Material-identity sources explicitly carry `material_identity_only` scope and
cannot fabricate spatial/listener/force evidence. Public research recordings
whose redistribution terms are not reviewed may be measured only as attributed
external local inputs with `NOASSERTION` and
`no_repository_or_distribution`; they are not distributable content.

The first frozen measurement uses 15 real wood/metal/glass recordings, 30
published generated variants, an official pretrained BEATs representation and
13 Next Engine shadow candidates. BEATs separates the 15 real objects under the
frozen split but classifies only `8/13` engine candidates; current wood is
`3/3`, steel-as-broad-metal is `0/3` and glass is `5/7`. Every report remains
`NoAcceptanceAuthority`. The corpus is small and material-only, the classical
head disagrees on selected Q30, and no selective-risk threshold is calibrated.
This checkpoint supplies real failure evidence, not a perceptual-risk or
AV-P0C acceptance gate.

The repository now also contains the first measured AV-P0C substrate. `xtask
physical-sound-registry` validates an external hash-closed formula/domain index
and deliberately has no `Pass` value or acceptance authority. `xtask
physical-sound-mutations` builds external hash-closed controlled negatives for
stationary white/coloured tails, frozen spectral evolution and shuffled
amplitude envelopes. The corpus benchmark retains the frozen AV-P0B descriptor
as a separate feature profile, adds deterministic temporal-spectral evolution
features and reports calibration-only thresholds plus parent-grouped holdout/
shadow false-pass risk. The first three-object-per-split measurement has only
`1/3` holdout real coverage, `1/3` grouped holdout and shadow false passes and a
`0.7923` 95% Wilson upper risk bound. It therefore remains
`NoAcceptanceAuthority`; in particular, the shuffled-envelope counterexamples
require a separate amplitude-envelope specialist and broader real families. The
[implementation plan](../plans/2026-08-27-physical-sound-domain-admission-implementation-plan.md)
defines the remaining P0C/P0D and production-promotion boundaries.

The external-only `physical-sound-registry internet-sources` checkpoint now
validates official-source metadata, exact artifact hashes and byte counts,
bounded credential-free HTTPS retrieval, content-addressed caching and
artifact-backed `E1`--`E4` capability claims. Only an implemented adapter may
validate a claim; opaque or discovery-only bytes grant no acoustic evidence.
The first pilot verifies small hash-closed ObjectFolder metadata and its `E4`
synthetic lineage. The first ObjectFolder-Real monolithic archive route remains
discovery-only because its first official acoustic archive is 36.37 GB and has
no publisher-provided SHA-256; the archive was not downloaded. A second typed adapter validates one immutable
AV-MSF page card and two finite float32 recordings for Object 95 (`Glass`), and
grants exactly `E3IdentifiedRecording`. It does not grant force, geometry,
position, support, calibration or transfer-response evidence. Both checkpoints
have no corpus-admission authority and do not alter the clip fallback or
production block.

The current-only `physical-sound-registry identified-corpus` checkpoint now
reruns that exact source audit offline and normalizes all ten object cards and
twenty recordings exposed by the same frozen official AV-MSF page. It derives
publisher/project/revision, object and recording groups, rejects partition
leakage within one project revision and measures target-material coverage
against the frozen PS-2 plan. The complete page contributes ten object groups
but only one project/revision group; Glass contributes two object groups and
four recordings against a minimum of sixteen. All entries therefore remain in
development, calibration/holdout/shadow remain unopened, and the result is
`DevelopmentCoverageMeasured / NO_CORPUS_ADMISSION_AUTHORITY`. Missing force,
geometry, position, support, composition and transfer axes remain unavailable
rather than receiving placeholder metadata.

The next current-only checkpoint adds the independently published YCB Impact
Sounds robot component through `ycb-impact-identified-recording-v1`. The
adapter freezes one exact object/material workbook plus Wineglass and Skillet
lid identities, four repeated horizontal-poke recordings per object and their
exact OSF file IDs, byte counts and SHA-256 values. The `.ogg`-named payloads
are validated by their actual RIFF/WAVE float32 stereo 48 kHz structure. A
source-specific `osf_storage_v1` fetch policy permits exactly one validated OSF
redirect to the expected hash object in the approved Google storage bucket;
generic redirects remain disabled. The two YCB objects grant only
`E3IdentifiedRecording` and remain in development. Combined with AV-MSF the
measured corpus has two publisher/project/revision groups, twelve objects and
twenty-eight recordings; Glass reaches four object groups and twelve
recordings against the frozen minimum of sixteen groups. Calibration, holdout
and shadow remain unopened, and the result still has no corpus-admission
authority.

The following current-only checkpoint adds the CMU AuditoryLab Sound Events
Database through `heller-impact-identified-recording-v1`. It freezes the
versioned KiltHub Impact Events item, one exact audio archive and its recording
notes, and credits only the explicitly named `Marbles Dropped in Glass Vase`
event as one Glass object group with five repeats. Mirror and red-vase events
are excluded because their reviewed metadata does not explicitly identify
Glass, and different impactors on one target do not create new object groups.
A source-specific `figshare_kilt_hub_v1` policy validates one exact redirect to
the approved CMU bucket, while a bounded in-process ZIP reader validates unique
safe paths, compression/size limits and exact per-WAV hashes before PCM16
inspection. Combined AV-MSF, YCB and Heller coverage is three
publisher/project/revision groups, thirteen objects and thirty-three E3
recordings; Glass is five object groups/seventeen recordings against the
minimum sixteen groups. All entries remain in development, eleven Glass groups
and all reject-parent coverage remain open, and the result has no
corpus-admission authority.

The bounded Greatest Hits discriminator reads the official low-resolution
ZIP64 central directory and only its label entries through byte ranges rather
than downloading 20 GB. It reproduces 46,577 material/action/reaction labels,
including 382 Glass-labelled actions across 31 videos. Those videos are scene
records, not stable object IDs; 28 also contain other material labels, and the
paper permits multiple objects per scene. Video identity therefore cannot be
promoted to E3 object identity. The archive host also fails ordinary TLS chain
verification at the reviewed checkpoint, so no insecure fetch adapter is
added. Greatest Hits receives no E3 credit and Glass remains `5/16`.

The current-only V2 `physical-sound-registry corpus-inventory` boundary now
requires a typed source adapter for force-deconvolved transfer entries. Its
`realimpact_force_deconvolved_transfer_v1` profile freezes the official
REALIMPACT repository commit, five exact upstream source files and five exact
object rows with their metadata, transfer and provenance hashes. It
cross-checks archive/object/mesh/transfer/impact/listener identities and grants
only `E2TransferResponse` capabilities for geometry, positions, object identity,
real recording and the force-deconvolved transfer. Missing raw force,
composition, repeat and fixture revisions remain unavailable, so the entry is
still `FallbackOutOfDomain` and gives no corpus-admission or split credit. V1
inventory reports remain byte-identical.

The frozen `physical-sound-registry realimpact-row` GreenGoblet profile obtains
the second row through validated HTTPS ranges: EOCD, central directory, six
small NPY members, the mesh and a fixed 1 MiB compressed prefix of the large
transfer member. It transfers 1,612,392 bytes, `0.069749%` of the 2.31 GB
archive, and reproduces byte-identical acquisition, row, audition, manifest and
inventory reports. The decoded row-0 impact coordinate exactly matches mesh
vertex 31676. The command accepts no arbitrary source URL or object identity;
another object requires a separately reviewed frozen profile. GreenGoblet is a
second E2 geometry/transfer target, not an E3 group, so Glass coverage remains
`5/16`, `Pass` remains disabled and PS-2 remains open.

The next bounded discriminator rejects the monolithic ObjectFolder-Real archive
as the immediate E3 acquisition route despite its strong object/force/
coordinate metadata. Its
official acoustic batches are 34–39 GB single-stream gzip tar archives, and a
256 MiB bounded prefix remained inside one object's embedded media before a
second impact recording. Prefix growth is not retried unless an official
per-object audio-only or seekable route appears.

Instead, `freesound-glass-bowl-identified-recording-v1` freezes one explicitly
named medium-pitched Glass bowl and eight repeated wood-strike HQ MP3 previews
from pack 14905. The adapter validates exact preview hashes, MPEG-1 Layer III
frames, Xing/LAME gapless counts and 44.1 kHz stereo metadata. Dynamic page
fields are excluded by an explicit bounded canonical pack-identity projection;
two independent online fetches and offline audits repeat byte-identically.
Combined AV-MSF, YCB, Heller and Freesound coverage is four project/revision
groups, fourteen objects and forty-one E3 recordings; Glass is six object
groups/twenty-five recordings against the minimum sixteen groups. CC BY-NC
source bytes stay external.

`freesound-wine-glass-identified-recording-v1` adds a fifth publisher/project/
revision group: three CC0 tracks numbered `Knife hits wine glass 1` through
`3`. The adapter deliberately records one pack-specific family rather than
claiming that publisher metadata proves one retained physical specimen. Exact
HQ-preview hashes, MPEG/Xing/LAME structure and a canonical pack identity
repeat across two imported external cache roots and three offline corpus
audits. The current host's fail-closed direct route receives HTTP 403 from the
canonical Freesound address, so this is cached E3 and not an independent-online-
fetch claim; proxy bypass is not enabled. Combined coverage is five groups,
fifteen objects and forty-four recordings; Glass is seven groups/twenty-eight
recordings. All entries remain in development, nine Glass groups and all
reject-parent coverage remain open, and the result has no corpus-admission
authority.

The current-only identified-corpus manifest now supports explicit `target` and
`reject_parent` roles. Enabling roles requires every source to be classified,
and exact adapter material evidence must agree with the role before any report
is published. Seven Glass groups remain targets; eight non-Glass AV-MSF object
groups with sixteen recordings become development-only reject parents. The
report records `8/35`, with all calibration/holdout/shadow reject counts still
zero. This establishes parent identity, not generated negative controls,
validator rejection or false-pass risk; twenty-seven reject parents and nine
Glass groups remain open. Historical implicit manifests retain byte-identical
reports.

`freesound-pack-identified-recording-v1` now separates the repeated mechanical
Freesound checks from pack-specific values. The declarative profile still fails
closed over a canonical pack projection, derived publisher/pack/CDN identities,
exact preview hashes, MPEG/Xing/LAME frame counts, supported license policy and
material-bearing evidence phrases present in publisher metadata. Replaying the
two existing Freesound families through this adapter in two frozen cache roots
keeps all 15 sources evidence-ready and repeats the explicit-role corpus at 15
objects/44 recordings, Glass `7/16` and reject parents `8/35`. The original
pack-specific source and identified reports remain byte-identical. Two newly
discovered Glass-bottle pack candidates are not imported because current raw
Freesound access fails closed with HTTP 403; search-index text cannot supply
canonical bytes or hashes. This checkpoint reduces the next source to a
manifest/evidence operation but adds no object, recording, split or risk credit.

`objectfolder-real-demo-identified-recording-v1` now uses a distinct bounded
route through the official ObjectFolder-Real interactive demos; the rejected
34–39 GB single-stream gzip route remains rejected. Official object-table and
demo-card bindings, immutable repository commits/root trees, exact raw paths,
SHA-256, computed Git blob SHA-1 and bounded ISO BMFF/MP3 structure validate
five objects and fifteen recordings. Two Glass objects and three non-Glass
parents raise the explicit corpus to six project/revision groups, twenty
objects/fifty-nine recordings, Glass `9/16` and reject parents `11/35`. All
groups remain in development; seven target and twenty-four reject-parent groups
remain open. Unknown redistribution terms keep the bytes external, and the
checkpoint grants no force, geometry, position, support, split, admission,
quality or production credit.

The additive vertical profile of `ycb-impact-identified-recording-v1` uses the
official per-object `Known_Objects`/`Unknown_Objects` tree rather than the
non-Glass horizontal folders whose clips are aggregated only by material. It
freezes three objects in each of nine non-Glass primary material classes and
two unique publisher files per object. A bounded in-process Ogg/Vorbis parser
validates page checksums/sequence, one logical stream, Vorbis headers, stereo
44.1 kHz identity, EOS and a 60-second frame ceiling. The 27 objects/54 files
raise the combined E3 corpus to 47 objects/113 recordings and explicit
reject-parent coverage to `38/35`; Glass stays `9/16`. This satisfies only the
development group-count minimum. The new objects share the existing YCB
project/revision, all partitions beyond `dev` remain unopened and no
single-impact segmentation, negative-control, false-pass, quality or admission
credit is created. Seven target groups plus complementary E2/E1 axes remain
open.

The latest `physical-sound-registry realimpact-row` checkpoint adds a second
frozen range profile for REALIMPACT `6_Bowl`. The REALIMPACT/ObjectFolder
numeric identity binds it to the existing ObjectFolder object 6 `Blue_Bowl /
Glass`, whose interactive-demo source already supplies three E3 recordings.
The new profile transfers `1,734,304` bytes (`0.072330%`) of the 2.40 GB
archive, validates a `3000 x 230215` transfer array, `47,738`-vertex mesh and
row-0 impact at exact mesh vertex `35950`, then reproduces acquisition and V2
inventory outputs twice. This creates the first same-object E2/E3 Glass anchor
and the third typed REALIMPACT E2 object, but no new E3 group: Glass remains
`9/16`, reject parents remain `38/35`, every split beyond `dev` remains closed
and the row is `FallbackOutOfDomain` because force bytes, composition revision,
repeat identity and fixture revision are unavailable.

The fourth frozen range profile, `shell-plate-row-0-v1`, binds REALIMPACT
`51_ShellPlate` to numeric ObjectFolder object 51 `Fruit_Bowl / Glass`. It
transfers `1,700,993` bytes (`0.072607%`) of the 2.34 GB archive, validates a
`3000 x 210424` transfer array, a `48,070`-vertex mesh spanning approximately
`298 x 299 x 41` mm and row-0 impact at exact mesh vertex `15341`. Two online
acquisitions and two V2 inventory audits are byte-identical. The generator now
reads material family from the frozen profile instead of hard-coding Glass,
preventing later non-Glass profiles from inheriting the wrong label. This is a
distinct fourth E2 geometry/transfer object only: Glass remains `9/16`, reject
parents remain `38/35`, all splits beyond `dev` remain closed and the row is
still `FallbackOutOfDomain` with the same four unavailable claim axes.

The next bounded E3 checkpoint reuses the official Kronland material-impact
stimuli previously measured by AV-P0B, but now imports only the five numbered
`Glass N original` PCM16 recordings through
`kronland-material-identified-recording-v1`. The official page distinguishes
each original from its published synthesized and tuned derivatives; two
independent online caches match each other and the earlier AV-P0B hashes. The
adapter grants only material, numbered-object, real-recording and exact
recording identity. It raises the combined corpus to seven project/revision
groups, 52 objects and 118 recordings; Glass reaches `14/16` objects and 39
recordings while reject parents remain `38/35`. Every entry remains in `dev`,
two Glass targets and every project-disjoint split remain open, and no shape,
composition, force, position, support, transfer, admission, quality or
production credit is created. The safe HTTPS resolver now prefers public IPv4
when a host publishes both address families, after validating every resolved
address as public; pinned resolution, byte bounds and hash closure are
unchanged.

The following bounded source audit does not weaken E3 identity. ObjectFolder's
remaining `fork_vis` demo contains only objects 36 `Polycarbonate` and 52
`Steel`; its material-classification benchmark exposes object-keyed processed
spectrograms in an 81.95 GB bundle rather than original recordings. The YCB
Impact OSF release publishes object-named vertical recordings with no Glass
object and material-grouped horizontal Glass clips without per-clip object
identity. These routes remain discovery-only. The fallback
`skull-cup-row-0-v1` profile instead binds REALIMPACT numeric object 60 to the
ObjectFolder `Beer_Glass / Glass` row and validates a `3000 x 209549` transfer,
`47,810`-vertex mesh and exact row-0 vertex `2764` after transferring
`1,670,815` bytes (`0.071751%`) of the 2.33 GB archive. Two acquisitions and
two V2 audits are byte-identical. This is the fifth typed REALIMPACT E2 object,
remains `FallbackOutOfDomain`, and leaves E3 Glass at `14/16`, reject parents at
`38/35` and every split beyond `dev` closed.

The next current-only checkpoint imports two pack-specific Glass object/family
groups from the published SoundPacks `Glass Recordings` archive through
`soundpacks-glass-recordings-identified-recording-v1`. A canonical page
projection removes dynamic recommendation state, a source-specific MediaFire
resolver binds the expiring download URL to one exact file key, and a bounded
pure-Rust RAR5 reader validates the readme plus four `drinking glass` and three
`metallic vase` float32 stereo WAVs by embedded hash. Two independent online
caches and an offline replay repeat byte-identically. The combined development
corpus is now eight project revisions, 54 objects and 125 recordings; Glass
reaches `16/16` objects and 46 recordings, while reject parents remain `38/35`
objects and 79 recordings.

Aggregate counts alone do not open a partition. The current-only
`physical-sound-registry split-feasibility` audit applies the frozen
`20/30/25/25` plan to whole project/revision groups and requires target plus
reject-parent evidence in each partition. All eight projects carry targets, but
only AV-MSF, YCB and ObjectFolder carry reject parents. The exact result is
`ProjectDisjointSplitInfeasible`: three reject-bearing projects cannot cover
four partitions. All entries remain in `dev`, `Pass` and PS-3 stay disabled,
and the next evidence increment must add at least one independent reject-bearing
project rather than more target-only Glass.

The following current-only checkpoint expands the same official Kronland
project with its five numbered Wood originals and five numbered Metal
originals. A canonical `kronland_material_page_identity_v1` projection retains
the publication identity and all fifteen original/synthesized/tuned track
bindings while excluding rotating WordPress state; the earlier five-Glass
manifest still emits its previous report bytes. The ten object-bound real
recordings add only E3 material/object/recording identity and remain external.
The combined corpus now contains eight project revisions, 64 objects and 135
recordings. Glass remains `16/16` objects and 46 recordings; reject parents
increase to `48/35` objects and 89 recordings. Kronland becomes the fourth
dual-role project, so a repeated feasibility audit returns
`ProjectDisjointSplitFeasible` with no blockers.

`physical-sound-registry split-freeze` then binds the pre-split E3 report,
feasibility report and corpus plan, and applies the frozen seed through bounded
`seeded_sha256_first_feasible_backtracking_v1`. It assigns two complete
project/revision groups to each partition while requiring target and
reject-parent presence. Independent post-split audits and entry-by-entry
verification repeat byte-identically: `dev/calibration/holdout/shadow` contain
`30/6/12/16` objects and `70/20/27/18` recordings. The decision is
`ProjectDisjointSplitVerified`, but its claim remains split structure only. It
does not fill absent geometry/support/excitation/listener axes, measure
selective risk, authorize corpus admission, open shadow to optimizer feedback
or enable `Pass`, PS-3 or AV-P0D.

`physical-sound-registry domain-claims` now binds that verified partitioned E3
report, all five typed REALIMPACT E2 reports and the frozen plan into an
executable exact-domain matrix. Its V1 identity adapter accepts only the
reviewed Blue Bowl `6_Bowl` / ObjectFolder object 6 bridge; numeric equality is
not a generic object link, so REALIMPACT `94_GlassGoblet` cannot borrow
ObjectFolder object 94 `Salad_Bowl`. The repeated matrix is
`DomainEvidenceIncomplete / FallbackOutOfDomain`: object identity, real
recordings, E3 repeats and one force-deconvolved transfer are supported, while
composition revision, planned geometry/support/excitation/impact/listener
alignment, matched-condition lineage and four-partition exact-domain coverage
remain unsupported. This result makes the next research query precise; it does
not weaken the missing axes, move frozen partitions, release a validator or
authorize `Pass`.

`physical-sound-registry source-feasibility` now binds the incomplete V1
domain report to frozen primary-source snapshots for REALIMPACT, ObjectFolder
Real and AV-MSF. The repeated current-only report is
`ReviewedSourcesCannotCloseV1`: all eight exact-domain blockers remain open.
REALIMPACT is selected only as
`realimpact-normalized-transfer-calibration-v1`, with credit for relative
force-deconvolved transfer, impact/listener identity and scanned geometry.
Absolute amplitude, exact composition/support, matched cross-tier conditions,
corpus admission, `Pass` and runtime content remain prohibited. ObjectFolder
Real force calibration is deferred and AV-MSF remains method-only until its
code/data lineage is published. This narrows the next experiment without
weakening the fallback-only V1 result.

`physical-sound-registry transfer-calibration` now makes that transfer-only
experiment executable. The frozen first revision is retained as
`INVALID_METRIC_CONFOUND`: non-injective tail assignment and shared coarse
damping bins inflated its apparent holdout recall, so its formal threshold
result receives no modal credit. A separately hash-closed V2 fixes the
extractor before opening the reserved Skull Cup, selects a 16-mode injective
profile on Shell Plate and crosses every preregistered relative
modal-frequency/damping gate on the fresh Skull Cup holdout. Its decision is
`RelativeModalDampingSupportedSpatialUnavailable`. Exactly one listener row
per object makes spatial participation not evaluable. This result is not glass
identity or naturalness evidence and does not close any of the eight exact
domain blockers, release a validator, authorize `Pass` or add a content/runtime
role.

`physical-sound-registry realimpact-row --profile
green-goblet-listener-block-0-v1` now closes the narrower acquisition question
for one development object. The frozen preprocessing revision and published
annotation arrays prove that rows `0..14` share one Green Goblet impact vertex,
azimuth and distance while spanning 15 distinct microphone positions. Two
bounded acquisitions produce byte-identical typed manifests, reports and raw
blocks. The decision is `MultiListenerAcquisitionPilotOnly`: no participation
model is fitted, no calibration or spatial holdout is opened and no spatial
credit is granted. A multi-object split, candidates, metrics, gates and exact
archive ranges must be preregistered before reading further Shell Plate or
Skull Cup listener rows. Exact-domain, quality, admission and runtime boundaries
remain unchanged.

`physical-sound-registry spatial-calibration` now executes the separately
preregistered multi-object discriminator. Green Goblet is development-only;
Shell Plate selects among four frozen vertical interpolation candidates; the
selection snapshot is hashed before Skull Cup listener rows `1..14` are
opened. The selected `vertical-rbf-sigma052-ridge001-v1` candidate crosses all
six Skull gates and improves median held-listener error over the microphone-7
constant baseline by a ratio of `0.7623`. The improvement-component fraction
is exactly its `0.5` threshold, so the result supports only relative selected-
mode magnitude along one vertical listener line at one angle/distance. It does
not support a 3D radiation field, arbitrary listener geometry, absolute
amplitude, material identity, naturalness, admission, `Pass` or runtime use.

`physical-sound-registry spatial-extension` now tests that fixed RBF without
retuning across four distance strata, three observed angles and two additional
objects. Green development passes all six declared condition blocks. The
hash-closed evaluation requires both Blue Bowl and Glass Goblet to pass; Glass
passes `4/6`, while Blue passes only `3/6` and fails its base block because the
RBF is nearly indistinguishable from the constant baseline. The decision is
`FixedVerticalCandidateTwoObjectAxisStratificationRejected`. Blue and Glass
remain immutable rejection evidence, not new tuning data. Any next spatial
formula must be shape-conditioned, use a fresh object-disjoint split and retain
per-object/clip fallback; the negative result grants no angle/distance
interpolation or 3D-field credit.

`physical-sound-registry spatial-shape` implements that fresh split for one
minimal candidate: an object-level RBF bandwidth predicted from bounding-box
scale/aspect and normalized impact position. Ten development objects produce a
non-constant bandwidth target, but the immutable two-object calibration returns
`MeshConditionedBandwidthCalibrationRejected`: both candidate condition gates
pass and p90 improves, while median ratio to the coordinate-only control is
`1.0122` against the frozen `0.95` limit and maximum object ratio is `1.0242`
against `1.0`. The two declared holdouts remain unopened. Coarse mesh geometry
alone is therefore insufficient; a next candidate may test frequency-dependent
acoustic scale on a new calibration split, but still grants no spatial/runtime
credit until its own preregistered holdout passes.

That per-mode successor is now rejected too. Twelve opened development objects
fit `sigma(f, bbox, impact)` under manifest `88cac5bc…09f`; fresh calibration
manifest `92ebe6a7…42f8` binds the fit before `100_Frisbee` and
`32_WoodChalice` are opened. Both absolute condition gates pass, but median
ratio is `1.0146` against `0.95` and maximum object ratio is `1.0266` against
`1.0`; reports repeat at `42b6605d…983`. The frequency coefficient collapses
near zero and the candidate behaves like the fixed control. Do not tune another
RBF bandwidth. The next bounded research discriminator must preserve per-mode
complex response and test a compact mode-shape/radiation basis before any
surface-mode plus PAT/BEM-style cooker is proposed. The two ceramic holdouts
remain unopened and all spatial/runtime credit remains disabled.

The compact complex discriminator is rejected as well. Manifest
`686c1d42…0ff` freezes fourteen already-open blocks before their complex modal
responses are inspected. An axisymmetric order-three outgoing multipole basis
passes all fourteen absolute condition gates but loses to magnitude RBF:
median object ratio `1.2506`, maximum ratio `2.0861`, maximum p90 regression
`+7.2513 dB` and improved fraction `0.3482`; reports repeat at
`7cbf7c59…f25`. Do not raise order or retune empirical harmonics on these rows.
Before any fresh REALIMPACT payload, the next package must prove a classical
surface-mode-to-acoustic-transfer pipeline on an analytical or independently
checkable synthetic fixture. Neural acceleration remains downstream of a
hash-closed classical target and error budget. Ceramic holdout and all
spatial/runtime credit remain disabled.

The first analytical boundary-solver control is reproducible but formally
rejected. Its `320`-panel pulsating-sphere field passes the frozen absolute
complex (`2.1251%` maximum), magnitude (`0.1827 dB`), phase (`0.2993°`) and
direction-symmetry (`0.0002 dB`) gates, while its median error is `2.8589x` the
`80`-panel result instead of at most `0.8x`. Both reports repeat at
`6f74a309…a689`. The coarse result therefore cannot be used as evidence of
convergence, and the fine absolute match alone does not create a trusted
BEM/FFAT oracle. Keep all REALIMPACT holdouts sealed; distinguish
geometry/quadrature error from a boundary-equation defect on synthetic data or
an independent classical implementation before any real-data calibration.

The first remediation rejects ordinary higher-order panel quadrature as the
missing ingredient. A frozen symmetric seven-point rule preserves all absolute
field gates but yields fine median error `1.0447x` the three-point control and
fine/coarse ratio `2.6109` against `0.8`; reports repeat at
`7460750e…b48d` and historical V1 remains exact at `6f74a309…a689`. Stop
regular quadrature variants. The next control must use an independent
Galerkin implementation with singular treatment—currently frozen to Bempp-cl
`0.4.2` revision `a1eaaef9…e1c0`—on the same analytical sphere before any
surface-mode or REALIMPACT promotion.

The independent control now passes. Bempp-cl `0.4.2` at revision
`a1eaaef9…e1c0` uses a direct hypersingular/adjoint-double-layer solve on
`128/512`-panel spheres. Fine maximum complex error is `1.2710%`, magnitude
error `0.1111 dB`, phase error `0.4844°`, direction span `0.0074 dB` and
fine/coarse median ratio `0.2625`; all GMRES and field gates pass. Reports are
byte-identical at `ba638a21…01f1`. Credit is limited to a pinned analytical
sphere oracle. Before fresh REALIMPACT access, freeze one non-spherical
prescribed surface mode and require the repository cooker to reproduce its
near/far directional field.

That directional surface-mode checkpoint now passes without real-data access.
The final V3 protocol applies `P2(cos(theta))` through a radial pullback on
`128/512`-panel spheres, covers 22 directions including exact nodes, and
repeats at report `e8e1d4d5…6437`. Fine maximum peak-normalized error is
`4.5546%`, nodal leakage `1.293e-6`, directional correlation `0.9999978` and
fine/coarse ratio `0.2713`; all frozen field gates pass. A repository-owned
order-three outgoing-multipole cooker then fits seven directions only at
`1.5a` and predicts all 22 directions at `3a/10a`. Its repeated report
`054901ee…f871` concentrates at least `99.9907%` coefficient energy in degree
two and reaches `0.7856%` maximum held peak-normalized error. Credit is limited
to one analytical axisymmetric surface-mode class. The following triaxial
checkpoint satisfies the required non-spherical prescribed-mode cross-check;
FEM coupling, real 3D transfer, quality, admission and runtime remain blocked.

The next checkpoint removes the spherical and axisymmetric controls. A
triaxial `0.08/0.10/0.13 m` closed mesh carries the prescribed radial-pullback
mode `2 u_x u_z`; Bempp levels `128/512/2048` converge across 336 field
conditions with `0.03305` maximum fine/medium peak-normalized difference and
`0.26654` median refinement ratio. Reports repeat at `49bee8c2…76ef`. The
repository full-angular cooker retains a sparse-near-shell protocol as an
immutable rejection (`0.06032 > 0.05` at the omitted near angles, report
`6d5f653a…ef55`) instead of weakening its threshold. A separately frozen
product-shaped protocol fits the complete 56-direction near shell and holds
out the complete `4L/10L` shells. It rejects the `m=0` control and selects the
smallest passing full-angular candidate, degree two, at `0.04604` maximum
peak-normalized error and `0.9990417` minimum correlation; reports repeat at
`a0e0d881…b0be`. This supports one synthetic non-spherical prescribed-mode
near-to-far representation only and defined the following elastic FEM coupling
checkpoint. REALIMPACT and ceramic holdouts, real-object/material/quality/
admission/runtime credit and `Pass` remained sealed or disabled.

The synthetic elastic coupling checkpoint now passes too. A deterministic
core-clamped linear tetrahedral FEM on the same triaxial solid solves and
tracks one non-axisymmetric radiating eigenmode across `128/512/2048` surface
panels. The original coarse schedule remains an immutable rejection at report
`a58f28b9…90ef`. A separately frozen refined schedule passes all 16
eigenfrequency, mode-match, surface-profile, eigensolver, GMRES and Bempp-field
gates and repeats at `a71515fa…a667`; its selected frequencies converge from
`388.960` through `363.091` to `355.651 Hz`. The unchanged full-near-shell
cooker then fits 56 directions at `2L`, holds out 112 conditions at `4L/10L`,
rejects the axisymmetric control and selects full degree two at `0.030975`
maximum peak-normalized error and `0.9996932` minimum correlation. Reports
repeat at `c3101130…b476`. This closes one synthetic FEM-eigenmode-to-acoustic
representation prerequisite only. It does not identify a real material,
support or object and grants no quality, admission, runtime or ProductCheck
credit. Any fresh real-data step must be separately preregistered with an
object-disjoint split, immutable gates and per-object/clip fallback before its
reserved payload is opened.

That preregistration now exists without opening a reserved payload. Manifest
`5be5f195…e576` binds `65_PitcherCeramic` as calibration and
`63_SmallPlanterCeramic` as a one-shot holdout by their previously frozen
roster-hash order. It binds both ZIP/payload/mesh identities, a `90`-anchor /
`510`-held 3D listener split, the fixed coordinate RBF and constant controls,
conjunctive frequency/solver/spatial/comparison gates and per-mode/object
fallback. The single candidate is a deliberately non-authoritative cotangent-
biharmonic surface proxy projected through pinned Bempp and the already proven
full-angular cooker; it explicitly grants no elastic-shell, wall-thickness,
support or material identity. Two validator runs repeat at report
`c2f51cff…01ef` and record zero network requests and zero reserved audio bytes.
This freezes protocol and opening order only. A byte-identical geometry-only
preflight is next and must fall back before calibration audio if either reduced
mesh fails the frozen topology or numeric gates. Real transfer, quality,
admission, runtime and ProductCheck credit remain disabled.

The frozen geometry-only V1 then rejects before acoustic access. Both hashed
REALIMPACT OBJ entries are triangle soup: Pitcher has `48418` vertices for
`16140` faces and `16139` connected components; Planter has `47732/15912` and
`15910`. Unmodified `8192/2048` simplification preserves thousands of
components, so the closed-manifold gate returns `GeometryUnsupportedFallback`
for both. Two reports repeat at `2fb9fd0f…e25d`; exactly `1308328` compressed
mesh bytes and zero reserved audio bytes are read. A post-rejection,
metadata-only discriminator shows that exact bitwise coordinate welding—no
tolerance or repair—recovers connected closed surfaces with `8070/7958`
vertices and zero boundary/nonmanifold edges. V1 remains rejected. The next
package may preregister only that exact-weld preprocessing change while keeping
all reductions, eigenmode/Bempp/cooker gates and the sealed opening order
unchanged.

The separately frozen exact-weld V2 now passes without audio. Manifest
`85ca065b…f46f` groups only bitwise-identical parsed coordinates and changes no
downstream reduction, eigenmode or field-transfer gate. Pitcher/Planter become
connected closed `8070/7958`-vertex surfaces, retain valid `8192/2048`-face
spectral/BEM meshes and produce 64 modes with maximum residuals
`3.141e-13/4.446e-13`. Report `c1c86f78…7e5d` and geometry blocks
`bcd54087…9acc` / `9310910f…5431` repeat byte-identically; reserved audio bytes
remain zero. This grants deterministic geometry setup only. Before Pitcher
audio, a separate calibration manifest must bind the exact block/prefix,
decoder, scale/mapping, Bempp/cooker inputs, `90/510` split, controls, gates and
stop-before-Planter fallback.

That Pitcher-only calibration manifest is now frozen too, still without audio
access. Manifest `c60621cc…4a7` binds geometry block `bcd54087…9acc`, one exact
`536870912`-byte compressed prefix, the `<f4 [3000,230470]` decoder and rows
`0..599`, the bound injective 16-mode extractor, the one-refit 16-of-64 dynamic
program, pinned Bempp/cooker inputs, `90/510` split, controls and every stop
condition. The repository validator also binds the relevant implementation
source hashes and requires parity evidence before a port may open the prefix.
Two reports repeat at `2ae1bc0b…9c72` with zero network requests and zero
reserved audio bytes. Prefix growth, threshold/decoder retry and Planter access
are prohibited. This remains calibration preregistration only; measured real
transfer and all broader credit stay disabled until a bounded repeated Pitcher
execution passes. Even then a separate Planter holdout manifest is required.

The first executable runner preflight is now independently repeatable. An
exact Rust fixture using the bound extractor emits report `2e3db2d3…5572` and
sample block `c8316f81…0477`; the Python port reproduces all 16 modes with
maximum absolute difference `3.02336e-12`. It also validates the exact geometry
block, 16-of-64 dynamic program and full `90/510` coordinate split. Preflight
reports repeat at `6e60d71f…fd2d` with zero network and audio bytes. A separate
Rust helper includes the bound spatial DSP for the future 600-row projection,
avoiding an approximate Python substitute. Audio execution remains disabled in
this revision. A final execution manifest must still bind Bempp environment,
direction artifact, expansion origin and staged acquisition/decode bytes before
the single Pitcher request.

That final execution revision is now frozen and locally repeated too. Manifest
`8e791327…ba45` binds script `c5900a9c…bbfa`, the exact original-mesh bbox
centre and diagonal, all 56 directions, `2L/4L/10L` shells, Python/Bempp/NumPy/
SciPy/Numba versions, DP0/P1 spaces, assemblers, GMRES bounds, raw-DEFLATE NPY
decoder, exact 512-MiB range and every preregistered gate. Two preflight reports
repeat byte-identically at `94d5e1e6…9b32`; extractor parity remains
`3.02336e-12`, and an exactly representable degree-4 outgoing field recovers at
less than `2e-13` maximum held peak-normalized error. Network requests and
reserved Pitcher/Planter audio bytes remain zero. This authorizes only the one
frozen Pitcher acquisition; it creates no measured-transfer credit and does not
allow retry, prefix growth or Planter access.

The one authorized Pitcher request has now been spent successfully. Acquisition
report `899fbbe9…c819` binds one `206` response and compressed prefix
`a0dd7006…6cf5`; decode report `29496c6f…9eef` binds the exact 600-row,
`553128000`-byte block `182f2010…1e0f`. The first offline analysis completed
the heavy computations but failed before publication because a NumPy comparison
boolean was not JSON serializable. No calibration decision exists. Repair
manifest `603c1185…28e3` fixed comparison-result conversion but then rejected
the correct parent-bound decode lineage before audio-block access. It remains
immutable negative repair evidence. Successor `f51a6046…db7e` additionally
accepts only decode report `29496c6f…9eef` bound to original execution manifest
`8e791327…ba45`, while future analysis output binds the successor itself. It
still prohibits `acquire` and `decode` and changes no numeric path or gate. Two
preflights repeat at `9c5c9ca8…471c`. Offline analysis repeat is next; Planter
remains sealed and no physical-transfer credit is available.

The repaired Pitcher calibration now completes twice and rejects the combined
extractor/scalar-proxy/Bempp/cooker protocol byte-identically at report
`8bd5323c…1aea`. Thirteen modes satisfy its numeric solver/coverage admission,
but frequency mapping misses at median/p90 `0.5664/1.6260` octave. Every held
3D stratum fails: medians are `19.90–23.44 dB`, p90 values are
`45.24–57.39 dB`, and the candidate is `2.7039×` worse than the frozen RBF
median with `+27.58 dB` p90 regression. The exact Rust projection also repeats.

Read-only causal audit `68c79a37…5a57` then applies the already-frozen transfer
V2 observation thresholds without selecting them from Pitcher. The observation
fails because `decaying_mode_fraction = 0.4375 < 0.50`, although the physical
protocol consumed it; `11/16` selected peaks are also below the paper's
`500 Hz` less-anechoic-room boundary. The immutable combined-protocol rejection
therefore remains valid, but it cannot uniquely attribute failure to scalar
mechanics or acoustic transfer. Bempp convergence is not the numeric failure.
Planter MUST remain sealed, the authored clip fallback remains active, and the
next separately frozen test MUST run complete observation admission first on
unopened development object `78_CeramicCup`. Only an admitted observation may
enter a later vector shell/hollow-volume FEM discriminator. Opened Pitcher
values, gates and cooker degrees MUST NOT be tuned.

That object's discovery is now executed and verified from immutable cache:
acquisition `2b183dae…782b` and offline audit `cd68ba79…1b0b` bind the exact
12-entry archive structure with zero payload bytes. Successor manifest
`71123b21…cae5`, runner `bad26592…60e2` and repeated preflight
`5f34993f…8f21` freeze two metadata ranges, one `512 MiB` audio prefix, the
600-row decoder, reference row `7`, extractor parity and all five unchanged V2
observation gates. One exact three-request acquisition is authorized next.
Physics execution, threshold tuning, further audio and all Planter payloads
remain prohibited unless the cached observation passes in repeated analysis.

That execution now rejects the observation at report `56591bb8…3fd9`: four
unchanged V2 gates pass, but decaying-mode fraction is `0.25 < 0.50`, and
`13/16` selected peaks are below `500 Hz`. Together with Pitcher's
`0.4375` / `11-of-16` result, this blocks another object or mechanics attempt.
Only a separately frozen offline diagnostic across fixed Ceramic Cup listener
axes may run next; it may not tune the gate, select a listener, denoise, access
the network or open Planter.

Diagnostic manifest `e7b952fe…375b`, runner `92d51c02…3ecb` and repeated
preflight `6e97bc0a…066c` now bind exactly 27 already decoded height/angle/
distance rows, the immutable parent/block, unchanged V2 analysis and frozen
listener-local/shared/low-frequency classification. Two offline analyses are
authorized next. A passing row cannot replace row `7`, and the diagnostic may
not tune, denoise, fetch, run physics or open Planter.

Repeated result `47b578ac…2603` classifies the failure as shared: `23/27` rows
fail, including `15/15` height, `7/10` angle and `3/4` distance rows. The
listener-local hypothesis is rejected. The frozen low-frequency association is
also rejected: fitted-decay fractions are `0.3583` below and `0.2252` at/above
`500 Hz`. The next authorized work is synthetic only: freeze and test a
multi-output spatial-energy decay estimator against known modal decay and a
node-contaminated single-output control. Real Ceramic reanalysis, another
object, validator change, mechanics and Planter remain prohibited.

Synthetic-control manifest `cd8ee856…2078`, runner `f0483c39…2068` and
repeated preflight `88b017a1…1153` now bind 15 outputs with known 16-mode
frequency/decay truth, a near-node/delayed channel-7 comparator and one candidate
that sums modal power across outputs without changing V2 windows or thresholds.
Two synthetic executions are authorized. Any gate failure rejects the revision;
only a complete repeatable pass may authorize a separately frozen read-only
Ceramic counterfactual. No real row, physics or Planter access is authorized.

Two executions now emit byte-identical report `a099f50d…e8bea` and pass every
frozen gate: spatial decay fraction `1.0` versus node-channel `0.0`, known-decay
median error `0.35979 dB/s` and tail RMSE `0.34787 dB`. This supports the
estimator only on the declared synthetic counterexample. The next permitted
step is to hash-close, but not yet execute, a read-only counterfactual over the
existing Ceramic Cup block and all 15 microphones at the fixed reference
impact. Fetching, row selection, mechanics and Planter remain prohibited.

Counterfactual manifest `add0017b…25e1`, runner `b5635c42…fb7a` and repeated
preflight `7bf54ec8…e0e1` now bind the existing Ceramic decoded-block identity,
rows `0..14` at the fixed `0°/0 mm` condition, unchanged reference row `7`, the
successful synthetic implementation and six conjunctive gates. Two read-only
analyses are authorized after this checkpoint is committed. They must rehash
the block before use; any gate failure rejects the counterfactual. No fetching,
row tuning, admission, mechanics or Planter access is authorized.

Two real analyses now emit byte-identical report `9947c427…96cbf` and reject
the counterfactual: spatial decay fraction is `0.1875` versus reference
`0.25`, although the other four V2 gates pass. Primary setup/code evidence
rejects incompatible impacts across the 15 synchronized microphones. It also
shows that REALIMPACT fits a bandpassed RMS envelope over a mode-adaptive
peak/noise interval, unlike the fixed V2 `50–900 ms` regression. Freeze a
synthetic source-faithful adaptive-decay control before any further real reuse;
do not apply a frequency cutoff, fetch, run mechanics or access Planter.

Adaptive-control manifest `926921e2…cedf`, runner `fe59b3d6…9c20` and repeated
preflight `f51e513a…7b49` now bind a delayed/noisy 15-output known-truth fixture,
source-derived bandpass/RMS-envelope interval, fixed-window comparator and all
conjunctive gates. Two synthetic executions are authorized. Any failure rejects
the revision; only a repeatable full pass may authorize a separately frozen
Ceramic adaptive counterfactual. Real payload, fetching, mechanics and Planter
remain prohibited.

Two executions now emit byte-identical report `9fabc2bd…0297f` and pass every
check: all 16 adaptive fits are valid, median decay error is `0.68081 dB/s`,
median `R²` is `0.999812`, and error falls by `58.27233 dB/s` versus the frozen
fixed-window comparator. This remains synthetic method evidence. The next
permitted step is to hash-close, but not yet execute, a read-only Ceramic
adaptive counterfactual over the same rows and parent lineage.

Ceramic adaptive manifest `48001fb7…ff9a`, runner `e50bec23…056b` and repeated
preflight `4358d4da…220f` now bind the same decoded identity and rows `0..14`,
the successful adaptive implementation, rejected fixed fraction `0.1875` and
seven checks. Two read-only analyses are authorized after commit; each must
rehash the block. Any failure rejects, while a pass still requires an
independent unopened-object validation before admission or mechanics.

Two analyses now emit byte-identical report `2ec3b03e…d6b7` and reject the
counterfactual: only `6/16` components have valid dynamic range, decay fraction
is `0.375`, and improvement is `0.1875`. The six fits peak within `9 ms` and
decay near `-362 dB/s`, so they cannot be promoted as mechanical damping.
Ceramic Cup is closed to further method development. Freeze a synthetic
source-faithful ±`10%` salience-selector discriminator before a separately
preregistered unopened object; mechanics and Planter remain prohibited.

Salience-control manifest `e11ffd56…72f1`, runner `62043142…4ed2` and repeated
report `fa940710…0b3e` now support the source-derived relative selector on its
declared synthetic counterexample. Fractional ±`10%` dominance followed by a
`900 ms` persistence match recovers `16/16` known modes with no false positive,
while the prior top-16 comparator recovers `4/16`; exact bin identity repeats at
signal scales `0.125`, `1` and `8`. The adaptive estimator remains valid for
`16/16` modes with `0.39520 dB/s` median decay error. This does not reproduce
the source notebook's unavailable absolute normalization and grants no real,
material, mechanics, quality, admission or runtime credit.

Independent-object manifest `a77f6d97…357f`, runner `27b11de8…70f1` and
repeated metadata-only preflight `85c53d80…7ead` freeze official
`17_IronSkillet` as development object. Selection follows the frozen roster and
excludes already referenced/opened or reserved objects. One prior HTTP metadata
request read zero member-payload bytes; the only authorized next access is one
exact archive-tail and 30-byte local-header discovery followed by two offline
audits. Observation member payload, threshold tuning, object substitution,
mechanics and Planter remain prohibited until a separate post-discovery
manifest exists.

The exact discovery now succeeds with acquisition report `b9f659b4…6f6c` and
byte-identical offline audit `a3bd84f1…8dab`. It reads 65536 ZIP-tail bytes and
one 30-byte local header in two requests, verifies 12 central-directory entries
and opens zero observation member bytes. Protocol manifest `573ff0d6…8c93`,
runner `7385241d…8273` and repeated zero-access preflight `19c1f57a…2680`
freeze four future ranges, audio shape `(3000, 230549)`, impact-zero rows
`0..599`, condition identity, the unchanged salience/adaptive sources and seven
gates. No payload request is authorized until a separate execution runner is
implemented, hash-closed, committed and revalidated against that manifest.

Execution manifest `2c98971a…d8ee`, runner `0c53553a…0988` and repeated
zero-access preflight `b9927f17…672b` now bind the four-stage acquisition,
decode and analysis path. The runner derives adaptive working length from the
recording, validates exact condition/row identity and keeps top-16 diagnostic
only. After this checkpoint is committed, one exact four-request acquisition is
authorized. Failure is final for this revision; retry, prefix growth, threshold
change, object substitution, mechanics and Planter access remain prohibited.

The exact acquisition/decode now emit reports `6a99d291…dfeb` and
`47acdc35…c099`; two analyses emit byte-identical `a9c4ae36…206d` and reject
real transfer. Persistence recall is `0.35294 < 0.50`, and only `3/6` selected
modes have valid adaptive fits. Scale invariance and the other five checks pass.
The diagnostic top-16 set has `0.9375` valid/decaying fractions, so absence of
decaying content is not a sufficient explanation. Preserve the synthetic pass
but reject the current real selector pipeline. Before another change, freeze a
bounded existing-block diagnostic that separates ±`10%` dominance from fixed
`900 ms` persistence and research a multichannel transient modal estimator with
known-truth controls. No new object, tuning, acquisition retry, mechanics or
Planter access is authorized.

The preregistered existing-block diagnostic now emits byte-identical report
`02551f29…48d1` and decides `FixedTailTimingMismatchSupported`. The complete
source-derived onset set has `13/17 = 0.7647` valid adaptive fits and therefore
does not support the selector-composition mismatch. Persistence passes at all
three declared earlier starts (`0.7059/0.7647/0.6471` at `100/200/400 ms`) but
falls to `0.3529` at 900 ms; the top-16 comparator falls in the same direction.
Retire 900 ms as a universal survival gate, but do not select an earlier value
from this opened object. Next freeze a synthetic multichannel Gabor/subband
ESPRIT common-pole control with model-order, frequency, damping and node-channel
truth. No new object, real counterfactual, mechanics or Planter access is
authorized before that separate control succeeds.

The separate synthetic common-pole control now emits byte-identical report
`3fcb5fc8…2284f` and passes all eleven frozen gates. Across four declared Gabor
bins it selects orders `[2,2,1,1]`, recovers six known common poles including
two sub-FFT-bin pairs, and limits frequency/decay errors to `0.043014 Hz` and
`0.259092/s`. Output 7 contains a declared node and recovers only one first-band
mode; the 15-output estimator recovers both. Numeric results are exact at
scales `0.125/1/8`. This is synthetic support only for a preselected-subband
core. Before any real reuse, freeze a separate synthetic broad-band control for
input-driven band selection, adjacent-bin duplicate clustering, complex
amplitude fitting, post-estimation pruning and reconstruction/residual gates.
No new object, real payload, mechanics or Planter access is authorized.

The separate broad-band holdout control now emits byte-identical report
`dcd83252…0094` and passes all fifteen frozen gates without receiving truth band
centres. It discovers six regions and 18 analysis bins, removes 12 duplicate
estimates, retains seven strong modes and prunes one deliberately weak nuisance.
Maximum frequency/decay errors are `0.095876 Hz/0.190671/s`; strong-truth,
full-observed and post-transient NRMSE are `0.033913/0.130840/0.049273`.
Input-driven region/bin selection is exact under scales `0.125/1/8`. This
supports only the synthetic discovery-to-resynthesis method. Before any real
transfer claim, freeze a read-only counterfactual on the already acquired Iron
rows with no threshold tuning, new payload, mechanics or Planter access.

That hash-closed existing-Iron counterfactual now emits byte-identical report
`7635f8ac…075d` and rejects at its first capacity gate: input-driven discovery
finds 31 regions and 91 neighboring bins against frozen maxima 16/48. Discovery
is exact under scales `0.125/1/8`; pole estimation and later gates are not run,
so the result neither rejects modal structure nor supports real method transfer.
Do not select the strongest opened regions or raise the cap. Before an Iron V2,
freeze a fresh-seed dense synthetic scaling/discriminator control that separates
legitimate modal density from leakage/nuisance admission and measures the full
post-estimation prune cost. No new real payload, object, mechanics or Planter
access is authorized.

The fresh-seed dense scaling control now emits byte-identical report
`b9a5b226…978b` and passes all 18 gates. Rank-7 partial SVD reproduces the
complete parent core within `9.09e-13 Hz/1.34e-12/s`, then processes 32
regions/96 bins, removes 122 duplicate estimates and recovers all 64 strong
modes with zero false positives or selected weak nuisance regions. Maximum
frequency/decay errors are `0.080465 Hz/0.610466/s`; post-transient NRMSE is
`0.049039`. This is synthetic scaling support only. It authorizes one frozen
Iron V2 over the complete already discovered region set, changing only the SVD
core and 32/96 capacity envelope. No opened-region ranking, new payload/object,
mechanics or Planter access is authorized.

The frozen Iron V2 now emits byte-identical report `f2fb359f…6a73` and passes
all 17 gates. All 31 regions/91 bins are analyzed without ranking; 59 raw
estimates become 31 clusters and 30 retained modes. Even/odd microphone halves
match `0.833333/1.0`; full damped NRMSE is `0.796572` versus `0.991326`
undamped, and predictive damped/undamped error ratios are
`0.005848/0.002251`. This supports method transfer only on one opened real
object. Absolute fit remains modest, so it grants no material, perceptual,
domain, physics or runtime credit. Before any broader claim, freeze and run an
object- and family-disjoint internet-sourced real holdout with unchanged V2
algorithm and gates.

The independent holdout discovery now verifies official REALIMPACT
`43_IronMortar` without opening any member payload. Acquisition report
`84308e8e…d080` uses exactly two ranges; repeated offline audit
`67c92621…8a42` binds the 12-entry central directory and observation entry.
Uncompressed size proves an exact `3000×208375` f32 observation, sufficient for
the frozen V2 windows. This is source/capacity evidence only. A separate
one-impact manifest must bind condition rows, the minimum bounded audio prefix
and unchanged V2 gates before waveform access; no substitution or tuning is
authorized.

That independent execution now emits byte-identical analysis
`64bc63aa…d0cc` and passes all 15 method gates. Iron Mortar discovers 26
regions/77 bins, retains 13 modes and matches `0.846154/0.846154` across the
microphone halves. Full damped NRMSE is `0.762138` versus `0.993895` undamped.
However predictive damped NRMSE is effectively `1.0` in both future windows;
small ablation ratios reflect undamped divergence, not transparent synthesis.
This supports a report-only modal-observation extractor, not a quality pass.
The next allowed package is a deterministic experimental registry builder over
the exact Iron Skillet/Mortar reports, carrying residual metrics and explicit
fallback/admission state. No public schema or runtime consumer is authorized.

Registry runner `a958c795…c064b`, manifest `0f473f4d…05c20` and repeated
preflight `fbffbe1e…441c0` now freeze those two exact JSON reports before the
build. `ModalObservationV0` must retain modal, spatial, fit/prune and absolute
residual facts while marking quality/domain/runtime admission disabled and the
authored fallback required. Listener identity is source output order `0..14`;
coordinates absent from both reports cannot be inferred. The one allowed next
action is a repeated byte-identical external registry build.

That build now emits registry `e0bec857…e09e98` and report
`34a7f209…daed0` twice identically. The two entries preserve 43 retained and
three pruned modal observations plus absolute residuals; both explicitly keep
quality/domain/runtime admission disabled and authored fallback required. The
next permitted experiment is a frozen object/family-grouped batch protocol for
modal-only, transient and residual candidates. The two opened seeds cannot
define their own quality threshold.

Metal-batch runner `ecbaa724…acbfc`, manifest `873d82e3…23ae4` and repeated
preflight `578e143c…aec31` now freeze three development, two calibration, one
holdout and one two-object shadow family. Four existing observations are
hash-closed; four new archive identities have zero member-payload access. The
next allowed action reads only each ZIP tail and observation local header,
then audits that cache before any candidate or waveform access.

Acquisition `d0a051eb…b0468` and repeated audit `5790f8cc…a1219` now verify
all four observation entries using eight requests and zero member-payload
bytes. Inferred shapes range from `3000×208584` to `3000×208736`. The next
allowed change must freeze exact metadata/audio ranges, decoder shapes, all
modal/transient/residual candidate parameters and calibration-only gates before
new waveform access. Prefix growth, retries and per-object human admission stay
forbidden.

Protocol runner `c3d0b406…7e2ea`, manifest `24231974…b724b` and repeated
preflight `bd755741…709e3` now freeze nine metadata ranges, four 32 MiB audio
prefixes, decoder conditions and three candidate representations. Selection is
development/calibration-only; holdout and the two-object Spatula shadow open in
sequence. Relative residual gates are representation evidence only and cannot
enable quality/domain/runtime admission. The next allowed change is the
zero-access execution runner and deterministic fixtures recorded below.

Execution runner `8484f947…13cc`, manifest `bffaa21c…a8664` and repeated
preflight `1af90ed4…e4f68` now freeze the complete role-staged acquisition,
decode, modal baseline, seeded residual, metric and selection path. The
offline fixture selects the two-exponential seeded residual while rejecting
the truncated transient, and repeats with zero new network/member payload.
This authorized only `86_MetalHoledSpoon` fit-role acquisition. Spoon holdout
could not open before an immutable successful selection report; the two
Spatulas still cannot open before an immutable successful holdout. Any failure
keeps authored fallback and quality/domain/runtime admission disabled.

The fit stage now uses exactly two requests and decodes Holed Spoon microphone
rows `0..14`; acquisition `454933ae…163d8` and decode `af7af9fb…e068a` bind
the shared `0°/0 mm` condition. Selection runs A/B emit byte-identical report
`83f858bf…43f2` and select `modal_plus_seeded_subband_residual_v0`. Across the
two calibration objects its median envelope/energy/flatness ratios are
`0.387055/0.709616/0.552735`; the maximum object/metric ratio is `0.943533`.
The truncated transient fails. This freezes representation selection only;
waveform NRMSE remains diagnostic and quality/domain/runtime admission stays
disabled. Only the exact Metal Spoon holdout ranges may open next. Both
Spatulas remain sealed until an immutable successful holdout report exists.

The exact Metal Spoon holdout then uses three bounded responses; acquisition
`6653fea3…2d32c` and decode `5fd5ec04…633` bind microphone rows `0..14` at the
shared `0°/0 mm`, vertex `6840` condition. Evaluation A/B repeats
byte-identically at `fef9dd34…cb73` and rejects the selected representation.
Envelope error improves to ratio `0.534247`, but listener-energy ratio
`0.959237` exceeds the frozen `0.95` per-metric limit and spectral-flatness
ratio `1.053763` exceeds both `0.95` and the frozen `1.05` maximum. Thresholds
and the opened holdout cannot be reused for tuning. Both Spatulas remain at
zero payload; shadow, quality/domain/runtime admission and automatic pass stay
disabled. The next permitted step is a zero-network representation diagnostic
over already opened rows followed, if discriminating, by a separately
preregistered grouped successor—not another variant inside this lineage.

Diagnostic runner `d2ab638b…43f88`, repeated preflight `d2f07a3f…797bf` and
repeated six-object report `d8acc909…ba1eb` identify within-band coloration as
the leading successor hypothesis. Median spectral-shape and short-lag
autocorrelation errors are `7.808400 dB` and `0.330411`, passing both frozen
diagnostic gates. Cross-listener coupling is independently deficient:
candidate/observed median coherence is `0.999177/0.172267` and effective rank
is `1.005682/3.292782`. Temporal modulation fails its conjunction because
modulation-power error is only `1.908039 dB`. No new network, member or shadow
bytes are read. A fresh grouped successor may change only deterministic
within-band coloration; spatial covariance remains a recorded later
hypothesis, Metal Spoon cannot be reused as holdout and both Spatulas remain
sealed.

Successor runner `fe8a12f5…cebf5`, manifest `f03e428e…e564a` and repeated
preflight `4c754f06…a360` now freeze that one change. At most eight bounded
DCT-II coloration coefficients per existing band modify the seeded excitation;
modal extraction, two-exponential envelope, listener gains and rank-one
spatial control remain unchanged. A deterministic fixture reduces spectral-
shape and short-lag autocorrelation errors to ratios `0.313717/0.628724`. Fresh
calibration is the grouped `19_Pan`/`37_PiePan` pair, holdout is `22_Cup`, and
the existing Spatula pair remains shadow with zero member payload. Names do not
grant material identity. The next permitted stage discovers only the three
fresh archive identities; no ZIP tail, member payload, quality/domain/runtime
admission or shadow access is implied.

Identity-discovery runner `898d1af1…c7c58`, manifest `55be5ed2…01abe` and
repeated preflight `0d38bfb1…f5d7b` now freeze exactly three one-shot HTTPS
`HEAD` requests for Pan, PiePan and Cup. Final URL, status, byte length,
`Accept-Ranges`, `ETag` and `Last-Modified` are mandatory; redirect drift or
the first failure stops with no retry. Range requests, response bodies, ZIP
structure, member and shadow payload remain forbidden. The current managed
sandbox did not execute the network stage, so archive identities remain open
and no acquisition success or rejection is claimed.

`physical-sound-registry classical-baseline` now consumes only the unsealed
PS-2N0 train/development projection and requires one explicit hash-closed
binding per row. Its exact selected-glass Q30 snapshot has profile hash
`19b051fe…ef181`; the full-energy canonical WAV remains
`c912806c…b9c823`. It exports canonical recurrence order, the bounded onset
residual, PCM/WAV hashes, shared acoustic features and exact A/B metrics, while
stale projection/audio lineage, sealed roles, reject parents and missing axes
fail closed or select `FallbackOutOfDomain`. The frozen coloured-DCT sources
are also hash-bound, but there is no per-row DCT fit or exact cooked renderer,
so that branch remains `FallbackOutOfDomain` rather than receiving partial
baseline credit. Those exporter synthetic repeats pass; that package itself
added no real projection, training, quality/admission decision, runtime
consumer or public schema.

The compatible real-data boundary now exists separately. PS-2N0 V2 projects
23 internet-sourced rows across five disjoint roles and distinguishes eight
recorded impact waveforms from 15 force-deconvolved Green Goblet transfer
responses. Its fit/calibration/report/commitment outputs repeat exactly;
method holdout and admission shadow remain commitments only. A frozen
fixed-impact listener baseline evaluates seven odd listener positions from
eight even-position contexts using nearest-listener and shortest bracketing
linear interpolation. The repeated linear control records `2.5373 dB` mean
absolute level error and `9.1745 dB` mean gain-matched multi-resolution
spectrum RMSE. These are R1 controls, not quality thresholds or admission
credit. No runtime/public schema is added.

The direct and coordinate-derived propagation-delay-aligned external listener
fields have completed without promotion. Their fixed-seed PyTorch/MLflow
training, checkpoints, cooked PCM and frozen Rust-metric evaluations reproduce.
The phase-aligned rank-4 candidate improves four of five primary endpoints but
still fails the unchanged P95 spectrum criterion; the conjunctive decision is
`RejectListenerField`. A repeated query-informed subspace diagnostic also fails
held-query level/spectrum aggregates, so the opened time-domain latent family
is retired rather than retuned. Method holdout and admission shadow remain
sealed.

The R2B data/representation boundary is now reproducibly ready. Two bounded
internet acquisitions project the same full published 600-position
fixed-impact REALIMPACT semicylinder, and two complete preflights repeat
byte-identically. Complete angle-plane groups produce `420 context / 180
query`; query audio contributes neither feature normalization nor candidate
fit. The frozen complex STFT inverse reaches `-153.348 dB` worst NRMSE and at
most one PCM16 LSB, while nearest, linear and complex interpolation controls
are measured before optimization. Method holdout and admission shadow remain
sealed.

The bounded R2C research boundary is now complete and rejected. The shared
coordinate/time/frequency complex-pressure MLP was trained twice with
Helmholtz weights `0` and `0.0001`; checkpoints, non-MLflow output trees and
reports repeat under the frozen protocol. One-shot evaluation selects neither
candidate: both collapse toward near silence, with about `53 dB` mean level
error and `27 dB` mean spectrum error. Method holdout and admission shadow
remain sealed.

A repeated context-only discriminator rejects Helmholtz regularization and
rank-96 capacity as the primary causes. A linear rank-96 oracle retains
`99.6396%` of context energy, while the trained data-only full-context
objective is `1.0498x` the zero predictor and every logged step reaches the
gradient clip. The next permitted revision is therefore an energy-preserving
objective/optimizer/cooker trainability gate using identity, one-row,
small-block, zero, global-mean and context-only rank controls. It must read zero
query audio. Only a passing trainability revision may authorize one frozen
coordinate-to-low-rank-coefficient field and one grouped query evaluation.
Nearby width, rank, seed, step, Helmholtz-weight or threshold grids over the
opened R2C query are not authorized. This result adds no model-quality,
admission, runtime or ProductCheck authority.

The first query-free R2D trainability revision is now complete and rejected.
It freezes a canonical rank-96 basis retaining `99.6396%` context energy,
repeats all reports/checkpoints byte-identically and uses zero query,
method-holdout or admission-shadow bytes. Identity, one-row, coefficient,
cooker, rank-oracle, clipping and zero/global-mean comparison gates pass. The
eight-row and full-context tasks fail only the unchanged mean absolute
log-energy bound at `0.007785` and `0.005062` versus `0.005`; N0.3E remains
unauthorized. The next permitted revision retains basis, loss, initialization,
steps, tasks, cooker, metrics and thresholds and changes only fixed AdamW
learning rate to one hash-closed deterministic decay schedule ending near
zero. It must repeat twice and still read no query audio. Threshold relaxation,
more fixed-rate steps or concurrent representation changes are not authorized.

That optimizer-only R2D V2 revision is now complete and passes. It binds the
exact V1 manifest and replaces only fixed `0.05` with an inclusive-endpoint
half-cosine `0.05 -> 0.00001` schedule. Two independent runs produce the same
normalized report hash `898ee201…0875`; all checkpoints and 37 prediction WAVs
are byte-identical. Every unchanged coefficient, energy, clipping,
oracle-proximity and real-cooker gate passes, while query, method-holdout and
admission-shadow reads remain zero. This closes context trainability and
authorizes one separately frozen data-only R2E coordinate-to-rank-96-
coefficient research candidate. It does not authorize held-listener quality,
admission, public content/runtime records or production use. R2E may open the
grouped query exactly once only after its candidate and repeat policy freeze;
query-informed tuning, a candidate grid and physics-loss rescue remain
unauthorized.

That sole R2E revision is now complete and rejected. The group-conditioned
cosine coefficient field fits the complete context coefficients and energy to
floating-point noise, and two training runs reproduce reports, checkpoints and
all prediction WAVs byte-for-byte. The one permitted 180-row grouped query is
worse than every frozen classical control on all five primary endpoints and
returns `RejectLowRankCoefficientField`. A separate post-reject query-seeing
projection oracle repeats exactly, retains only `92.38%` query energy at
Frobenius NRMSE `0.2760`, and itself misses the mean-spectrum control. The
bounded conclusion is `RepresentationAndInterpolationBothLimited`.

This closes the opened fixed-impact listener-field family and forbids nearby
rank, architecture, seed, step or threshold search on that query. It does not
reject neural impact sound as a whole. The next permitted Roadmap V4 research
boundary is a new internet-only multi-object/multi-impact corpus at one
declared canonical listener condition, followed by a compact modal/residual
representation-oracle gate before neural training. The first candidate after
that gate may learn geometry/contact-position-conditioned modal gains;
listener radiation requires a separate denser published corpus or independently
validated solver. This change authorizes no quality, admission, public content,
runtime or ProductCheck result.

The first R3A revision is also complete and rejected. Two metadata-only
REALIMPACT Blue Bowl preflights freeze five source-ordered impact locations at
one exact published listener condition; two streaming extractions stop before
the fifth sealed contact; two query-seeing development oracles reproduce
byte-identically. A 512-scalar 32-mode plus sparse-DCT residual record and an
equal-budget sparse-DCT alternative both fail the frozen level, spectrum,
envelope, modal-frequency and decay gate. The exact result is
`REJECT_REPRESENTATION`; no optimizer or neural checkpoint exists.

R3B remains blocked. A subsequent R3A revision must bind a materially different
representation and new unopened development projection before access. Nearby
mode/bin/budget tuning on the opened Blue Bowl contact is negative-knowledge
repetition, not progress. The Blue Bowl field holdout, method holdout and
admission shadow remain sealed. This evidence update preserves the Proposed,
external-only, authored-fallback boundary and authorizes no public record,
runtime model, ProductCheck or production claim.

R3A V2 and V3 are now also closed without readiness. V2 freezes a new
REALIMPACT Large Swan split and a pinned 44.1 kHz DAC oracle, but its
48→44.1→48 kHz control itself fails full-band level and decay; the exact result
is `INCONCLUSIVE_RESAMPLING_CONTROL`. V3A freezes Plastic Bin and stops before
quality metrics when the native NDAC decoder returns `143,992` samples for a
`144,000`-sample target. An eight-sample guard is derived only from synthetic
audio, so Plastic Bin is not re-evaluated.

V3B applies that frozen guard to a new Purple Scoop projection. Two
preflights, sealed extractions and native-48 kHz NDAC-75 oracles reproduce
byte-identically; the identity control is exact zero on all five endpoints.
NDAC preserves absolute level, envelope and decay but fails spectrum at
`12.1198 dB` and modal-frequency median at `560.81` cents. The exact result is
`REJECT_LEARNED_CODEC_REPRESENTATION`; see the
[R3A V3 evidence](../development/physical-sound-r3a-v3-native-ndac-result-2026-08-30.md).
No field holdout is decoded and no neural field, deterministic distillation,
quality, admission or runtime claim is authorized.

R3A V4 is now complete and rejected before development. Two preflights and fit
runs reproduce all JSON/NPY artifacts byte-for-byte. Each of three
task-specific pole/gain/multiresidual capacities meets its shared and
per-contact byte budgets, yet all `12/12` fit contacts fail the frozen spectrum
endpoint. Development, row `2407`, method holdout and admission shadow access
remain zero; see the
[exact evidence](../development/physical-sound-r3a-v4-fit-probe-and-neural-rebaseline-2026-08-31.md).

The next permitted [Roadmap V7](../plans/physical-sound-synthesis-roadmap.md)
boundary freezes an internet-only task-specific neural rate-distortion model
with direct spectral/modal objectives before reading development. A passing
neural waveform decoder may support the already-declared authored-asset route:
an external contact field decodes a frozen grid offline and an asset baker
publishes ordinary bounded clips plus coverage/fallback metadata. It does not
satisfy the modal cooker, enter runtime, authorize a public content record or
remove the authored fallback. Deterministic modal/residual distillation remains
a later optional optimization behind separate evidence.

A production consumer requires a later Accepted ADR under ADR-046. That ADR
must freeze the exact engine-owned projection, content records, limits,
reference numeric profile and ProductChecks. Until then all record shapes and
check IDs below are illustrative candidate semantics.

## Product intent and non-goals

The product goal is more coherent reactive sound for physical interactions:
the same object should respond continuously to where and how it was struck,
rolled or scraped without selecting one event-specific recording from a large
sample table.

The candidate MUST NOT:

- make PCM, mixer state, audio device state or propagation output authoritative;
- derive gameplay hearing from a waveform or presentation voice admission;
- create a second writer for rigid transforms, contacts, materials or topology;
- solve an acoustic pressure field or finite-element eigenproblem at audio rate;
- promise that one small set of material constants synthesizes every acoustic
  behavior;
- require neural inference, network access or an optional propagation SDK;
- remove authored clips, speech, music, ambience or declared failure fallbacks;
- present cloth, liquids, fire, footsteps, vocal tracts or birds as automatic
  consequences of a rigid modal resonator.

## Ownership and data flow

The authoritative and presentation paths remain separate:

```text
committed semantic/world facts
    -> deterministic AcousticFactV1
    -> gameplay hearing / AI perception

committed physics contact batch + exact immutable content revisions
    -> PhysicalSoundExtraction (presentation only)
    -> bounded excitation batch
    -> source synthesis voices
    -> attenuation / optional propagation / spatialization
    -> AudioMixerV1-compatible PCM / device
```

Physics remains sole owner of body state and contact facts. Assets owns
immutable cooked acoustic content. Presentation owns only reconstructible
extractor, resonator, voice, propagation and mixer state. No arrow returns from
the presentation path to Physics, Runtime, RPG, AI state or `WorldCommand`.

| Layer | Input authority | Output | Persistence |
|---|---|---|---|
| Gameplay acoustics | committed semantic/world facts | deterministic `AcousticFactV1` | existing authoritative path only |
| Physical excitation extraction | committed engine-owned physics projection | bounded immutable presentation records | not saved |
| Source synthesis | excitation records plus exact cooked acoustic models | mono or small-channel source PCM | reconstructible voice state |
| Propagation/spatialization | source PCM plus listener/room/portal facts | listener PCM | reconstructible cache |
| Device/capture | mixed PCM | hardware output or canonical capture | never gameplay authority |

Steam Audio or a similar adapter belongs after source synthesis. It models
distance, directivity, occlusion, transmission and reflections; it does not
replace the physical source model. Adapter handles and vendor types remain
private under SPEC-08.

## P0 acoustic knowledge and admission boundary

The unit of research progress is an **acoustic domain**, not an unconstrained
material label or one auditioned WAV. A domain is the bounded Cartesian product
of:

- source class, initially `RigidImpact`;
- object and geometry family;
- acoustic material family and exact profile revision;
- support/boundary condition;
- excitation ranges and declared impact-position set;
- listener/radiation condition set;
- source-model family and exact formula revision.

For example, `thin steel vessel / freely supported / rim and wall impacts` is a
candidate domain. `Steel` by itself is not. A passing domain revision never
silently widens to another geometry, support or excitation range.

The P0 source-model factorization is:

```text
source_pcm(t) = radiation(
  sum(mode_participation(position, excitation)
      * exp(-t / decay)
      * sin(2*pi*frequency*t + phase))
  + bounded_source_residual(t, condition, seed)
)
```

Frequencies, decay and spatial participation are conditioned by the exact
geometry/material/support model. Excitation controls modal amplitudes and any
declared transient. The residual is source-model-specific and cannot be used as
an unconstrained noise term to repair a failed material classifier. Propagation
and listener mixing remain downstream under SPEC-08.

P0 maintains four separately versioned, external-only research artifacts:

1. **Corpus registry** — hash-closed recordings, object/family identity,
   geometry/support/excitation/listener metadata, provenance and immutable
   development/calibration/holdout/shadow partitions.
2. **Formula registry** — source-model family/revision, parameter-schema
   identity, bounded domain envelope, calibration inputs, cost envelope and
   exact authored fallback.
3. **Validator release** — frozen deterministic gates, specialist feature/model
   revisions, mutation families, OOD policy, split identities and selective
   risk/coverage policy.
4. **Domain admission record** — formula and parameter revision, exact domain,
   referenced evidence hashes, measured risk/coverage, cost result and one of
   `Pass`, `Reject` or `FallbackOutOfDomain`.

These records are research evidence, not `crates/contracts` schemas, project
content or shipping assets. They MUST stay outside the repository when they
refer to recordings, datasets, learned weights, generated audio or model
outputs. Repository tooling MAY define and validate current-only experimental
JSON shapes, bounds, ordering and hashes. An admitted production profile later
becomes cooked PresentationOnly content only through a concrete consumer and a
promoting ADR under ADR-046.

For the active PS-2 track, real corpus construction MUST use already published
internet-accessible datasets, papers, project archives and official metadata.
It MUST NOT require the product owner or another local operator to strike,
handle or record physical objects, and local microphones, force transducers or
instrumented hammers MUST NOT be roadmap prerequisites. Offline research tools
MAY access the network to discover and retrieve external inputs into an
external content-addressed cache; production runtime, cooked playback and
mandatory product checks MUST NOT depend on network access or that cache.

Every retrieved source MUST bind its canonical URL, publisher/project identity,
declared revision, retrieval date, byte hashes, provenance and
attribution/license review. Unknown or incompatible redistribution terms keep
the bytes external and non-distributable. Archive extraction and format
conversion MUST be bounded and MUST preserve the source identity of every
derived feature or report.

Evidence credit is claim-scoped rather than all-or-nothing:

- synchronized raw microphone/force/metrology data MAY establish absolute
  excitation-response and the supported modal/spatial claims;
- force-deconvolved or normalized transfer responses MAY establish modal,
  decay, relative participation and spatial claims, but not absolute
  force-to-amplitude mapping;
- identified real recordings MAY establish material/object, envelope and
  spectral-evolution evidence, but not undeclared geometry, force or position;
- synthetic/generated sources MAY establish numeric controls, mutations, OOD
  and source-model regressions, but not real identity or naturalness.

A complete synchronized acquisition bundle remains a supported import shape
when an external publisher supplies one; it is not an instruction to construct
a local capture rig. Multiple source tiers MAY support different specialists
inside one domain only when every claim retains exact source/capability lineage
and the grouped split remains leakage-free. Missing evidence selects
`FallbackOutOfDomain` for the dependent claim/domain instead of inventing an
axis or opening a human/local-capture queue.

Controlled mutations used as negative evidence MUST declare their expected
validator outcome explicitly. Published resynthesis, perceptual tuning or other
unlabelled transformations MUST NOT be inferred to be failures merely because
their origin is a mutation. A controlled mutation MUST preserve its parent's
partition, object/family, material and source identity, and related mutations
MUST be grouped by parent when estimating false-pass risk.

`Pass` is permitted only for the exact declared domain and validator release.
The validator used for admission MUST be frozen before evaluating a new
generator revision, MUST NOT train or calibrate on that generator's holdout or
shadow entries, and MUST publish a measured confidence-bounded selective risk/
coverage result on independent parent/object groups. Threshold choice uses only
the declared calibration partition; holdout and shadow cannot select features,
weights or thresholds. A point estimate with insufficient grouped support is
not an admission bound.
Missing coverage, an unsupported condition or insufficient confidence selects
`FallbackOutOfDomain`; it never creates a human approval queue. A validator
release may invalidate admission under a newer policy only by publishing a new
record. Historical records remain immutable evidence rather than being
rewritten.

The research registry therefore accumulates conditional model/cooker knowledge,
not a universal table mapping `material -> coefficients`. Runtime fitting,
training and validator inference are not implied: P1 cooks an admitted bounded
record and evaluates only the deterministic source representation.

## Candidate content model

The first candidate uses three separate PresentationOnly content roles.
Their exact V1 wire schemas MUST NOT land before the first production consumer.

### `AcousticMaterialProfileV1`

An acoustic material profile describes calibration needed by a sound model,
such as density, elastic constants, frequency-dependent damping, roughness or
friction-exciter coefficients, radiation class and calibration provenance.
Every value has declared units, finite bounds, revision and content hash.

It is separate from `PhysicsMaterialDescriptorV2`. The current physics material
owns contact friction, restitution, rolling/spinning friction and surface
velocity; it does not own Young's modulus, Poisson ratio, modal damping or
acoustic radiation. An exact binding may reference both records without
copying either record into the other or making acoustic calibration affect
collision response.

### `ModalSoundModelV1`

A cooked modal model binds:

- exact source geometry or an explicitly declared acoustic proxy revision;
- exact acoustic material/profile revision;
- cooker and numeric-profile identity;
- sorted bounded modes with frequency, damping and gain;
- a bounded spatial impact-response or participation map;
- a declared radiation approximation and validity range;
- a fallback clip or engine-native fallback class;
- source provenance, license metadata and canonical content hash.

Finite-element analysis, eigensolving and large transfer computation happen in
the cooker or offline research tool. Runtime never solves the eigenproblem.
The render mesh, collision mesh and acoustic proxy may differ, but the binding
must name the exact revisions and cannot infer identity from filenames.

### `PhysicalSoundBindingV1`

The binding maps an exact body/shape/content revision and canonical physics
material tags to one acoustic body/model, priority class and fallback. Multiple
physics shapes may drive one acoustic body only through a declared stable
mapping. Missing, duplicate, stale or hash-mismatched mappings fail before the
model is activated; they do not partially bind voices.

The primary modal branch SHOULD avoid event-specific impact WAV selection.
This does not mean “no sound assets”: acoustic profiles, cooked modal data,
calibration recordings and fallback clips remain ordinary governed content.

## Candidate excitation projection

SPEC-26's normative `ContactEventV1` already names the useful physical facts:
stable contact identity, tick/substep, participants/features, point, normal,
relative velocity, impulse bounds, effective mass and material tags. The
production synthesizer MUST consume an engine-owned quantized projection of
those committed facts, never raw PhysX callbacks, native manifolds, pointer
identity or solver scratch.

The current Rust `ContactEventV1` implementation publishes only a subset of
that normative record: contact identity, participant/features, point, normal,
phase and source-snapshot hash. It does not yet serialize relative velocity,
impulse bounds, effective mass, contact kind or material tags. Therefore a
production physical-audio vertical is blocked until the existing physics
projection gap is closed and its `PHYS-COLLISION-P1` evidence passes. An
offline P0 harness may supply exact synthetic excitation fixtures, but that
does not authorize a raw-backend runtime shortcut.

An illustrative private extraction record is:

```text
PhysicalAcousticExcitationV1 {
  source_contact_id,
  source_snapshot_hash,
  gameplay_tick,
  physics_tick,
  substep,
  participant_low,
  participant_high,
  feature_low,
  feature_high,
  point,
  normal_low_to_high,
  relative_velocity_low_to_high,
  impulse_lower_bound,
  impulse_upper_bound,
  effective_mass,
  canonical_material_tags[],
  phase: Begin | Persist | End,
  excitation_kind: Impact | Friction | Rolling,
  canonical_hash,
}
```

This is not a current public contract. A promoted version must define exact
quantization, bounds, canonical order, duplicate/collision behavior, end-event
semantics and the derivation from the complete committed contact batch.

`Impact` derives from a `Begin` record and bounded contact-energy/impulse
mapping. `Friction` and `Rolling` require a continuity interval, tangential
velocity, normal load/effective mass and stable surface calibration. A
`Persist` event alone is not proof of a usable friction force. If the first
projection cannot distinguish resting, sliding and rolling contact without
backend-private information, those exciters remain out of scope rather than
guessing from callback frequency.

Fracture, cloth, fluid, combustion and vocal excitation are not encoded as
fake contacts. A future owning subsystem must publish its own committed,
bounded, typed excitation fact before presentation may consume it.

## Modal runtime and exciters

For one damped mode, the continuous reference is:

```text
q'' + 2 * damping * angular_frequency * q'
   + angular_frequency^2 * q
   = participation_gain * excitation
```

The cooker supplies stable bounded coefficients. Runtime advances a bank of
damped resonators at the pinned audio sample rate and sums their radiation
gains. An impact injects a finite excitation at the contact location. A
scraping or rolling model feeds a separate bounded stochastic or pulse-train
exciter into the same resonator bank; noise generation uses an explicit seeded
profile, never ambient thread RNG.

The first reference implementation SHOULD use a scalar fixed-point or otherwise
integer-exact recurrence compatible with the existing 48 kHz canonical PCM
check. Optimized floating-point/SIMD implementations MAY be evaluated only
against that reference with a declared perceptual/numeric metric. Device PCM
is presentation data, but an optimized implementation cannot redefine the
pinned reference check or feed gameplay.

One promoted profile must declare:

- sample rate and channel layout;
- maximum modes per model and active modes per voice;
- maximum active physical voices and excitation records per tick/window;
- coefficient and accumulator ranges, saturation and stability rules;
- exact event-to-sample mapping and presentation epoch/reset behavior;
- canonical priority and tie-break order;
- allocation, queue, callback and whole-mixer budgets.

The real-time audio path preallocates its bounded state. It performs no file
I/O, dynamic allocation, blocking lock, unbounded queue operation or content
decode in the callback. Cooking, activation and model validation complete
before a voice is admitted.

## Radiation, propagation and spatialization

Object vibration and environmental sound propagation are different problems.
The first vertical uses a declared point-source, monopole/dipole or similarly
bounded radiation approximation. A later cooked acoustic-transfer model may
improve directivity from a geometrically complex vibrating body, but it remains
part of source radiation.

Room/portal response, occlusion, transmission and reflections remain the
SPEC-08 propagation layer. The engine-native attenuation/panning/zone path is
mandatory. Optional Steam Audio may consume generated PCM under its own
version, license, resource and fallback checks; its absence or failure cannot
disable source synthesis or change gameplay facts.

## Voice admission and presentation LOD

Physical audio work is admitted in canonical priority order using only
presentation facts declared by the profile. One candidate ladder is:

1. `FullModal` — complete admitted mode set and spatial excitation map;
2. `ReducedModal` — deterministic subset/grouping of the same model;
3. `HeuristicProcedural` — bounded material-class resonator/noise fallback;
4. `AuthoredSampleFallback` — exact declared clip;
5. `Muted` — diagnostic-only last resort.

Camera visibility, frame time, callback completion order and vendor voice
order cannot choose a tier when canonical displayless PCM is being checked.
Interactive presentation MAY use listener distance and declared presentation
priority for quality scaling because it is non-authoritative, but replay and
gameplay roots must remain unchanged for every tier choice.

## Determinism, replay and restart

Synthesizer phase, filter history, random-exciter state, propagation cache and
device buffers are reconstructible presentation state and are not saved. Load,
restart or presentation recovery begins a fresh presentation epoch, clears
physical voices and resumes only from newly committed inputs. A displayless
capture profile MAY declare bounded preroll from an existing replay, but the
preroll is evidence tooling rather than a new save owner.

The following roots MUST be identical with physical synthesis enabled,
disabled, voice-limited, missing its content, faulted or replaced by the
authored fallback:

- gameplay and RPG state;
- command ledger and committed events;
- physics snapshot/contact roots;
- deterministic `AcousticFactV1` gameplay-hearing facts;
- save/load/replay owner roots.

Canonical PCM evidence fixes exact content, excitations, sample rate, window,
numeric profile, source admission order and sink. Hardware output, optimized
float PCM or a third-party propagation adapter cannot claim the exact pinned
PCM result unless it proves the same declared comparison.

## Failure semantics

| Failure | Required behavior |
|---|---|
| Missing, corrupt or incompatible acoustic model | Reject that model before voice activation; use its exact declared authored/heuristic fallback or silence; do not mutate world state. |
| Missing physical binding | Emit one bounded diagnostic and use the existing clip path or silence; do not infer a binding from renderer material/name. |
| Stale contact or model revision | Reject the excitation/model pair; never apply it to a different body generation. |
| Excitation queue overflow | Apply declared canonical presentation priority/drop policy, count the loss and preserve gameplay/physics roots. |
| Nonfinite/unstable/overflowing resonator | Terminate the affected voice, record a stable diagnostic and select fallback; never retry with looser coefficients. |
| Late async content or propagation result | Use the already selected fallback/window; never insert audio retroactively into an elapsed canonical capture window. |
| Device or optional propagation failure | Continue engine-native mixing or discard device output while gameplay and acoustic facts continue. |
| PCM mismatch | Fail the physical-audio evidence only; do not reinterpret or repair authoritative simulation. |

## Bounded evaluation and promotion sequence

### P0 — offline reference

Cook or import a modal model for a tiny engine-owned primitive corpus. Drive it
with exact synthetic impulses, render canonical 48 kHz PCM and compare modal
frequencies, decay and bounded perceptual descriptors against reference
recordings or a high-quality offline solver. P0 changes no runtime contract.

P0 advances through five evidence checkpoints:

1. `AV-P0A` keeps hard signal, deterministic repeat and causal/metamorphic
   controls independent from subjective material identity.
2. `AV-P0B` hash-closes grouped real/generated/mutation corpora and measures
   frozen feature heads without acceptance authority.
3. `AV-P0C` adds temporal-spectral dynamics, leave-family/source/generator/
   mutation-out evaluation, explicitly labelled controlled negatives and
   calibration-only selective risk measured on parent-grouped holdout/shadow.
   Automatic `Pass` remains disabled until this checkpoint demonstrates its
   declared confidence bound and coverage.
4. `AV-P0D` freezes the neural feasibility benchmark: object-specific few-shot
   first, shared geometry-conditioned transfer second, with separate train,
   development, calibration, method-holdout and sealed admission-shadow roles.
   It must cook predictions exactly and cannot claim admission.
5. `AV-P0E` meets one frozen neural candidate/cooker with the independently
   frozen validator release and untouched admission shadow exactly once. It
   publishes `Pass`, `Reject` or `FallbackOutOfDomain` without retuning the same
   revision on the opened shadow.

Each research cycle changes one falsifiable source-model hypothesis, one model
architecture/cooker hypothesis or one validator release, not multiple sides of
the same comparison. A generator failure adds a reproducible mutation or
negative control before another similar pass. Two coherent failures without a
newly discriminating hypothesis trigger the repository research escalation
rule rather than another residual family or architecture sweep.

A domain is complete for P0 when one immutable revision has all hard and
metamorphic controls passing, measured selective risk/coverage on grouped
holdouts, untouched-shadow evidence, a bounded cost result and an exact
fallback. Research may then add a new domain or validator release without
reopening the completed domain silently. This is the stopping rule that turns
ongoing research into monotonically growing, reviewable coverage.

### P1 — first product vertical: rigid impact

Use three independently licensed/engine-owned object profiles representing
steel, wood and glass-like behavior and a small bounded geometry set. Exercise
hammer/drop impacts at multiple positions, energies and orientations through
the production contact/extraction path. The primary modal branch uses no
event-specific impact WAV; the declared sample/heuristic fallback remains
available for faults and A/B comparison.

P1 is intentionally impact-only. It must close the current contact-projection
implementation gap, content cooking/activation, canonical PCM and enabled/
disabled/fault root non-regression before rolling or scraping work begins.

### P2 — persistent contact

Add rolling and scraping only after P1. The corpus varies tangential speed,
normal load, surface roughness and contact continuity, and includes resting
and separating controls. If ordinary rigid contact facts cannot reproduce
stick-slip or micro-collision structure, evaluate one bounded flexible-contact
or micro-collision exciter rather than tuning arbitrary noise to the same
symptom.

### Later independent tracks

- fracture uses precomputed fragments/soundbanks or another bounded topology-
  aware source model after a committed fracture owner exists;
- footsteps combine a dedicated foot/shoe/surface contact model with authored
  or synthesized residuals;
- cloth, liquids and fire use their own motion-driven, particle/turbulence or
  combustion exciters;
- voice and bird syrinx models remain specialized biological instruments with
  separate controllability and quality criteria;
- differentiable or learned methods may fit the bounded acoustic field
  offline; the first product vertical consumes only its deterministic cooked
  record. Runtime neural inference requires a later measured product need,
  immutable local artifact, bounded inference/fault profile, complete
  non-neural fallback and a separate architecture decision.

## Candidate ProductChecks

These names reserve no current global gate. They become normative only with a
production consumer and promoting ADR.

| Check | Candidate observable result |
|---|---|
| `AUDIO-PHYS-SOURCE-P1` | Exact impact identity/point/energy drives the expected steel/wood/glass modal model and bounded PCM window; input/callback/worker permutations cannot change admitted excitation order. |
| `AUDIO-PHYS-CONTACT-P1` | Rolling/scraping fixtures distinguish resting, sliding, rolling and separation across declared speed/load/roughness controls without callback-frequency artifacts. |
| `AUDIO-PHYS-CONTENT-P1` | Acoustic profile/model/binding cook is repeatable and hash-closed; malformed, stale, oversized or incompatible content rejects atomically and selects the declared fallback. |
| `AUDIO-PHYS-PCM-P1` | Pinned 48 kHz displayless PCM and event-to-sample mapping are exact for the reference profile; enabled/disabled/faulted synthesis produces identical gameplay, ledger, physics, acoustic-fact and save/replay roots. |

The promoted implementation additionally maps to focused `fast`, `play` and
`content-package`; `platform` applies when host/device/propagation code changes,
and `performance` applies when the audio callback, mixer or cooker hot path is
materially affected. `persistence-replay` is required for proving the root
non-regression and fresh presentation epoch, not because synth state is saved.

No numeric quality or performance threshold is invented in this draft. P0
must measure the fixed corpus and choose an audible-error metric, maximum mode
count, voice count, queue capacity, memory budget and callback p95/p99 before
promotion. Complexity is approximately `O(active_modes * rendered_samples)`;
quality scaling must therefore reduce admitted modes/voices explicitly rather
than rely on an unbounded solver.

## Rejected alternatives

| Alternative | Reason |
|---|---|
| Replace all authored audio with one universal procedural layer | Different source classes need different models and authored fallback remains necessary for product quality and failure closure. |
| Feed raw PhysX callbacks directly to the mixer | Violates engine-owned contracts, stable identity, canonical ordering and backend isolation. |
| Put acoustic constants into `PhysicsMaterialDescriptorV2` | Creates parallel semantics in the physics authority and still lacks geometry, damping, radiation and calibration identity. |
| Runtime FEM/eigenmode solve per object | Unbounded and unnecessary; preprocessing can cook a compact resonator model. |
| Prompt-to-waveform model as the primary engine path | Produces an opaque asset rather than a controllable impact/listener field and does not by itself satisfy causality, exact cooking, OOD or deterministic fallback. |
| Runtime neural inference in the first vertical | External offline fitting can test the learned representation without adding model artifacts, inference budgets and a new runtime fault domain. |
| Make waveform propagation determine NPC hearing | Couples gameplay to voice admission, device state and optional presentation adapters. |
| Start with scraping, fracture, liquids or a bird synthesizer | Expands the unknown source/owner surface before the smallest rigid-impact consumer is proven. |
| Another manual residual family after the frozen DCT baseline | The remaining failures vary across object, impact and listener conditions; another global residual revision is not justified without a preregistered neural ablation identifying one missing statistic. |

## Research basis and limits

The candidate follows the established modal-sound pattern: offline geometry/
material analysis plus real-time excitation from rigid contact. The research
report records the primary sources and their limits. In particular, evidence
for impact/rolling, contact-rich scraping, fracture acceleration, parameter
fitting and source/propagation separation does not prove that one model covers
every material, object or biological sound. Product promotion depends on the
bounded Next Engine P0/P1 evidence above, not on a paper's demo or another
engine's marketing claim.
