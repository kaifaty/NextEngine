# Physical Sound V29 Q1a-M — internet source-growth research

| Field | Value |
| --- | --- |
| Date | `2026-09-02` |
| Status | `COMPLETE / PRIMARY_SOURCE_RESEARCH / METADATA_ONLY / STRICT_STEEL / BALANCED_POWER_NOT_YET_PROVEN` |
| Roadmap | [V29 Q1a-M](../plans/physical-sound-synthesis-roadmap-v29.md) |
| Predecessor | [Q1-M result](physical-sound-v29-q1m-metal-role-power-result-2026-09-02.md) |
| Product effect | None; no payload, role, model, validator, cooked clip or runtime path is authorized. |

## Question

Can published internet data close Q1-M's seven-project and nine-Steel deficits
without asking the user to record impacts, weakening exact `Steel`, reusing the
historical Glass split or opening any audio before roles exist?

The answer is narrower than the raw totals suggest. A bounded Freesound
increment can plausibly close the raw project, exact-Steel and aggregate-reject
floors from current publisher metadata. It cannot yet prove that the projects
can be divided into the two protected evaluations while leaving five whole
projects for the unprotected roles. Q1a must therefore solve the partition,
not merely add counters.

## Primary-source findings

### Scientific corpora

- [YCB Impact](https://www.iri.upc.edu/publications/show/2619) publishes more
  than 3,000 impacts over 75 YCB objects and explicitly distinguishes Steel,
  Aluminium and non-Metal materials. It is already one of Q0-M's two projects,
  so it adds no new project power.
- [RealImpact](https://openaccess.thecvf.com/content/CVPR2023/papers/Clarke_RealImpact_A_Dataset_of_Impact_Sound_Fields_for_Real_Objects_CVPR_2023_paper.pdf)
  supplies 150,000 force-synchronized recordings, but its 50 objects were
  selected from ObjectFolder. It is valuable generator/transfer evidence, not
  a clean new physical-object population for protected validator roles.
- Giordano and McAdams' [real plate study](https://www.mcgill.ca/mpcl/files/mpcl/blg_smc_2006_jasa.pdf)
  records five sizes each of Steel, Glass, Wood and Plexiglass plates. The
  paper is strong object/material evidence, but the current official page does
  not expose a hashable recording archive. It remains an acquisition lead, not
  current source power.
- McAdams et al. publish WAV supplements for their
  [simulated impacted-plate study](https://www.mcgill.ca/mpcl/files/mpcl/smc_2010_jasa.pdf).
  Those signals can support P0 known truth but cannot count as real validator
  positives.
- Generic event/video corpora and Greatest Hits do not bind a stable physical
  object to an exact publisher material. They remain background/OOD evidence.

### Freesound as a bounded source registry

Freesound's official pages expose uploader, pack, sound ID, title,
description, pack membership and a per-sound Creative Commons link. The
official [API documentation](https://freesound.org/docs/api/) confirms that
packs and sounds are first-class resources, while
[API authentication](https://freesound.org/docs/api/authentication.html)
requires a credential. Q1a therefore uses bounded public HTML pages only; it
does not guess an API token, download a preview or follow an audio URL.

The following project candidates are independent uploader/pack revisions and
were absent from the Q1-M baseline repository tree. Counts below are proposed
metadata identities that still require the capture/audit owner to verify the
live pages.

| Publisher / pack | Strict Steel groups | Non-Metal groups | Important evidence |
| --- | ---: | ---: | --- |
| [Benboncan / 4023](https://freesound.org/people/Benboncan/packs/4023/) | 1 | 0 | Fabricated `steel grille`; the stainless bowl is deliberately not exact Steel. |
| [ldezem / 21681](https://freesound.org/people/ldezem/packs/21681/) | 3 | 0 | Separately identified 3, 6 and 16 mm steel plates, each struck on the same named anvil. |
| [Bibow / 24227](https://freesound.org/people/Bibow/packs/24227/) | 1 | 0 | Hollow steel pipe; a sibling whose title says Steel but description says Aluminium is a mandatory conflict control. |
| [itinerantmonk108 / 31000](https://freesound.org/people/itinerantmonk108/packs/31000/) | 1 | 1 | Steel bowl impact and rubber-spatula reject. |
| [juskiddink / 5069](https://freesound.org/people/juskiddink/packs/5069/) | 2 | 0 | A steel metallophone bar and a dimensioned steel rod. |
| [AnthonyOstrander / 37664](https://freesound.org/people/AnthonyOstrander/packs/37664/) | 1 | 0 | Publisher title identifies a steel-can drum; action identity must remain explicit. |
| [FullMetalJedi / 19202](https://freesound.org/people/FullMetalJedi/packs/19202/) | 0 | 1 | Glass coffee can hit; stainless-steel thermos is not exact Steel. |
| [Puniho / 14173](https://freesound.org/people/Puniho/packs/14173/) | 0 | 1 | Tapped glass bottle with declared water contamination; reject-only. |

The candidate increment is eight projects, nine strict-Steel groups and three
non-Metal groups. Together with Q0-M it would produce ten projects, `32`
strict-Steel groups and `73` rejects. These are only raw metadata counts.

## Exact Steel means exact Steel

`Stainless Steel`, `Iron`, `Aluminium`, `Carbon Steel`, `zinc-plated steel`, a
generic `Metal` tag and an inferred tool composition do not satisfy the frozen
positive label. The owner accepts an exact positive only when publisher text
binds an unqualified whole-word `steel` to the identified sounding object.

This rule removes several apparently attractive results:

- three stainless-steel gongs are other-Metal diagnostics, not Steel credit;
- stainless bowls and a stainless thermos are not silently pooled;
- the Bibow title/description conflict resolves to exclusion, not the title;
- pack tags alone cannot upgrade an object whose description says only Metal.

Changing this policy would be a fresh broad-Metal release, not a Q1a repair.

## Why aggregate `32/73` can still fail

ObjectFolder owns `17` Steel and `63` rejects; YCB owns `6` Steel and `7`
rejects. A protected role needs `16/35`, at least two projects and whole-project
assignment. Five additional roles each need one untouched project.

With ten total projects, the two protected roles may consume at most five if
five remain for development, calibration and generator work. No permitted
split can give the YCB-side protected role enough Steel and rejects. The audit
must enumerate the exact project partition and publish the best reachable
frontier plus its deficits. It must not infer feasibility from aggregate sums.

A useful next-source target is therefore dual-class, not another isolated
Steel clip: one new independent project with roughly five exact-Steel objects
and at least 28 non-Metal objects would be far more valuable than many repeats
of one bowl. Giordano's plate study has the right five-Steel shape but only 15
non-Metal plates and no current recording archive, so it cannot close the gap
alone.

## Decision

Implement one two-phase Q1a metadata owner:

1. capture only the explicit Freesound pack/sound HTML pages into an external
   hash-closed directory;
2. normalize and audit the frozen capture twice without network access;
3. preserve exact Steel, compatible per-sound license metadata, current
   baseline exposure and every ignored/conflicting identity;
4. combine candidate counts with Q0-M and solve whole-project protected-role
   feasibility while reserving five unprotected projects;
5. return `RAW_GROWTH_VERIFIED / BALANCED_POWER_OOD` unless an actual complete
   partition exists.

No audio, preview, waveform, model feature or header is read. Failure or
insufficient power is a successful automatic OOD result and keeps Q2/P2
blocked.

