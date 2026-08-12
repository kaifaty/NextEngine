# Anthropometric target для humanoid biomechanics profile

| Поле | Значение |
|---|---|
| Статус | Target selection retained; exact `TRAIN-1` table is frozen separately |
| Дата | 2026-08-12 |
| Planning ID | `humanoid-young-adult-male-gait-1700.v1` |
| Frozen profile | [Fixed humanoid biomechanics profile V1](2026-08-12-humanoid-biomechanics-profile-v1.md) |
| Frozen profile SHA-256 | `968ceb82ac2a40af872b496fdd43e32ba2ce3ffd90b2b3a33f38461c8269aabc` (revision 2) |
| Target | healthy young adult male gait model, intended stature `1.700 m` |
| Exact source-model mass | `75.337 kg` over `22` source bodies |
| Требования candidate | [Humanoid motor requirements baseline](2026-08-12-humanoid-motor-requirements.md) |
| План реализации | [Humanoid motor training rebuild](2026-08-12-humanoid-motor-training-rebuild.md) |
| Не является | medical model, population percentile, muscle model, готовой collider/actuator/PD specification или разрешением начать ML |

## 1. Решение

Для первого fixed-body candidate выбран gait-oriented anthropometric target на
основе full-body model Rajagopal et al. (2016): здоровый молодой взрослый
мужчина, описываемый последующей high-flexion работой как модель роста `170 cm`
и массы около `75 kg`.

Target выбран не как «средний человек вообще», а как один воспроизводимый
engineering subject:

- primary use case модели — walking/running and lower-extremity biomechanics;
- её joint centers, body frames, mass, CoM and inertia доступны как explicit
  source data, а не восстанавливаются по картинке;
- размер хорошо подходит первому fixed-body locomotion/recovery candidate;
- модель можно свести к существующим `23` controlled DoF без заявления о
  сохранении её muscle-tendon system;
- исходный target остаётся масштабируемым в будущих morphology profiles, но
  первая revision не обучается на диапазоне тел.

Это **male-specific single target**. Он не представляет женское тело,
возрастной диапазон, гражданское население, мировую популяцию или конкретного
человека и не должен называться 50th percentile.

## 2. Source identity и provenance

### 2.1. Primary gait source

