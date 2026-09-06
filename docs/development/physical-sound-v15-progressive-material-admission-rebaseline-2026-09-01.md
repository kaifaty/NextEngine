# Physical sound V15 — progressive material admission rebaseline

| Field | Value |
| --- | --- |
| Date | `2026-09-01` |
| Status | `BOUNDED_RESEARCH_COMPLETE / METAL_FIRST_RECOMMENDED` |
| Supersedes | V14 assumption that Glass, Wood and Metal must enter the first protected run together |
| Signal access | `0` new PCM samples, `0` force samples, `0` protected samples |
| Product effect | None; SPEC-45 remains `Proposed` and authored clips remain authoritative |

External research root:

```text
/home/kaifaty/.codex/experiments/nextengine/physical-sound/physical-sound-v14-n1c-source-research.xzOKNX
```

| Artifact | Bytes | SHA-256 |
| --- | ---: | --- |
| RealImpact paper PDF | `12,606,133` | `e71398400fca52dac3f9b7cca0de61428c9c2e42fd434f0d5abc89e5ce0295a8` |
| YCB-impact publisher wiki | `8,732` | `a7e1dbd6149facd6dc8d70621128691727efeea0dd6fb5801f10ad8c8564f4b8` |
| YCB object/material workbook | `7,994` | `27672ecfdfaf2a1ecfc8926127ab9ab962e9cb59adb593b141caadc0459ae0fd` |

The workbook was inspected structurally and contains 77 object rows. No audio
archive body, PCM value, force value or protected signal was opened.

## Question

Can published internet data fill the unchanged Dataset Contract V1 role shape
for Glass, Wood and Metal at the same time, or must the first learned physical-
sound vertical admit materials progressively?

The role shape remains `4 train + 1 generator development + 1 validator
calibration + 1 method holdout + 1 admission shadow` for every admitted
material. This research may narrow the first domain; it may not lower the
minimum, reuse an exposed object as fresh evidence or inspect signal to choose
roles.

## Evidence

### Existing repeat-exact inventory

[N1b](physical-sound-v14-n1b-real-source-metadata-inventory-result-2026-09-01.md)
found only `1/7/18` unexposed real Glass/Wood/Metal metadata candidates under
the conservative exact-name and current-revision rules. It also found that
ObjectFolder object IDs `71…100` changed meaning between the pinned historical
table and the current publisher table. Numeric ID alone is therefore not a
stable cross-revision identity.

RealImpact's [paper](https://arxiv.org/abs/2306.09944) states that its 50
scanned objects originate from ObjectFolder and include material, impact,
listener and force metadata. Its published object names align with the
historical ObjectFolder revision, not the current `71…100` table. This can
recover source capability, but only after an exact revision-aware exposure
census: for example, `93_GreenGoblet` has already been used by earlier work and
must not be relabelled fresh because the current table assigns a different
object to numeric ID `93`.

### YCB-impact sounds

The public [YCB-impact paper record](https://www.iri.upc.edu/publications/show/2619)
and [OSF project](https://osf.io/4tcp6/) add a useful independent source. The
publisher describes manual impact, scratch and drop recordings for 75 YCB
objects and robot impact data for 49 objects, with object and material labels.
The official object/material workbook contains only three objects whose
primary material is Glass and three whose primary material is Wood. The source
therefore improves geometry-backed real diversity but cannot by itself close
the eight-group Glass role shape.

YCB-impact remains a candidate, not admitted evidence, until an adapter proves
stable object/recording parentage, exact YCB mesh revision, contact semantics,
support/listener axes and prior exposure without opening audio.

### AV-MSF

[AV-MSF](https://arxiv.org/abs/2608.05145) provides the strongest current
evidence for the intended representation. It models object-global modal
frequency and damping, position-dependent modal gains and a separately bounded
residual, and reports few-shot evaluation on ObjectFolder Real and RealImpact.
Its project reports roughly six input impacts per object in the 20% setting.

The paper also reports that nearest-neighbour interpolation is a strong
baseline on symmetric objects, that missing representative local geometry
causes failures, and that a generic data-driven baseline can hallucinate or
distort spectral structure. These observations require explicit KNN/geodesic
controls, geometry-coverage OOD and a structured modal candidate.

The [official repository](https://github.com/ZisenShao/AV-MSF) currently says
`Code coming soon`. Neither unpublished code nor unpublished processed data
can be a V15 prerequisite. The representation is a falsifiable hypothesis to
reimplement, not a dependency or proof of Next Engine quality.

### Audio-visual and audio-only corpora

The public [joint object/material audio-visual dataset paper](https://arxiv.org/abs/1601.02220)
and action-video corpora can strengthen material and artifact validators, but
they do not establish the stable 3D geometry/contact/support lineage required
for generator holdout or shadow credit. They remain T4 evidence.

## Conclusion

The full simultaneous Glass/Wood/Metal entry gate is not presently supported
by bounded published metadata. Continued undirected source search would delay
the representation experiment without changing that fact.

Metal has enough currently unexposed candidates to attempt the unchanged role
shape. Wood is near the gate and may become eligible after the YCB adapter and
revision-aware exposure census. Glass remains source-growth and fallback-only:
it can participate in known-truth synthetic tests and report-only acoustic
analysis, but not claim protected real admission yet.

## Decision

1. Rebaseline to progressive material admission.
2. Keep the complete `4/1/1/1/1` role invariant for every material that enters.
3. Attempt `Metal` first; add `Wood` and then `Glass` only through the identical
   frozen pipeline and fresh protected groups.
4. Complete a revision-aware identity/exposure resolver and a signal-blind YCB
   capability adapter before freezing any real role.
5. Implement a small mesh-conditioned modal field independently, with KNN,
   geodesic RBF, local-linear and geometry-agnostic controls.
6. Keep automatic validation independent of generator training. Human audition
   may audit a release but is not a per-object gate.
7. Cook only deterministic clips. Runtime neural inference and arbitrary-force
   transfer remain outside the admitted claim.

## Reconsideration conditions

- A published, hash-closable source supplies enough fresh Glass or Wood object
  groups with the required axes.
- The revision-aware census proves more eligible groups than the conservative
  N1b inventory without identity ambiguity or prior exposure.
- Known-truth or real opened-development evidence rejects the global-poles plus
  spatial-gains factorization; in that case V15 stops before protected signal.
