# Fixed humanoid biomechanics profile V1

| Поле | Значение |
|---|---|
| Статус | Frozen `TRAIN-1` implementation input; not a trained or published candidate |
| Дата | 2026-08-12 |
| Profile ID | `nextengine.body.humanoid-biomechanics-raja-1700.v2` |
| Schema | `BodySchemaV2`, revision `1`, ADR-069 |
| Coordinate profile | `nextengine.coordinate.y-up-x-right-z-forward.v1` |
| Source target | Rajagopal et al. healthy young-adult gait model |
| Standing collider height | exactly `1.700000 m` in the authored neutral pose |
| Total mass | exactly `75.337000 kg` |
| Topology | one free pelvis root, `24` solver bodies, `23` controlled revolute DoF |
| Training authorization | none before `TRAIN-4`; this document closes only the `TRAIN-1` table |

This file is the reviewed numeric input for `TRAIN-2`. Units are fixed-point SI:
translations and collider dimensions are micrometres (`um`), angles are
microradians (`urad`), mass is microkilograms (`ukg`) and inertia components
are micro-`kg*m^2`. `I6` is ordered `(xx, xy, xz, yy, yz, zz)`. An omitted
rotation is the identity quaternion. Array order in this document is
explanatory; canonical V2 records sort by stable ID.

## 1. Source, license and transformation

### 1.1. Exact source closure