| Fact | Source |
|---|---|
| Model and validation rationale | [Rajagopal et al., 2016, Full-Body Musculoskeletal Model for Muscle-Driven Simulation of Human Gait](https://doi.org/10.1109/TBME.2016.2586891) |
| Open full text | [PubMed Central PMC5507211](https://pmc.ncbi.nlm.nih.gov/articles/PMC5507211/) |
| Height/mass description and high-flexion limitations | [Lai, Arnold and Wakeling, 2017](https://pmc.ncbi.nlm.nih.gov/articles/PMC5989715/) |
| Engine-readable numeric authority | [`Rajagopal2015.osim` at the official repository's initial model commit](https://github.com/opensim-org/opensim-models/blob/a5c5f6ce9b904e618eeaf203e6efa48d457f9b41/Models/RajagopalModel/Rajagopal2015.osim) |
| Numeric-authority source commit | `a5c5f6ce9b904e618eeaf203e6efa48d457f9b41`, 2017-09-25, `add model` |
| Numeric-authority SHA-256 | `b8a31616557f73f798898c03a9beee723ba2987e646a688375d60c4327d90bff` |
| License authority | [SimTK Full Body Model](https://simtk.org/projects/full_body), MIT Use Agreement, packages `1738`/`1739` |
| License-text SHA-256 | `cb4e076bb74cecf35cf37f134c781e204745e8cd3c113ad1f7469b55ac7923b0` |
| Later 4.5 cross-check | [`Rajagopal2016.osim` at exact source commit](https://github.com/opensim-org/opensim-models/blob/e0cee6cbcf56f74731e841668ef668556d140ce9/Models/Rajagopal/Rajagopal2016.osim) |
| High-flexion cross-check | [`RajagopalLaiUhlrich2023.osim` at exact source commit](https://github.com/opensim-org/opensim-models/blob/e0cee6cbcf56f74731e841668ef668556d140ce9/Models/Rajagopal/RajagopalLaiUhlrich2023.osim) |
| Later cross-check commit | `e0cee6cbcf56f74731e841668ef668556d140ce9`, 2024-12-06, `Fix Inertial Inequality` |
| `Rajagopal2016.osim` SHA-256 | `3f5c5f23e486073f2ad2aa4a4967ffe2fcdd582b1e355512bc54f70c36376bf4` |
| `RajagopalLaiUhlrich2023.osim` SHA-256 | `8f30d0b64750b87eb7f705907862590535212b4afd7e919faa3fd7d1683d22ec` |

The exact initial official-repository `Rajagopal2015.osim` contains `22` bodies whose authored
masses sum to `75.337 kg`. Therefore `75 kg` is a human-readable source-model
description; `75.337 kg` is the exact mass-conservation target for the first
BodySchema derivation. Silent rounding back to `75.000 kg` is forbidden.

Source `.osim`, meshes and muscle data remain external and are not committed to
NextEngine. The SimTK package page records an MIT Use Agreement that explicitly
permits use, modification and distribution subject to retaining its notice.
The frozen profile records the notice/hash and uses the official repository's
initial model commit as its exact byte identity; later OpenSim modifications
are cross-checks rather than silent numeric replacements.

### 2.2. Independent cross-checks

- [de Leva (1996)](https://doi.org/10.1016/0021-9290(95)00178-6) provides
  joint-center-adjusted relative segment masses, CoM positions and radii of
  gyration for young adult male/female samples. It is a cross-check for segment
  mapping, not authority to overwrite exact source bytes silently.
- [NASA-STD-3001 Volume 2 Rev F](https://standards.nasa.gov/standard/NASA/NASA-STD-3001_VOL_2),
  Appendix E.3, provides current public unsuited range-of-motion reference data
  and explains its population/measurement context. It is an anatomical review
  source, not a reason to copy every table bound into a serial hinge.
- [ANSUR II](https://www.army.mil/article-amp/188601)
  is used only as a scale sanity check. Its `4 082` male and `1 986` female
  subjects are U.S. Army personnel; that sample does not turn this target into
  a civilian or world-population percentile.

When sources disagree, the gate report records the difference and the selected
rule. It never averages incompatible measurements merely to obtain one number.

## 3. Exact target interpretation

| Property | Selected interpretation |
|---|---|
| Biological class | healthy young adult male reference |
| Intended standing stature | `1.700 m` in unsuited neutral anatomical pose |
| Exact BodySchema total mass | `75.337 kg` after source-to-target mapping |
| Equipment/payload | none |
| Gravity | `9.80665 m/s²` evaluation reference |
| Symmetry | exact authored left/right symmetry except source-declared differences |
| Controlled topology | one free pelvis root plus existing `23` controlled rotational DoF |
| Muscle model | not imported; direct torque and muscle activation remain out of scope |
| Runtime controller | bounded residual joint targets plus engine-owned fixed PD/safety |

Standing root height is **not** `1.700 m` and is not copied from frozen Stage 0
V1. It is derived after exact segment/joint/collider mapping as the pelvis-root
height that places both sole support planes on ground in neutral pose without
penetration. The resulting value receives a new BodySchema identity/revision.

## 4. Mapping to the `23` controlled DoF

The selected controlled semantics stay aligned with the current bounded
humanoid lane:

| Group | Target DoF | Source correspondence |
|---|---:|---|
| Torso | `3` | lumbar rotation, extension and bending |
| Hips | `6` | bilateral flexion, adduction/abduction and internal/external rotation |
| Knees | `2` | bilateral flexion/extension |
| Ankles | `4` | bilateral ankle flexion plus subtalar inversion/eversion |
| Shoulders | `6` | bilateral flexion, adduction/abduction and rotation |
| Elbows | `2` | bilateral flexion/extension |
| Total | `23` | pelvis free root is not an actuator channel |

Source MTP/toe, forearm pronation/supination and wrist coordinates are not
controlled in this candidate. Their rigid neutral projection and mass/inertia
are merged into the nearest retained physical segment. They cannot appear as
unmodelled animated mass or hidden runtime DoF.

### 4.1. Physical segments versus serial-axis carriers

The source contains `22` anatomical rigid bodies; the current NextEngine
23-DoF representation contains pelvis plus one child body per revolute joint.
These are not equivalent notions. The mapping must distinguish a physical
mass-bearing segment from a virtual carrier used only to express a second or
third axis:

| Target chain | Required source-mass mapping |
|---|---|
| pelvis | source pelvis |
| torso yaw/pitch/roll | two virtual axis carriers; source torso mass on the declared physical torso link |
| each hip yaw/roll/pitch | two virtual axis carriers; source femur on the declared physical thigh link |
| each knee | source tibia plus merged patella contribution |
| each ankle pitch/roll | one virtual axis carrier; merged talus/calcaneus/toes on the physical foot link |
| each shoulder pitch/roll/yaw | two virtual axis carriers; source humerus on the physical upper-arm link |
| each elbow | merged source ulna/radius/hand on the physical forearm-hand link |

For every merge, in one declared target frame:

```text
M = sum(m_i)
CoM = sum(m_i * r_i) / M
I = sum(R_i * I_i * R_i^T
        + m_i * ((d_i · d_i) * Identity - d_i * d_i^T))
```

The compiler uses checked deterministic arithmetic and records source and
target frames. Total mapped mass must equal `75.337 kg` exactly in the chosen
canonical representation; whole-body CoM and inertia must agree with the
independently transformed source within predeclared quantization tolerances.

Current V1's practice of assigning arbitrary kilograms and sphere inertia to
each serial axis is not a valid mapping. If PhysX cannot represent a zero-source-
mass carrier, `TRAIN-1/2` must define an explicit solver mass/inertia projection
and conservation rule. An unexplained epsilon or simulator default blocks the
gate; a new public carrier semantic requires the ADR/SPEC step already called
out by the implementation plan.

### 4.2. Head and hands

The source torso includes head/neck mass, and the selected `23` DoF contain no
neck channel. Nevertheless the target must have declared head collision
geometry and contact classification. `TRAIN-1` chooses one of two explicit
representations:

1. head collider fixed to the physical torso while source torso mass/CoM/inertia
   remain intact; or
2. a separate fixed head body produced by a sourced inertia split whose merge
   exactly reconstructs the original torso properties.

Likewise merged hands remain represented by hand/forearm collision geometry
and recovery contact features even though wrist/finger DoF are absent. Visual
geometry cannot be collisionless hidden mass.

## 5. ROM policy

Neither source model's XML `range` values nor NASA population ranges are copied
blindly into BodySchema:

- Rajagopal 2016 is optimized for gait and has known limitations at high knee
  flexion;
- the Lai/Uhlrich revision broadens knee/ankle/subtalar motion for high-flexion
  tasks, but some generic upper-body/lumbar XML ranges are intentionally broad;
- NASA ROM tables describe measured functional ranges in their own axes and
  population context, not independent safe bounds for arbitrary serial hinges;
- get-up clips provide an additional task-required envelope but cannot expand
  anatomy after a run starts.

`TRAIN-1` therefore authors per DoF hard ROM from aligned anatomical axes and
these sources, then defines an inner soft ROM used by reference retargeting and
policy targets. The review must prove that every required prone/supine/side
transition fits inside soft ROM while knee/elbow hyperextension and unrealistic
spine/ankle twist remain impossible. Exact values and their citations are still
a blocking output; this target-selection document does not invent them.

## 6. Actuator, PD and collider boundary

The selected source's muscle-tendon forces, generic torque actuators and visual
meshes are **not** NextEngine actuator or collider defaults.

- per-joint effort/velocity/rate/power/energy limits need an explicit source or
  body-weight-normalized engineering derivation;
- PD gains and residual-action scales are tuned by passive/step/hold tests under
  the fixed target body and remain engine-owned hard facts;
- colliders approximate the authored external body and support geometry, not
  source rendering meshes or one sphere per DoF;
- foot geometry must expose heel, forefoot/toe and sole support features needed
  by contact/slip gates;
- every difference from source inertial geometry is recorded and revalidated.

The profile makes no claim to muscle activation, metabolic fidelity or human
injury thresholds.

## 7. Frozen-profile disposition

The linked frozen profile closes the license, mapping, carrier, body, collider,
joint, actuator and neutral-support specification items. It records analytical
mass/anatomy reviews as Pass and correctly leaves passive PhysX behavior and
recovery-clip fit `NotRun` for TRAIN-2 and TRAIN-4 respectively.

`TRAIN-1` may advance only with a gate report bound to the exact frozen-profile
hash. This target-selection document alone remains non-evidence.
