# Physical sound steel automated search — 2026-08-27

Status: `MEASURED / FALLBACK_OUT_OF_DOMAIN / NO_DEMO_OR_RUNTIME_PROMOTION`

## Question and boundary

Can the current experimental steel voice be tuned automatically against the
frozen AV-P0B real-material gallery without tuning on holdout/shadow, changing
the accepted wood/Q30 controls, or asking for per-sound human approval?

This is external P0 evidence under Proposed SPEC-45. It changes no public
contract, cooked content, gameplay authority, default demo profile or authored
clip fallback. Reports, WAVs, model weights and Python environments remain in
the external experiment store.

## Research escalation

The first 55-profile search changed global frequency, damping, high-mode tilt
and strike-noise shape while retaining the same sparse 12-mode spectrum. Frozen
BEATs classified `0/165` position renders as metal. Continuing those axes was
therefore stopped.

The primary material-perception study reports that damping alone cannot turn a
glass/wood spectrum into metal: glass is usually spectrally sparse while metal
is rich, broadband and dissonant, with roughness the next important descriptor
after normalized decay. It describes amplitude/frequency-modulation sidebands
inside critical bands as a direct roughness control and notes maximum sensory
dissonance near 25% of a critical bandwidth:

- [Aramaki et al., Controlling the Perceived Material in an Impact Sound Synthesizer](https://doi.org/10.1109/TASL.2010.2047755)
- [author-hosted material-space examples](https://kronland.fr/publications/controlling-the-perceived-material-in-an-impact-sound-synthesizer/)

Competing hypotheses and observations:

| Hypothesis | Evidence | Conclusion |
| --- | --- | --- |
| H1: sparse modal structure is the main glass/metal error | `0/165` sparse-grid BEATs metal classifications; roughness sidebands produce the first metal classifications | Supported; roughness/density is a required search dimension |
| H2: onset/noise shape determines material | sparse search varied three transient profiles without a pass; source study reports onset is not a relevant material-control parameter | Rejected as the primary cause |
| H3: decay/duration is sufficient | longer decay improves BEATs, but all sparse candidates remain glass and the independent head still rejects the rough candidate | Necessary but insufficient |
| H4: one successful representation is enough | BEATs accepts the refined candidate while PANNs maps every refined render away from metal | Rejected; evaluator disagreement must abstain |

## Implemented experiment path

`xtask physical-sound-steel-search` now emits an external-only deterministic
batch and report. It supports three frozen profile sets:

- `sparse-grid-v1`: 55 profiles × center/edge/corner;
- `roughness-grid-v2`: 37 profiles adding symmetric critical-band sidebands;
- `roughness-refinement-v3`: 24 profiles in the bounded neighborhood selected
  from v2, with one fixed strike seed per position across every profile.

The current 12-mode control renders byte-exactly through the search path. The
36-mode roughness recurrence is heap-backed only by the offline search voice;
ordinary lab/demo voices retain fixed 12-state storage. Generated PCM rejects
silence/clipping and repeated v3 generation is byte-identical for the report and
all 72 WAVs.

The corpus benchmark now reports an expected-label nearest distance, nearest
competing-label distance and signed expected-label margin. Positive margin
means the nearest intended material anchor is closer than every competing
material; same-label neighbors cannot inflate it.

## Results

All search targets are calibration-partition generated objects. The gallery is
real development only. Holdout and shadow never enter the objective.

| Search | Head | Metal renders | Best profile result |
| --- | --- | ---: | --- |
| sparse v1, 165 renders | classical | `5/165` | diagnostic only |
| sparse v1, 165 renders | BEATs | `0/165` | best worst margin `-0.07545` |
| roughness v2, 111 renders | classical | `0/111` | diagnostic only |
| roughness v2, 111 renders | BEATs | `7/111` | best profile `2/3`, worst margin `-0.00709` |
| refinement v3, 72 renders | classical | `0/72` | diagnostic only |
| refinement v3, 72 renders | BEATs | `25/72` | three profiles `3/3` |
| refinement v3, 72 renders | PANNs CNN14 | `0/72` | all refined candidates rejected |

The BEATs winner by worst-position margin is
`steel-refine-f0650-d4000-r0350-i0900`:

| Position | BEATs prediction | expected-label margin |
| --- | --- | ---: |
| center | metal | `+0.0363201` |
| corner | metal | `+0.0153442` |
| edge | metal | `+0.0150618` |

BEATs is `6/6` on development leave-object-out real identity and was already
`15/15` across the frozen real material split. [PANNs CNN14](https://arxiv.org/abs/1912.10211)
is independent AudioSet-pretrained CNN evidence: it recognizes all `5/5` real metal objects
across development/calibration/holdout/shadow, but maps the BEATs winner to
wood in all positions with margins `-0.22744`, `-0.21443`, and `-0.21903`.
The PANNs window is fixed at two seconds for every target, so container duration
cannot be its decision rule.

The frozen existing shadow controls remain unchanged under BEATs: wood `3/3`,
old steel `0/3`, glass `5/7`, including selected Q30 glass as glass. Exact PCM
tests retain the accepted wood and Q30 hashes.

## Decision

The automated ensemble result is `FallbackOutOfDomain`:

- no refined profile is admitted to the demo or runtime;
- the current steel profile and authored-clip fallback remain unchanged;
- the positive BEATs result is retained as evidence that roughness is causal,
  not as acceptance authority;
- classical/PANNs disagreement is not averaged away or overridden by the
  search objective.

The smallest next action is to broaden the real-metal/object-family corpus and
measure which spectral/noise statistics separate the PANNs-real-metal cluster
from the BEATs-only candidate. Only then should a frozen v4 counterfactual add
modal density/nonlinear or measured stochastic residual structure. It must be
selected without shadow data and must pass both independently validated heads
before any demo-profile proposal.

That bounded next action is now complete in the
[steel residual v4 report](physical-sound-steel-residual-v4-2026-08-27.md).
The residual counterfactual also falls back; do not start another gain/T20 grid.

## Frozen external evidence

Experiment root:
`/home/kaifaty/.codex/experiments/nextengine/physical-sound/av-p0b-kronland-material-v1`

| Artifact | SHA-256 |
| --- | --- |
| sparse v1 generation report | `3bbac90679fbb9fc1464ac6fe21eb87c7d819ce4702b958fe3be2b353c271996` |
| sparse v1 BEATs benchmark report | `1f94d3a61889cd09148feb49d2af3231832f8ae2316c36e17165bac683980878` |
| roughness v2 generation report | `d024f170adb0a0d178e27fbc6bedd4312ffbcca1c8b8c34f0bea017e14d4f667` |
| roughness v2 BEATs benchmark report | `39cbe2c179d0fa613d74bc0550227bbb08ecccdaa699680711a6db1a610ad7ce` |
| refinement v3 generation report | `c689d7b2fce58d6c41c160ec5d5cfcfed2d93724e7a8bcbe8bb7cd4260db6f0d` |
| refinement v3 ensemble manifest | `31d329ef1737a98de5be01546f6b0c6730d8969930e364e7820ff21869b341a7` |
| refinement v3 BEATs matrix | `11c461ca6b09be6cc96959aa7526fae8cae7fa44fd17e21b026e7276a147e3cb` |
| refinement v3 PANNs matrix | `624b8880d68b0a30f25297ffb772aec8edae58a8b72993887c93b47c73f6aa57` |
| refinement v3 ensemble report | `d4c9f24410d21b3f2aa71b99a45a94a540788e81d4a73b77e1ca345ea9075830` |
| PANNs `Cnn14_mAP=0.431` checkpoint | `0dc499e40e9761ef5ea061ffc77697697f277f6a960894903df3ada000e34b31` |

External extractor revisions are frozen by file SHA-256 in the experiment
store. Model/tool versions are PyTorch `2.8.0+cpu`, BEATs official checkpoint
`d43cbfad4d7b56381c061d7a24774f908d4d94c72961f6eb1d9090ff18cd8d34`,
and `panns-inference 0.1.1` with 2048-dimensional L2-normalized CNN14
embeddings. Both matrices use cosine distance in the hash-closed Rust
benchmark.