| Fact | Exact value |
|---|---|
| License authority | [SimTK Full Body Model project](https://simtk.org/projects/full_body), packages `1738` and `1739` |
| Use agreement | MIT Use Agreement, copyright `2015 Stanford University` |
| License-text SHA-256 | `cb4e076bb74cecf35cf37f134c781e204745e8cd3c113ad1f7469b55ac7923b0` over LF text with paragraphs unwrapped and one final LF |
| Exact model byte mirror | [`Rajagopal2015.osim`](https://github.com/opensim-org/opensim-models/blob/a5c5f6ce9b904e618eeaf203e6efa48d457f9b41/Models/RajagopalModel/Rajagopal2015.osim) in the official OpenSim model repository |
| Source commit | `a5c5f6ce9b904e618eeaf203e6efa48d457f9b41`, 2017-09-25, initial `add model` commit |
| Source file SHA-256 | `b8a31616557f73f798898c03a9beee723ba2987e646a688375d60c4327d90bff` |
| Source bodies | `22` excluding ground |
| Sum of authored source masses | `75.337000 kg` |
| Scientific identity | Rajagopal et al. (2016), DOI `10.1109/TBME.2016.2586891` |
| High-flexion ROM cross-check only | `RajagopalLaiUhlrich2023.osim`, SHA-256 `8f30d0b64750b87eb7f705907862590535212b4afd7e919faa3fd7d1683d22ec` |

The SimTK project page exposes the MIT grant for both downloadable Full Body
Model packages. It explicitly permits use, modification and distribution,
subject to retaining its notice. The source `.osim`, meshes and package data
remain external; only the derived fixed-point table and the notice are kept in
NextEngine. The later OpenSim 4.5 file is not the numeric authority: it is used
only to cross-check high-flexion ROM and inertial-inequality corrections.

The source-to-engine vector mapping is exact:

```text
engine(x_right, y_up, z_forward) = source(z, y, x)
```

Joint axes below are conservative anatomical principal axes in the target
frames. The source knee coupling and oblique ankle frames are not claimed to
be reproduced by a serial rigid-hinge model; their reduction is explicit in
the joint review rather than hidden in a backend default.

### 1.2. Retained MIT notice

```text
Copyright (c) 2015, Stanford University

Permission is hereby granted, free of charge, to any person obtaining a copy
of this software and associated documentation files (the "Software"), to deal
in the Software without restriction, including without limitation the rights
to use, copy, modify, merge, publish, distribute, sublicense, and/or sell
copies of the Software, and to permit persons to whom the Software is furnished
to do so, subject to the following conditions:

The above copyright notice and this permission notice shall be included in all
copies or substantial portions of the Software.

THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR
IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY,
FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE
AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER
LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM,
OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN THE
SOFTWARE.
```

## 2. Source-to-target body mapping

Ten serial-axis carriers are a declared solver projection. Each carrier has
mass `1,000 ukg`, isotropic inertia `[1, 1, 1]`, and the source segment CoM.
Two carriers are subtracted from each torso, femur and humerus physical row.
Carrier collision participation is `NonCollidingCarrier`; no invisible sphere,
support feature or ground response is permitted.

| Mapping group | Target rows | Source bodies | Mass rule (`ukg`) |
|---|---|---|---:|
| `map.pelvis` | `body.pelvis` | pelvis | `11,777,000` |
| `map.torso` | torso pitch/roll carriers + torso yaw physical | torso | `1,000 + 1,000 + 26,824,600 = 26,826,600` |
| `map.right-thigh` | right hip pitch/roll carriers + hip yaw physical | femur_r | `1,000 + 1,000 + 9,299,400 = 9,301,400` |
| `map.left-thigh` | left hip pitch/roll carriers + hip yaw physical | femur_l | `1,000 + 1,000 + 9,299,400 = 9,301,400` |
| `map.right-knee` | right knee | tibia_r + patella_r | `3,707,500 + 86,200 = 3,793,700` |
| `map.left-knee` | left knee | tibia_l + patella_l | `3,707,500 + 86,200 = 3,793,700` |
| `map.right-ankle` | ankle pitch + ankle roll | talus_r + calcn_r + toes_r | `100,000 + 1,466,600 = 1,566,600` |
| `map.left-ankle` | ankle pitch + ankle roll | talus_l + calcn_l + toes_l | `100,000 + 1,466,600 = 1,566,600` |
| `map.right-arm` | shoulder pitch/roll carriers + shoulder yaw physical + elbow | humerus_r + ulna_r + radius_r + hand_r | `1,000 + 1,000 + 2,030,500 + 1,672,500 = 3,705,000` |
| `map.left-arm` | shoulder pitch/roll carriers + shoulder yaw physical + elbow | humerus_l + ulna_l + radius_l + hand_l | `1,000 + 1,000 + 2,030,500 + 1,672,500 = 3,705,000` |
| **Total** | `24` target rows | `22` source bodies | **`75,337,000`** |

Rigid merges use the parallel-axis theorem in the declared target frame. The
authoritative tensor keeps product terms. The first PhysX projection uses its
axis-aligned diagonal and declares the maximum dropped component per row;
there is no backend eigenvalue or auto-inertia computation. Required checks:

- total mass: exact integer equality;
- group first moment: at most `4 ukg*m` per component after table quantization;
- group authoritative inertia: at most `6 micro-kg*m^2` per component after
  table quantization;
- solver versus authoritative tensor: at most the row's `solver_err`, never a
  shared implicit epsilon;
- whole-body solver tensor difference: at most `5,000 micro-kg*m^2` per
  component in neutral pose.

## 3. Body rows

`bind_um` is relative to the parent body. `solver_I3` is `(xx, yy, zz)` and is
the diagonal of `I6`. Right/left product-term signs are intentional.

| Body ID | Parent | Role | Mapping | bind_um | mass_ukg | CoM_um | I6 | solver_err |
|---|---|---|---|---|---:|---|---|---:|
| `body.pelvis` | — | `PhysicalRoot` | pelvis | `[0,943500,0]` | 11777000 | `[0,0,-70700]` | `[57900,0,0,87100,0,102800]` | 0 |
| `body.torso-pitch` | pelvis | `NonCollidingCarrier` | torso | `[0,81500,-100700]` | 1000 | `[0,320000,-30000]` | `[1,0,0,1,0,1]` | 0 |
| `body.torso-roll` | torso-pitch | `NonCollidingCarrier` | torso | `[0,0,0]` | 1000 | `[0,320000,-30000]` | `[1,0,0,1,0,1]` | 0 |
| `body.torso-yaw` | torso-roll | `PhysicalTorsoHead` | torso | `[0,0,0]` | 26824600 | `[0,320000,-30000]` | `[1431398,0,0,755498,0,1474498]` | 0 |
| `body.right-hip-pitch` | pelvis | `NonCollidingCarrier` | right-thigh | `[77260,-78490,-56276]` | 1000 | `[0,-170000,0]` | `[1,0,0,1,0,1]` | 0 |
| `body.right-hip-roll` | right-hip-pitch | `NonCollidingCarrier` | right-thigh | `[0,0,0]` | 1000 | `[0,-170000,0]` | `[1,0,0,1,0,1]` | 0 |
| `body.right-hip-yaw` | right-hip-roll | `PhysicalThigh` | right-thigh | `[0,0,0]` | 9299400 | `[0,-170000,0]` | `[141198,0,0,35098,0,133898]` | 0 |
| `body.right-knee` | right-hip-yaw | `PhysicalShankKnee` | right-knee | `[-2750,-407960,-8090]` | 3793700 | `[1451,-178403,7947]` | `[54816,26,-1,5117,111,54103]` | 111 |
| `body.right-ankle-pitch` | right-knee | `PhysicalTalus` | right-ankle | `[1485,-396465,-1910]` | 100000 | `[0,0,0]` | `[1000,0,0,1000,0,1000]` | 0 |
| `body.right-ankle-roll` | right-ankle-pitch | `PhysicalFoot` | right-ankle | `[7920,-41950,-48770]` | 1466600 | `[-2425,26160,116748]` | `[7599,-79,344,6524,544,1675]` | 544 |
| `body.left-hip-pitch` | pelvis | `NonCollidingCarrier` | left-thigh | `[-77260,-78490,-56276]` | 1000 | `[0,-170000,0]` | `[1,0,0,1,0,1]` | 0 |
| `body.left-hip-roll` | left-hip-pitch | `NonCollidingCarrier` | left-thigh | `[0,0,0]` | 1000 | `[0,-170000,0]` | `[1,0,0,1,0,1]` | 0 |
| `body.left-hip-yaw` | left-hip-roll | `PhysicalThigh` | left-thigh | `[0,0,0]` | 9299400 | `[0,-170000,0]` | `[141198,0,0,35098,0,133898]` | 0 |
| `body.left-knee` | left-hip-yaw | `PhysicalShankKnee` | left-knee | `[2750,-407960,-8090]` | 3793700 | `[-1451,-178403,7947]` | `[54816,-26,1,5117,111,54103]` | 111 |
| `body.left-ankle-pitch` | left-knee | `PhysicalTalus` | left-ankle | `[-1485,-396465,-1910]` | 100000 | `[0,0,0]` | `[1000,0,0,1000,0,1000]` | 0 |
| `body.left-ankle-roll` | left-ankle-pitch | `PhysicalFoot` | left-ankle | `[-7920,-41950,-48770]` | 1466600 | `[2425,26160,116748]` | `[7599,79,-344,6524,544,1675]` | 544 |
| `body.right-shoulder-pitch` | torso-yaw | `NonCollidingCarrier` | right-arm | `[170000,371500,3155]` | 1000 | `[0,-164502,0]` | `[1,0,0,1,0,1]` | 0 |
| `body.right-shoulder-roll` | right-shoulder-pitch | `NonCollidingCarrier` | right-arm | `[0,0,0]` | 1000 | `[0,-164502,0]` | `[1,0,0,1,0,1]` | 0 |
| `body.right-shoulder-yaw` | right-shoulder-roll | `PhysicalUpperArm` | right-arm | `[0,0,0]` | 2030500 | `[0,-164502,0]` | `[13407,0,0,4119,0,11944]` | 0 |
| `body.right-elbow` | right-shoulder-yaw | `PhysicalForearmHand` | right-arm | `[-9595,-286273,13144]` | 1672500 | `[20332,-178978,-6690]` | `[19867,1785,161,2289,-794,19297]` | 1785 |
| `body.left-shoulder-pitch` | torso-yaw | `NonCollidingCarrier` | left-arm | `[-170000,371500,3155]` | 1000 | `[0,-164502,0]` | `[1,0,0,1,0,1]` | 0 |
| `body.left-shoulder-roll` | left-shoulder-pitch | `NonCollidingCarrier` | left-arm | `[0,0,0]` | 1000 | `[0,-164502,0]` | `[1,0,0,1,0,1]` | 0 |
| `body.left-shoulder-yaw` | left-shoulder-roll | `PhysicalUpperArm` | left-arm | `[0,0,0]` | 2030500 | `[0,-164502,0]` | `[13407,0,0,4119,0,11944]` | 0 |
| `body.left-elbow` | left-shoulder-yaw | `PhysicalForearmHand` | left-arm | `[9595,-286273,13144]` | 1672500 | `[-20332,-178978,-6690]` | `[19867,-1785,-161,2289,-794,19297]` | 1785 |

Every row uses `solver_mass = mass`, `solver_CoM = CoM`, identity principal
orientation and `solver_I3 = [I6.xx, I6.yy, I6.zz]`. The complete V2 record
stores those fields explicitly; this shorthand is not permission for compiler
defaults.

## 4. Collider, filter and contact rows

All solid rows use layer `10` (`HumanoidSolid`) and mask `0x0000000000001c01`
(ground layer 0 plus humanoid layers 10..12). Carriers use no collider and no
mask. The material is `physics-material.humanoid-body.v1`; the sole uses
`physics-material.humanoid-sole.v1`. Box dimensions are half-extents.

| Collider ID | Body | Geometry and local pose (`um`) | Contact role |
|---|---|---|---|
| `collider.pelvis` | pelvis | box `[145000,100000,110000]`, centre `[0,-20000,-30000]` | `PelvisGround` |
| `collider.torso` | torso-yaw | box `[160000,220000,120000]`, centre `[0,300000,-20000]` | `TorsoGround` |
| `collider.head` | torso-yaw | sphere `r=105000`, centre `[0,570000,0]` | `HeadGround` |
| `collider.right-thigh` | right-hip-yaw | box `[75000,190000,75000]`, centre `[0,-195000,0]` | `ThighGround` |
| `collider.left-thigh` | left-hip-yaw | box `[75000,190000,75000]`, centre `[0,-195000,0]` | `ThighGround` |
| `collider.right-shank` | right-knee | box `[55000,185000,55000]`, centre `[0,-190000,0]` | `ShankGround` |
| `collider.right-knee` | right-knee | sphere `r=65000`, centre `[0,0,20000]` | `KneeGround` |
| `collider.left-shank` | left-knee | box `[55000,185000,55000]`, centre `[0,-190000,0]` | `ShankGround` |
| `collider.left-knee` | left-knee | sphere `r=65000`, centre `[0,0,20000]` | `KneeGround` |
| `collider.right-talus` | right-ankle-pitch | sphere `r=40000`, centre `[0,0,0]` | `AnkleGround` |
| `collider.left-talus` | left-ankle-pitch | sphere `r=40000`, centre `[0,0,0]` | `AnkleGround` |
| `collider.right-foot` | right-ankle-roll | box `[55000,30000,130000]`, centre `[0,11365,80000]` | `FootWithSoleFeature` |
| `collider.left-foot` | left-ankle-roll | box `[55000,30000,130000]`, centre `[0,11365,80000]` | `FootWithSoleFeature` |
| `collider.right-upper-arm` | right-shoulder-yaw | box `[55000,135000,55000]`, centre `[0,-140000,0]` | `UpperArmGround` |
| `collider.left-upper-arm` | left-shoulder-yaw | box `[55000,135000,55000]`, centre `[0,-140000,0]` | `UpperArmGround` |
| `collider.right-forearm` | right-elbow | box `[35000,110000,40000]`, centre `[20000,-120000,0]` | `ForearmGround` |
| `collider.right-hand` | right-elbow | box `[45000,65000,30000]`, centre `[40000,-305000,-15000]` | `HandGround` |
| `collider.left-forearm` | left-elbow | box `[35000,110000,40000]`, centre `[-20000,-120000,0]` | `ForearmGround` |
| `collider.left-hand` | left-elbow | box `[45000,65000,30000]`, centre `[-40000,-305000,-15000]` | `HandGround` |

Sole feature rows are the bottom faces of the two foot boxes:

| Feature ID | Body-local rectangle | Area | Skill roles |
|---|---|---:|---|
| `feature.right-sole` | `x=[-55000,55000], y=-18635, z=[-50000,210000]` | `0.028600 m^2` | locomotion sole support, recovery sole support |
| `feature.left-sole` | `x=[-55000,55000], y=-18635, z=[-50000,210000]` | `0.028600 m^2` | locomotion sole support, recovery sole support |

Heel/forefoot effectors are at `y=-18635`: heel `z=-35000`, forefoot
`z=180000`. Palm effectors are the centres of the hand boxes. Hands and
forearms are allowed support only in brace/get-up; knees only in get-up; every
non-sole ground role is forbidden locomotion after TRAIN-3 grace processing.

Self-collision is enabled for non-adjacent solid bodies. Exclusions are every
joint endpoint pair plus these physical/source-adjacency pairs hidden by
carriers:

```text
(pelvis, left-hip-yaw)       (pelvis, right-hip-yaw)
(torso-yaw, left-shoulder-yaw) (torso-yaw, right-shoulder-yaw)
```

Carrier pairs require no exclusion because carriers have no simulation shape.
No left/right limb pair and no hand/torso non-adjacent pair is excluded.

## 5. Joint rows

Axes are exact principal Q1.30 values: `+X=[1073741824,0,0]`,
`+Y=[0,1073741824,0]`, `+Z=[0,0,1073741824]`; a leading minus negates the
non-zero component. `parent_frame.translation = child.bind_um`, child-frame
translation is zero, and both rotations are identity. Neutral is `0 urad` for
all rows. `mirror=+1` means the mirrored semantic coordinate retains its value;
the left roll/yaw axis sign supplies the physical reflection.

| Joint ID | Parent -> child | Semantic / axis | hard `[min,max]` urad | soft `[min,max]` urad | max vel urad/s | Actuator profile | Mirror |
|---|---|---|---|---|---:|---|---|
| `joint.torso-pitch` | pelvis -> torso-pitch | lumbar flexion, `+X` | `[-523599,785398]` | `[-349066,610865]` | 4000000 | spine | — |
| `joint.torso-roll` | torso-pitch -> torso-roll | lumbar side bend, `+Z` | `[-523599,523599]` | `[-349066,349066]` | 4000000 | spine | — |
| `joint.torso-yaw` | torso-roll -> torso-yaw | lumbar axial rotation, `+Y` | `[-785398,785398]` | `[-523599,523599]` | 4000000 | spine | — |
| `joint.right-hip-pitch` | pelvis -> right-hip-pitch | hip flexion, `+X` | `[-523599,2094395]` | `[-349066,1919862]` | 8000000 | hip-pitch | left-hip-pitch `+1` |
| `joint.right-hip-roll` | right-hip-pitch -> right-hip-roll | hip adduction, `+Z` | `[-872665,523599]` | `[-698132,436332]` | 8000000 | hip-roll | left-hip-roll `+1` |
| `joint.right-hip-yaw` | right-hip-roll -> right-hip-yaw | hip rotation, `+Y` | `[-698132,698132]` | `[-523599,523599]` | 8000000 | hip-yaw | left-hip-yaw `+1` |
| `joint.right-knee` | right-hip-yaw -> right-knee | knee flexion, `+X` | `[0,2443461]` | `[0,2356194]` | 10000000 | knee | left-knee `+1` |
| `joint.right-ankle-pitch` | right-knee -> right-ankle-pitch | plantar/dorsiflexion, `+X` | `[-698132,523599]` | `[-523599,436332]` | 8000000 | ankle-pitch | left-ankle-pitch `+1` |
| `joint.right-ankle-roll` | right-ankle-pitch -> right-ankle-roll | inversion/eversion, `+Z` | `[-349066,349066]` | `[-261799,261799]` | 8000000 | ankle-roll | left-ankle-roll `+1` |
| `joint.left-hip-pitch` | pelvis -> left-hip-pitch | hip flexion, `+X` | `[-523599,2094395]` | `[-349066,1919862]` | 8000000 | hip-pitch | right-hip-pitch `+1` |
| `joint.left-hip-roll` | left-hip-pitch -> left-hip-roll | hip adduction, `-Z` | `[-872665,523599]` | `[-698132,436332]` | 8000000 | hip-roll | right-hip-roll `+1` |
| `joint.left-hip-yaw` | left-hip-roll -> left-hip-yaw | hip rotation, `-Y` | `[-698132,698132]` | `[-523599,523599]` | 8000000 | hip-yaw | right-hip-yaw `+1` |
| `joint.left-knee` | left-hip-yaw -> left-knee | knee flexion, `+X` | `[0,2443461]` | `[0,2356194]` | 10000000 | knee | right-knee `+1` |
| `joint.left-ankle-pitch` | left-knee -> left-ankle-pitch | plantar/dorsiflexion, `+X` | `[-698132,523599]` | `[-523599,436332]` | 8000000 | ankle-pitch | right-ankle-pitch `+1` |
| `joint.left-ankle-roll` | left-ankle-pitch -> left-ankle-roll | inversion/eversion, `-Z` | `[-349066,349066]` | `[-261799,261799]` | 8000000 | ankle-roll | right-ankle-roll `+1` |
| `joint.right-shoulder-pitch` | torso-yaw -> right-shoulder-pitch | shoulder flexion, `+X` | `[-1047198,2967060]` | `[-872665,2792527]` | 10000000 | shoulder-pitch | left-shoulder-pitch `+1` |
| `joint.right-shoulder-roll` | right-shoulder-pitch -> right-shoulder-roll | shoulder ab/adduction, `+Z` | `[-1745329,1570796]` | `[-1570796,1396263]` | 10000000 | shoulder-roll | left-shoulder-roll `+1` |
| `joint.right-shoulder-yaw` | right-shoulder-roll -> right-shoulder-yaw | shoulder rotation, `+Y` | `[-1570796,1570796]` | `[-1308997,1308997]` | 10000000 | shoulder-yaw | left-shoulder-yaw `+1` |
| `joint.right-elbow` | right-shoulder-yaw -> right-elbow | elbow flexion, `+X` | `[0,2617994]` | `[0,2530727]` | 12000000 | elbow | left-elbow `+1` |
| `joint.left-shoulder-pitch` | torso-yaw -> left-shoulder-pitch | shoulder flexion, `+X` | `[-1047198,2967060]` | `[-872665,2792527]` | 10000000 | shoulder-pitch | right-shoulder-pitch `+1` |
| `joint.left-shoulder-roll` | left-shoulder-pitch -> left-shoulder-roll | shoulder ab/adduction, `-Z` | `[-1745329,1570796]` | `[-1570796,1396263]` | 10000000 | shoulder-roll | right-shoulder-roll `+1` |
| `joint.left-shoulder-yaw` | left-shoulder-roll -> left-shoulder-yaw | shoulder rotation, `-Y` | `[-1570796,1570796]` | `[-1308997,1308997]` | 10000000 | shoulder-yaw | right-shoulder-yaw `+1` |
| `joint.left-elbow` | left-shoulder-yaw -> left-elbow | elbow flexion, `+X` | `[0,2617994]` | `[0,2530727]` | 12000000 | elbow | right-elbow `+1` |

The knee hard maximum `140 deg` comes from the declared high-flexion
cross-check, while `0 deg` remains a hard non-hyperextension boundary. Shoulder
limits combine the source axes with the task-required brace/get-up envelope;
they remain below a full unconstrained turn. Every required recovery reference
must still validate against the soft range in `TRAIN-4`; this table does not
waive a failed clip.

## 6. Fixed actuator, PD and action rows

`Kp/Kd` use Q16 SI. Effort is signed `uN*m`; rate is `uN*m/s`; power is `uW`;
work is positive `uJ` per 60 Hz motor tick. Negative/positive target delta is
symmetric here but stored separately in V2. These are conservative engineering
bounds, not injury thresholds or a claim of muscle activation fidelity.

| Profile | Kp_q16 | Kd_q16 | effort `[neg,pos]` | rate | power | work/tick | residual urad | delta `[neg,pos]` urad |
|---|---:|---:|---|---:|---:|---:|---:|---|
| spine | 19660800 | 1966080 | `[-180000000,180000000]` | 1800000000 | 600000000 | 10000000 | 150000 | `[-80000,80000]` |
| hip-pitch | 29491200 | 2949120 | `[-320000000,320000000]` | 3200000000 | 1200000000 | 20000000 | 200000 | `[-100000,100000]` |
| hip-roll | 22937600 | 2293760 | `[-220000000,220000000]` | 2200000000 | 800000000 | 13333333 | 150000 | `[-80000,80000]` |
| hip-yaw | 19660800 | 1966080 | `[-180000000,180000000]` | 1800000000 | 700000000 | 11666667 | 150000 | `[-80000,80000]` |
| knee | 32768000 | 3276800 | `[-350000000,350000000]` | 3500000000 | 1200000000 | 20000000 | 220000 | `[-100000,100000]` |
| ankle-pitch | 26214400 | 2621440 | `[-220000000,220000000]` | 2200000000 | 800000000 | 13333333 | 150000 | `[-80000,80000]` |
| ankle-roll | 19660800 | 1966080 | `[-140000000,140000000]` | 1400000000 | 500000000 | 8333333 | 120000 | `[-60000,60000]` |
| shoulder-pitch | 11796480 | 1179648 | `[-120000000,120000000]` | 1200000000 | 400000000 | 6666667 | 250000 | `[-120000,120000]` |
| shoulder-roll | 10485760 | 1048576 | `[-100000000,100000000]` | 1000000000 | 350000000 | 5833333 | 200000 | `[-100000,100000]` |
| shoulder-yaw | 9175040 | 917504 | `[-80000000,80000000]` | 800000000 | 300000000 | 5000000 | 200000 | `[-100000,100000]` |
| elbow | 9175040 | 917504 | `[-90000000,90000000]` | 900000000 | 300000000 | 5000000 | 250000 | `[-120000,120000]` |

The lower-limb scale is anchored to published maximum-voluntary-joint-torque
measurements that normalize young-adult lower-limb strength by body
weight/height and show strong angle/velocity dependence (Anderson, Madigan and
Nussbaum, 2007). The selected caps are rounded, symmetric control envelopes
large enough for gait/recovery, not copied subject maxima. Upper-body and spine
caps are lower because their task is balance, brace and get-up rather than
lifting. `Kd = Kp/10` is the initial fixed damping profile; it is accepted only
after TRAIN-2 passive step/hold/settle tests. Any gain adjustment creates a new
profile revision/hash before data collection.

At every motor tick the reference plus residual is intersected with the soft
range and previous target plus the signed delta before PD. At every physics
substep, effort, effort-rate, velocity, power and accumulated positive work are
hard clamps outside reward. A zero or absent energy field never means unlimited.

## 7. Neutral/support derivation and reviews

### 7.1. Exact neutral geometry

The pelvis root height is `943500 um`, not copied from Stage 0. Head top:

```text
943500 root + 81500 torso joint + 570000 head centre + 105000 radius
= 1700000 um
```

Each neutral sole plane is exactly ground-aligned:

```text
943500 root - 78490 hip - 407960 knee - 396465 ankle - 41950 subtalar
+ 11365 foot-box centre - 30000 foot half-height
= 0 um
```

The two sole rectangles are `0.110 m x 0.260 m`; neutral left/right centres
remain separated by the authored hip placement. No other neutral collider has
a point at or below the sole plane.

### 7.2. Independent table review

| Review | Result | Evidence/disposition |
|---|---|---|
| Source/license | Pass | explicit SimTK MIT grant, notice and exact official-mirror hash recorded |
| Mass conservation | Pass | integer group table sums to exactly `75,337,000 ukg` |
| CoM/inertia mapping | Pass for specification | parallel-axis merge recorded; products and per-row solver errors retained, not silently discarded |
| Spine | Pass | hard pitch `-30..45 deg`, roll `+/-30 deg`, yaw `+/-45 deg`; no V1 reverse fold |
| Hips | Pass | source gait hard envelope retained; soft envelope is strictly inner |
| Knees | Pass | `0..140 deg`; no negative/hyperextension side |
| Ankles | Pass | pitch `-40..30 deg`, roll `+/-20 deg`; no axial twist channel |
| Shoulders | Pass | recovery-capable but bounded pitch/roll/yaw; no unrestricted turn |
| Elbows | Pass | `0..150 deg`; no negative/hyperextension side |
| Mirror closure | Pass | left X positions negate right; roll/yaw axes negate; value sign remains explicit `+1` |
| Neutral stature/sole | Pass | exact integer equations above; no simulator default used |
| Collision coverage | Pass for specification | head, torso, pelvis, limbs, knees, feet, forearms and hands have authored rows; carriers are explicitly non-colliding |
| Passive PhysX behavior | NotRun | belongs to TRAIN-2 implementation; cannot be inferred from a table |
| Recovery soft-ROM clips | NotRun | belongs to TRAIN-4 corpus validation |

The analytical review closes `TRAIN-1`; it does not turn NotRun physical or
motion checks into Pass. TRAIN-2 must reject the profile if its compiler cannot
represent any row exactly within the declared projection bounds.
