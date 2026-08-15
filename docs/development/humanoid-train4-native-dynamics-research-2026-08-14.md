# TRAIN-4 native-dynamics research decision — 2026-08-14

| Field | Value |
| --- | --- |
| Scope | Optimizer-free research after complete-clip V9 fresh-scene rejection |
| Status | `R123_INVALID / R127_INVALID / R130_INVALID / R133_PASS / R134_COMPLETE / R135_CONFORMANCE_NEXT` |
| Acceptance authority | Fresh scene under ADR-070 |
| Claim ceiling | Research and generated-test design only; no corpus admission or training |

## Why this cycle exists

R93 proves a narrow but useful fact: one deterministic V9 trajectory exists for
each selected complete clip and satisfies every frozen *offline kinematic*
constraint. R94 then disproves the stronger hypothesis that those trajectories
are dynamically reproduced by the accepted PhysX environment and its frozen
zero-residual fixed-PD controller.

This is a stage boundary, not a reason to weaken the gate. The work returns to
bounded research before another solver change or expensive native run.

## Immutable evidence

| Evidence | Identity | Result |
| --- | --- | --- |
| R49 V7 fresh all-17 | report file SHA-256 `6b977a870c50b25545c2b73bc371575b38b40f6a914d1638ab19a3c7a56b5f0c` | `PASS 17/17`, required safety `0`, passing-control regressions `0` |
| R93 V9 complete clips and exact slices | canonical/file SHA-256 `7ee041b7710302b9fae909b0cb34c0de3a6c2df257032cd0e1c7dcaf16afefeb` / `53984129cc45d295ced9ef4f49ea8532d0b224c3ccc19980e8b986cee7604d70` | All three clips and all 17 slices pass offline; `390` arrays over `30` overlap pairs agree byte-for-byte; no contact point is deleted |
| R94 V9 fresh all-17 | canonical/file SHA-256 `0d429faff356e3240a7f2906115566b565fc0800043dd7ffa04544b2edb38925` / `4aab74888d50fae2ab644445597d3f7f7a64217f078ceca4b4f8045ed34f0929` | `FAIL 7/17`, `gate_decision=STOP_AND_RESEARCH` |
| R95 V7↔V9 differential audit | canonical/file SHA-256 `ec453b347e9323819ba706daa40949a91924e7a359c9225ab7ce031dcf60bd6f` / `8287e3ef751d22a46cc312d1dc7a79f3245302a2fa4d3da965cd7e5efd652f76` | `COMPLETE`; selects ordinals `2` and `10` for at most two fresh counterfactuals |
| R96 direct emitted-acceleration candidate | canonical/file SHA-256 `94981a8b14b48c6ad01c331676fee8a4a24e9aa764a1cbdf13c090d524aaa2ae` / `d4fe0ac9aec9ba56b7ffed6ac097ba205f1ee8afb3c38eb7570ec7034189d46c` | Offline `PASS`; case `10` acceleration `-42.70%`, jerk `-54.67%`, modes and unselected channels byte-identical |
| R97 two-case fresh discriminator | canonical/file SHA-256 `63d1331529905bd25354884331973bfed4578eb061283c40f7639b31a2f4bfcd` / `6b7ac6a1f2dad49f2c85ebbe5139859ab855549c905e86af2bdfcce4b6b0ac62` | `FAIL 1/2`; contact case passes, derivative case regresses to a new joint-velocity reason; `gate_decision=STOP_AND_RESEARCH` |
| R98 V9 native trace | canonical/file SHA-256 `b8e367e873d0384f4a849d25a338751e0a9c56aa7c4143923c1446b63b016372` / `fd6deb633a5b53c4a00b5de6943a5ce12ae59d4b0a232629586807107ef8d77b` | `COMPLETE`, exact R94 outcome reproduced, `40/40` physical substeps captured, trace SHA-256 `b032098868b6bf732155cf8211b4070704363e1f85f070294cb510d116240bcd` |
| R98 V11/control native trace | canonical/file SHA-256 `06727950f3c2b8b6344b1bf15c2c975d52e9d7630a0edcd9503993dfb9f87910` / `5143a7ef5bb068c97594926d226460b657c87c6ba7f9b94a123b0f03c79e31cb` | `COMPLETE`, exact R97 PASS/FAIL reproduced, `44/44 + 36/36` substeps captured; no acceptance authority |
| R99 matched V7 native trace | canonical/file SHA-256 `19aa8ddfce365fadce4146067470e60e6bd48d0c83f80e9064c587fe7e3613db` / `04553001223cdcb638a8cf8029b5f6a453610ca026fa995833418f97ed6b22dd` | Exact R49 case-10 `PASS 11/11` reproduced with `44/44` substeps; trace SHA-256 `552ddd671bc5501500504d54876451e5e93fd4a6979ff1b6b50b5b1649c2b40d`; no acceptance authority |
| R100 one-scalar input | canonical/file/artifact SHA-256 `59d556f5a71943e6bdf60beb3dabbee03d5cc2a7618a341966a6ce3a5d050e78` / `e66cdda52f674d76a7ac1dd70cd60810c02143a9efdefed78804b52751a6a3da` / `427b5e1895519fe83c40189406c3027f14d9b9aa36d50e3b45e300d516f5761a` | Offline proof changes exactly one velocity cell and zero other array elements; PhysX/optimizer/training `0` |
| R100 native trace | canonical/file SHA-256 `024e2251e4518f83c1f5fba4d22142afa83d6a23d8dfdd04e41c40ba1d4210b3` / `afee35e799be7522c434998583175d71f4629cdeeb06e0009fffde9d45629426` | `FAIL` tick `5`, new right-ankle hard impact `6092658 µN·s`; complete `20/20` trace SHA-256 `24489460446a9bee160508b7e16b4125e43cb5ab1f70dbb4440710e754b6eeae` |
| R101 velocity-vector input | canonical/file/artifact SHA-256 `42cc653095911a71a01d6ab3750228e2f47e6190a9fc4a378ce7a19840c183fb` / `096b2f94eaa59a555da82165276c39456cf0ee33cec9123489271548b6deeb2c` / `2dab3cbea1e71f60c328a48b3ec52a3a95af23b6c0214ac05c77c0c3a13ae58a` | Exactly `18` frame-0 joint-velocity cells change to V7; all other array elements remain V9; PhysX/optimizer/training `0` |
| R101 native trace | canonical/file SHA-256 `04b91df1ca6be06c10af9fbf11505457fed8e5a25e2c50abe3f8523bdd684cb6` / `08ef6a379bfc2379ed43b1373f784025760d5b2bdce9bd54ab164376b1afc468` | `FAIL` tick `10`, right-ankle-pitch hard ROM plus remote hard impact `6006560 µN·s`; complete `40/40` trace SHA-256 `364ed05872fb522e6afbab5c945924fda5eaa09fd14d796163255c9ecb83ac31` |
| R102 native-rollout audit | canonical/file/profile SHA-256 `b881f8a7a70542f07c045e2451458a427a38067078520a07dc02dac4034546f6` / `ef5f95eafe18f513abfa90803bc7ff67e8db756f327bd2b4d27f27c320b8ebee` / `ea7159cf69d530e3242aab03638415212ebeb7b2be0f369b8ae13ab5446d167c` | Clean `COMPLETE`: all frozen identities/facts reproduce; PhysX/candidate/trajectory/optimizer/training counts are `0`; candidate search remains `NOT_AUTHORIZED` |
| R103 target-knot formulation v2 | canonical/file/profile SHA-256 `81e213c73a9a0ad071dd7459b0f6e3c285fe18985620b81045e3d4d4c90ddddc` / `0d213ecae85157ef4f2748fd7b82b4af245f43f97f05c26b82c43be03bccf954` / `cfb17e8f01ec7a120110646377206cf0fce173c41263d267fcde4c1f893d2378` | Clean `COMPLETE`: descriptor-ordered DoF identity is hash-bound; three convex V9→V7 scalar knots define exactly `27` report-only targets; candidate/PhysX/optimizer/training counts are `0` |
| R104 exact offline preflight v2 | canonical/file/profile SHA-256 `d80504975367c13e01d724e9bbbae47335ed4d9ab366179b44851741813197ab` / `1b03170a2679d5061ed6d72c520fc7b0890431672f24ba3fa1f825c9454ebfa8` / `f7547b66d65341155b583735b9dcb52c08680810db102662e46064814b45076d` | Clean `STOP_AND_RESEARCH`: only the zero control passes; all `26/26` nonzero targets fail exact offline contact and/or velocity bounds; PhysX/optimizer/training counts are `0` |
| R105 local feasible-direction audit | canonical/file/profile SHA-256 `c107230f01f75d25987ca0e9ac07d81cdda71fe94bca1889509d4f5504436f50` / `19971615f285608903877a265e0387e7dbb963254e1b56493c99c43617e39ae9` / `48cf5b605e5fc6b3ae36b72ccc85a05d6fffeb6ccaa6d635d468c3e5c5b82d9e` | Clean `COMPLETE`: early/middle bases collapse; late basis retains `9722 bp` anchor component and `9872 bp` cosine after a feasible local projection; only R106 formulation is permitted |
| R106 projected-direction formulation v2 | canonical/file/profile SHA-256 `67a94a01f624cae4db9615646ee00a8ef00920119ad017f3ddead60d2e0c2996` / `30512398d0e7030e95a61b8a662417b9ea1436ae96b58d0cdb0edaa3fe6b0679` / `6e67882da4e305aa0566c93d44216fc92f0318173a620aac4c2095eefc4a9937` | Clean `COMPLETE`: selects only late offset `11`, freezes reconstruction/quantization and permits one in-memory R107 exact audit; projection/candidate/PhysX/optimizer/training counts are `0` |
| R107 projected-direction exact audit | canonical/file/profile SHA-256 `5a3c26ca731ac2734637a3d9953d84eca15c6553fb97c9da2ed9a12c2f7781fa` / `4095dfa6517847c5babfb9fb958e015908fc8c560f4e7c7083f840918b4f9a45` / `00b826d3aa11f1d1b4df66f99856ba5ed8dd3a08256a662dbfdb8a531c0e4539` | Clean `FAIL`: contact/collider geometry/ROM/root velocity retain V9 PASS metrics, but exact joint velocity is `2501/2500 bp`; selects progressive formulation with zero artifacts/PhysX/optimizer/training |
| R108 progressive-kinodynamic formulation | canonical/file/profile SHA-256 `4ab1ccbc697fdf97efadc9e53ca6f2605956000927c88ed76f1960917590f9e8` / `6a288d0cd2bdfc5f580c1dabb96ecf63ed71f79f09ae00bc40ebe15ca18fd4fb` / `1f5a006eb0a2d0f1bb575b67928e6bb9a86b42c00659393994189ebf8ce62b0f` | Clean `COMPLETE`: freezes model-identity→KTO→inverse-dynamics→kinodynamics gates; only R109 identity preflight is authorized and every solve/work count is `0` |
| R109 dynamics-model identity preflight | canonical/file/profile SHA-256 `2867aecd144d7996d3bf5bd0b6498dc1a5d480f7b8060106a97c5c6fc07da784` / `97f149b12f5f4a6694da04298df774f33fd4854e93b54f3096d8882f2c1efc85` / `961926664ca8ff08a4c384180092dcbb7cb6591880bb8501148fef04d1e65ed0` | Clean `FAIL / STOP_INVALID_MODEL_LINEAGE`: structural/USD/gravity/controller identity closes; canonical material/combine and contact-wrench ownership do not; all solve/work counts are `0` |
| R110 canonical-material/point-force formulation v2 | canonical/file/profile SHA-256 `83408b97b6ba13dc801b4d9b4f68e55146f9f39b09a451c238b24cc4c8c7d88d` / `234c7e51c3e6135bff55e503d8ce2bb58946c6cbef36598d92641ce7deb72637` / `85604a87bfc05ef170d21ff49d217d21327095414fb12b565efe76eb1afb9b18` | Clean `COMPLETE`: freezes a full SPEC-26 material successor, exact combine profile and solver-private ordered point forces; only R111 implementation is authorized and every runtime/solve/work count is `0` |
| R111 canonical material-lineage implementation | canonical/file/profile SHA-256 `eafc8fc7f5bc64706b53c313cff143e0c0f8bd7684371e714d94bdef3e86f058` / `c8b5663c86fdf89ba4f8729fdccb2860328fd5e7b6144388ab892d5d4bc97bd7` / `a5cb5a3330eddefaeff33639e79c885ecbace7dfb8bddbf86de32f11b04bf44e` | Clean `PASS`: Accepted ADR-071, successor contracts/compiler/mirror and native ABI 4 close engine/native lineage; only static R112 USD/Isaac implementation is authorized and every scene/solve/work count is `0` |
| R112 derived USD/Isaac material lineage | canonical/file/profile SHA-256 `dfb3bd892b04023054ce947127743e7ada78b40000a93e22887101c544c493f2` / `a615359b7855ca270dacced899960d71ab4a43c8aab69403fd154a55a0067778` / `c98c383117f03b5bb594855c831f2aa3a31453ac94b6f4fa2dc483015a3b7f0a` | Clean `PASS`: exact humanoid/ground material prims, `19+1` bindings and explicit Isaac consumption close the derived lineage; only report-only R113 is authorized and every scene/solve/work count is `0` |
| R113 clean dynamics-model identity | canonical/file/profile SHA-256 `3ac92ae2ca508234a52d77f0414ad5557f1164028e51a3938cc045ac4c5147cf` / `de584af485e789a19e457708cf154193c5d870dd1deb8ee29a61b6abddb6541c` / `fa52cf18be25144893fb1d4da57bbae2fc2056d00d13e4bac012276ccc8cdf5d` | Clean `PASS`: all R108 identity groups close with zero blockers; only report-only R114 KTO formulation is authorized, with one preflight and zero scene/solve/work counts |
| R114 quantization-aware KTO execution formulation revision 2 | canonical/file/profile SHA-256 `7a735320509a303f9feacba79087f2042d451d526b57585d4fe4041603b46ae4` / `2666a275180b222c014eea91051ff4d3ebdb16a5740eb742ed2cca3a83b3d958` / `8e26b84de2a25e07d19cd93400bc8c04a6bc4840fb9a4d8eb6ab88237db59a43` | Clean `COMPLETE`: supersedes underdefined v1 before any solve; freezes `69687` q/v/a scalars, continuous-V9 yaw emission, exact progress/resource gates and one R115 execution; all scene/solve/work counts remain `0` |
| R115 single bounded KTO execution | canonical/file/profile SHA-256 `b7baa0f4337597c1c61748255535d1102bbff35d0ce560a8a661ed6be7685433` / `ed89c2733a376f5f50694fc01029abcf9791f2a186054196461ce2a787e8caa8` / `427958340debb62a3dfc7b9309411184c8e922965a5cef694163cae61d0b9e75` | Clean `FAIL / STOP_AND_RESEARCH`: one solved QP and six exact emitted audits find no all-gate step; no cache, candidate, scene or downstream execution exists and retry is forbidden |
| R115-RC1 analytic-contact linearization research | canonical/file/profile SHA-256 `5edfe0613e2e4f9327cd1bfb6922c96e84ce8056144b05f9617787de0f883475` / `74f6b7f7c13565618a2972d83b568d411ae8f4598b76699cd176b328f4a42f51` / `7e4186d0e44049102ab1d61e0cf49f51f3682e88557f9c8c2dd2d92c5c53c3f4` | Clean `COMPLETE / CONFIRMED_ANALYTIC_CONTACT_LINEARIZATION_IDENTITY_GAP`: all `3723` analytic QP rows omit q derivatives although the exact kernel has nonzero q sensitivity; permits only report-only R117 formulation, with zero solves/scenes/candidates |
| R117 exact-kernel linearization-repair formulation | canonical/file/profile SHA-256 `b0a9f07012e0f43c660019df8a1f31e7368f6130312231cf6c2bddb0f59fb7c7` / `f32f31d895ed26f1f7ec842b2125ce0329a0d5802435aaebc5973d198dc06ed4` / `eaabc22958324504756d95239ceefec3597523cf01e2d3c8633d6942db23da55` | Clean `COMPLETE`: freezes the kernel-identical full-q/v derivative, norm-squared tangent constraint, quantized re-anchoring and explicit single-bridge/restoration filter; permits only report-only R118 implementation/conformance, with zero QP/KTO/scenes/candidates |
| R118 exact-kernel full-q/v conformance | canonical/file/profile SHA-256 `23d9d556d8be630f7f2e9fe9907f394b74186ef545d0c46f7484f0efc1df45ca` / `f4d186a476c6bb73f1f001ac6e991088b7563d2aeeca49f63db6e872afb7e861` / `d97340714757c1ead27b9f571332f926690104a89fed8a2d3a6ef357c9de88f3` | Clean `PASS`: `1241/1241` active point-frames preserve the exact V9 kernel and all seven full-q/v Jacobian anchors pass; permits only a separate report-only R119 execution formulation, with zero QP/KTO/scenes/candidates |
| R119 repaired-KTO execution formulation | canonical/file/profile SHA-256 `ac38e3f0a9dfb5e900373bc5a4908168a49f3c3b799fb431bd6e5c1306a4dd1b` / `e233f9dd60ba8056e55b132167e5dbd6e952781fb15bece56cbb23e62b0c1882` / `3b48c618cffc5494601a40ac15b04df0ffba2a1b7d6aadbdfd4309b6c167dcd1` | Clean `COMPLETE`: replaces `3723` component rows with `2482` exact normal/norm-squared rows, freezes emitted-anchor conformance guards and one-process budgets; permits exactly one R120 repaired-KTO execution, with zero QP/KTO/scenes/candidates in R119 |
| R120 single repaired-KTO execution | canonical/file/profile/cache SHA-256 `dfcb05e006467ee30bab70aac00f4408acce26782fbdbf5d1cea89821c06953b` / `35e35581b4062ce3048cb564a7856787efdd59e1fa12b64eef9526200ea4f2fc` / `3dfa2f1b8357cd3452481c9518e8d1ca0ce5c0bb664b3a024fc5ce2653837d55` / `e305fc5888a1cf1dff238c32ac07707a284b215bb49d25920dfb5ee8f097afc5` | Clean direct `PASS` at `1/32`: all exact gates pass with `614 bp` strict V7 progress and no bridge; one KTO/QP, six emitted audits, zero candidate/scene/ID/kinodynamic/optimizer/training work; permits only report-only R121 fixed-PD inverse-dynamics execution formulation |
| R121 fixed-PD inverse-dynamics execution formulation | canonical/file/profile SHA-256 `4e6e9494cd7695208aa893fb898003a74f6d3f91fd1ecab583c026509c298ce3` / `5be2a83f03fd6eb29b61992cfe0995ddb2f419110ba6078bd107a21489343bf7` / `9145d5f3312d5615df84f5bef6444210e7a7a110b5d41206b0ce582bf45a8c93` | Clean `COMPLETE`: freezes `3200` independent `64 × 64` pointwise systems (`204800` variables/equalities, `4956` friction cones); cache/contact/fixed-PD preflights PASS, zero dynamics solves; permits only report-only R122 implementation/conformance |
| R122 fixed-PD inverse-dynamics implementation conformance | canonical/file/profile SHA-256 `a03f0a7e605a7e35c370e3ee12dcb7e737c24ee928d00ca92f33c2ff8958d309` / `8bb3f9cbe371f679ffa3d782ee4662de3fedc586ccf70c9d19536594c95afb81` / `21303443993a34bd527e735961f0e790aa0da88f2d94b46a4c6e1f85557e0ab2` | Clean `PASS`: all seven descriptor/M/h/J/Jdot-v, R113, exact R121 reproduction and no-solve resource discriminators pass; its one R123 authority was consumed by the invalid result below |
| R123 single fixed-PD inverse-dynamics execution | canonical/file/profile SHA-256 `3436d95d492586570cdd27fa685f2a517e1ac81ab9350fbd4f2c42f7bb5ab6c7` / `543513bf4f51797b515b718684123a9f5aeabe394bf4ea73fe20194a9d65acb2` / `492ce5da3852aa68811ce8afc6f0c5b57205ce8f2fc2ddf32dd71279b4ecda30` | Clean `INVALID / STOP_INVALID_EVIDENCE_WITHOUT_RESTART`: first flat-foot system condition `6.875e16`; one SVD, zero local solves/cache/downstream work; its R123-RC1 authority was consumed by the completed research below |
| R123-RC1 redundant-contact research | canonical/file/profile SHA-256 `ebf257991c36970e9ccf9501fe4175fc0efa0efed2e3b1a8ac1e176acef045cf` / `a35d408a901ea2c439ac387287fa03aa77c669863211fb6b0d7634ce3b0836f9` / `4ee1fd77701e638a3087cbeaa6498482e073c33133bbb89a1a0b6d134d2ee8b7` | Clean `COMPLETE / CONFIRMED_REDUNDANT_FLAT_FOOT_FORCE_GAUGE`: all nine exact geometry/artifact discriminators pass; rank ≤5, nullity ≥1; zero dynamics reconstruction/solve/downstream work; permits only report-only R125 formulation |
| R125 gauge-aware feasibility formulation | canonical/file/profile SHA-256 `ddf443610315680d0326b478212105c846a0cfda2b8bcda95567556ea1773080` / `bdc5cd5388005bbb549df7bfb723dda49a1535d2a6a39c4e31da58340e3ac0db` / `ca9cc4019e45ea316072378422e7aab304664b24034f204e41b3ccbe30e7da05` | Clean `COMPLETE`: exact `29/32/35` layouts, `2316` gauge scalars and all `4956` cones; SVD particular plus complete line-cone interval semantics; zero execution work; permits only report-only R126 conformance |
| R126 gauge-aware implementation conformance | canonical/file/profile SHA-256 `2a500b6e6514e3a5cc8cec453756089f235d66d8684718a56678c471202f3e8f` / `4931ff4c96e8bb062bed64a45681097ae70b23c615c022ba267c0ae6edb6ffd3` / `23007c0455fef7cf84da411528f9f6162cd04d97e8be86baa7be2c19ae7e87cb` | Clean `PASS`: seven real rank/nullspace anchors, five synthetic SVD cases and four decimal-oracle cone cases pass; zero real particular/gauge classification/downstream work; permits exactly one bounded R127 execution |
| R127 sole gauge-aware fixed-PD execution | canonical/file/profile SHA-256 `255f2dd900f7ca67381fd6853aa42e47a991680f723b17539e9505651d6e7a4e` / `0bdf21b2e995a3ea16f7670666a10b73379d6f6eda9dede04dea5c5e788e1a2b` / `0e3537dd36e1a148788d6e4634e4409a01d10136033bf957f7a5d684699976c2` | Clean `INVALID`: collocation 0 rank/nullspace passes but scaled equality residual is `6.204e-5`; one SVD/particular, zero gauge classification/cache/downstream; no retry |
| R130–R134 | R130 canonical `acd92a581a732c293cc9440d4215702fdf356f614c150ec4886d82a2fb197955`; R133 `f1fad2ca3c7abd49acaefd9fcd37873d02fb1d289a3081fd2039b0b1192fd3b6`; R134 `17d696166a05a24bd50a545ca31e4aabd15a72ed49444a09983c472ff6b13783` | [R130 research](humanoid-train4-r130-projected-schedule-research-2026-08-15.md) closes the exit hotspot and actuator schedule; R134 freezes the projected ID composition with zero numeric systems |

R94 is bound to clean repository commit
`5cedc41d23958023f7b4d7dcee46c34f2f230b73`, R93, the unchanged source
corpus, descriptor, reference-tracker profile, gate report and USD. It starts
17 separate worker processes and creates one fresh scene for every case.
Indexed partial reset is `NOT_RUN` and remains report-only. Optimizer steps and
training runs are both zero.

## R94 result

R94 fails seven required cases:

| Ordinal | Slice | First failing tick | Required-safety reason |
| ---: | --- | ---: | --- |
| 2 | `cmu05@25` | 2 | left-ankle hard impact, `6440089 µN·s` |
| 7 | `cmu139@626` | 1 | right-ankle hard impact, `6978856 µN·s` |
| 10 | `cmu16@238` | 10 | right ankle-pitch hard-ROM excess, `13193 µrad` |
| 11 | `cmu16@239` | 9 | right ankle-pitch hard-ROM excess `11805 µrad` plus right-ankle impact `6144072 µN·s` |
| 12 | `cmu16@241` | 8 | right ankle-pitch hard-ROM excess, `12113 µrad` |
| 14 | `cmu16@407` | 8 | left ankle-roll joint-safety/velocity excess, `985768 µrad/s` |
| 15 | `cmu16@409` | 8 | right ankle-pitch hard-ROM excess `11126 µrad` plus right-ankle impact `6003242 µN·s` |

The aggregate event counts are hard impact `4`, hard ROM `4`, joint safety
`1` and joint velocity `1`; every other required category is zero. Four cases
that were passing controls in the source discriminator regress: ordinals
`2`, `7`, `10` and `14`. Targeted failure counts therefore do not strictly
decrease for impact (`3 -> 4`) or ROM (`4 -> 4`); velocity improves only
partially (`2 -> 1`).

At the authoritative post-reset observation, the worst initial-state mismatch
over all 17 workers is only `1 µm` root position, `0 µm/s` root linear
velocity, `1 µrad/s` root angular velocity, `0 µrad` joint position and
`0 µrad/s` joint velocity. The minimum root-orientation dot is
`1073741823` in Q1.30. This rejects reset-state authorship mismatch as the
explanation for R94.

The failure timing also rejects one uniform first-tick contact explanation:
two impacts happen at ticks `1/2`, while ROM, velocity and additional impacts
appear at ticks `8..10` after the physical state has diverged from the
reference.

## Controller and architecture boundary

The accepted zero-residual path sends the current reference joint position
through slew limiting and applies the fixed explicit law
`K(q_des - q) - D*q_dot`. It has no desired-velocity term and no inverse-
dynamics or gravity feed-forward term. This matches the Isaac Lab description
of [position control with fixed impedance](https://isaac-sim.github.io/IsaacLab/v2.2.0/source/overview/core-concepts/motion_generators.html#position-control-with-fixed-impedance):
the simple controller assumes zero desired joint velocity, while a dynamics-
aware formulation additionally uses the inertia matrix and gravity vector.

This observation does **not** authorize a controller change. SPEC-35 and
ADR-070 freeze the current reference-tracking path for this gate, and the
earlier causal matrix already rejects feed-forward and velocity-zeroing as
primary fixes. The reference must be feasible for the accepted controller, or
the architecture must be changed separately through its own decision process.

## Primary-source research synthesis

The external literature supports treating this as a kinodynamic-reference
problem rather than another purely geometric tolerance problem:

- [KDMR v2](https://arxiv.org/html/2603.09956v2) treats a kinematic trajectory
  only as an initialization/tracking prior. Its nonlinear program jointly
  varies configuration, velocity, acceleration, actuator torque and scheduled
  contact force, constrains active-contact velocity to zero and enforces
  rigid-body dynamics, friction and physical limits. This is direct evidence
  against treating an independently edited joint target as a dynamically
  feasible reference.
- [SPARK](https://arxiv.org/html/2603.11480) avoids one cold full-order solve by
  progressively applying kinematic trajectory optimization over `q/v/a`,
  per-timestep inverse dynamics over acceleration/torque/contact wrench, then
  full kinodynamic optimization. Its contact stage jointly enforces contact
  velocity, joint position/velocity, torque and wrench-cone limits.
- MIT's primary [trajectory-optimization notes](https://underactuated.mit.edu/trajopt.html)
  formulate state and input values at time knots as joint decision variables
  and impose dynamics at collocation points; the related
  [multibody-dynamics notes](https://underactuated.mit.edu/multibody.html)
  show that contact force can remain an optimization variable in the implicit
  constrained dynamics. These sources support an eventual full kinodynamic
  fallback, but do not justify skipping the smaller local-feasibility audit.
- [Multi-Contact Motion Retargeting](https://arxiv.org/abs/2206.00542) couples
  whole-body kinematics with sequential force equilibrium to obtain physically
  viable multi-contact motion.
- [DeepMimic](https://arxiv.org/abs/1804.02717) is evidence that a learned
  physics controller can correct example motion, but it is not authority to
  bypass TRAIN-4 or start PPO while the reference gate is open.
- Isaac Lab `2.3.2` distinguishes applied actuator effort from computed effort
  after clipping in its [actuator model](https://isaac-sim.github.io/IsaacLab/v2.3.2/_modules/isaaclab/actuators/actuator_base.html),
  while PhysX documents that
  [`maxJointVelocity`](https://nvidia-omniverse.github.io/PhysX/physx/5.1.2/_build/physx/latest/class_px_articulation_joint_reduced_coordinate.html)
  is enforced with joint-space solver impulses. R98 therefore treats the
  frozen `9000 bp` inner guard as part of the hybrid plant, not as proof that
  the outer velocity terminal cannot be crossed.
- [Trajectory Optimization under Contact Timing Uncertainties](https://arxiv.org/abs/2407.11478)
  shows why nominal contact timing is insufficient: every candidate
  pre-contact state across the uncertain switching region must remain safe.
  This directly matches R98's safe/unsafe touchdown-phase distinction.
- [DynaRetarget v3, 2026-06-10](https://arxiv.org/abs/2602.06827) treats the
  simulator as a black-box dynamics function, rolls out control sequences by
  single shooting, reduces the search to interpolated control knots and grows
  the optimized horizon incrementally. This is the closest published shape to
  the frozen NextEngine plant because it does not require differentiating or
  replacing PhysX contact semantics.
- [DiffMimic v2, 2023-04-26](https://arxiv.org/abs/2304.03274) demonstrates
  simulator-rollout optimization of PD target angles with zero desired
  velocity, but it optimizes a policy and relaxes joint limits for gradient
  propagation. It is therefore evidence that rollout state matching can work,
  and simultaneously a counterexample to importing that method into TRAIN-4:
  learned optimization and weaker limits remain forbidden here.
- The current official PhysX
  [articulation stability guide](https://nvidia-omniverse.github.io/PhysX/ovphysx/latest/guides/articulation_stability.html)
  warns that stiff drives, high one-step angular acceleration and competing
  contacts can destabilize an articulation. It recommends controller/drive
  changes as possible remedies, but those are outside this gate; for R102 the
  bounded inference is only that the reference must be tested against the
  exact clipped/rate-limited plant rather than an open-loop derivative proxy.

The inference for NextEngine is deliberately smaller than those methods. R98
has already rejected contact as the first cause and isolated pre-contact
closed-loop phase divergence. R99 now compares the exact passing V7 trajectory
with the two failed variants before any bounded construction change is chosen.

## Ranked hypotheses

| ID | Hypothesis | Current evidence | Discriminator |
| --- | --- | --- | --- |
| H22 | V9 satisfies pose/velocity geometry but asks the fixed PD plant for dynamically infeasible acceleration or effort | R108 now separates exact kinematics, fixed-PD inverse dynamics and full substep dynamics | Confirmed structurally; R109 must first close model identity |
| H23 | Near-zero offline collider clearance does not predict the full PhysX contact manifold and impulse | R95 isolates ordinal `2`: active foot `1539 µm` above surface with no derivative amplification; R97 reserve case passes all `11` ticks and lowers peak left-foot impulse `6440089 -> 4466405 µN·s` | Supported for the selected case only; broader contact cases remain untested |
| H24 | Complete-clip corrections are nonlocal and regress safe controls while repairing old failures | R108 lifts all 23 joint plus floating-base q/v/a rather than another scalar/local splice | Confirmed; R109 binds the exact conventions those variables require |
| H26 | Motor-frame pose/derivative summaries hide the causal physical substep | R98 localizes V11 touchdown to tick `9` substep `2` and overspeed to the immediately following pre-substep state | Confirmed; retain substep traces for every future native discriminator |
| H25 | Fresh-scene initialization is responsible | Initial state is exact within quantization in every worker | Falsified by R94; do not repeat partial-reset experiments |

## R95 differential-audit result

R95 ran from clean commit
`6491d241748cfd1a5080f7866186507f37e7c9bc`. Profile SHA-256 is
`11e5eb851f1a5cecb29f9d7796dee7e8b5f7e51104a1e9ed00b3c65285b7fbba`.
It is report-only: PhysX runs, trajectory mutations, optimizer steps and
training runs are all zero.

The four new control regressions have median V9/V7 joint-acceleration ratio
`6.8865x` and jerk ratio `21.8469x`. The three persisting/changed source
failures are higher still at `8.3367x` and `34.3599x`. Stable passing controls
also become rougher (`1.7454x` / `3.3520x` median), so amplification alone is
not sufficient; failure timing and contact state remain required discriminators.

Two cases isolate the competing mechanisms:

- ordinal `2`, `cmu05@25`, has no acceleration amplification (`1.0000x`) and
  only `1.0375x` jerk, yet its declared flat left support begins `1539 µm`
  above the collider surface; passing V7 begins at `-2 µm`. R94 impacts that
  left ankle at tick `2`. This is the contact-geometry discriminator.
- ordinal `10`, `cmu16@238`, has no impact and fails hard ROM at tick `10`.
  V9 reaches `78001200 µrad/s²` right-hip-pitch acceleration at frame `245`
  (`8.3576x` V7) and `9073080000 µrad/s³` jerk at frame `246` (`32.7269x`).
  This is the derivative/nonlocal-correction discriminator.

Ordinal `7` independently shows both mechanisms and is therefore not the first
counterfactual: its right flight foot starts only `49 µm` above ground versus
`13513 µm` in V7, while acceleration/jerk also rise `5.4956x/11.6804x`.

R95's fixed-PD quantities remain explicitly labelled proxies. They rank
experiments but are neither native torque traces nor feasibility proof.

## R96 independent offline results

Both frozen counterfactual profiles and their solver support were implemented
at clean commit `14c322ab7db44bdc9dad2e7f5b5e2e3b9476ae09`. The complete
lab suite passed `172/172`; optimizer steps, training runs and PhysX runs were
zero. The variants were evaluated independently and are not a bundled fix.

### Contact reserve: qualified offline

The contact-only profile has SHA-256
`289133123584fc1f228839731e5df405aacfb0deb6c7aca6cc692dbbe96c8a58`.
It changes only the internal normal-residual margin from `100` to `4500 µm`;
the public residual remains `5000 µm`, the collider floor remains `-2 µm`, and
all contact, ROM, velocity and controller identities remain unchanged.

The clean `cmu05` complete-clip build passes in two SQP iterations. Its
canonical/file manifest SHA-256 is
`376fedff6f8a6a327c2f99f6700d82a338e7117e310d26c36e92b154f9c691eb` /
`08b28cea1ebd69ebfc5bff88f6f17936cb98caa86bc8fb4661b1a4e2cc13fab9`.
The complete clip retains zero contact-point deletion, maximum normal residual
`510 µm`, finite tangential/normal steps `1982/426 µm`, analytic
tangential/normal steps `1970/998 µm`, collider minimum `+49 µm`, joint
velocity `2500 bp` and root vertical velocity `199770 µm/s`.

On ordinal `2`, the left active-support clearance moves from `1539` to
`496 µm` (`67.8%` reduction), maximum normal residual is `506 µm`, and the
contact modes and contact bytes are unchanged. This variant qualifies for its
single fresh-scene R97 discriminator, but that run is held until the independent
derivative variant also qualifies.

### Derivative regularity: two rejected mechanisms

An exploratory first-difference-only run changed the existing coefficient from
`0.01` to `1.0`. It passed the unchanged offline bounds, but case `10`
acceleration increased from `78001200` to `80047800 µrad/s²` (`+2.62%`) while
jerk fell only to `7404696000 µrad/s³` (`-18.39%`). It emitted no frozen
candidate/report and is non-promotable; no coefficient sweep followed.

Read-only localization then found that the right-hip-pitch correction falls
from `47446` to `505 µrad` across frames `244 -> 245` and rebounds to
`4398 µrad` at frame `246`, exactly when the right forefoot changes from
sticking to flight. This produces the R95 `-78001200 µrad/s²` acceleration at
frame `245` and `9073080000 µrad/s³` jerk at frame `246`.

The clean second-difference profile has SHA-256
`8d7a9dbc8f3d080891a8ec0ba4db38dd5ae73274888b559ff26e1303fbcd129e`.
It adds one dimensionless correction-curvature coefficient of `1.0`, equal to
the frozen correction-magnitude regularization, with no sweep or other change.
The `cmu16` complete clip and exact case `10` slice pass every unchanged
offline bound with zero mode/contact disagreement and zero changes in
unselected joint channels. Canonical/file manifest SHA-256 is
`ac7b180fcd429edc4360b4187837002472202717c39d2b7fbebf6f9490af2931` /
`0c61860517cfe6c6f87507203b4ec8251c193b514d88e9134b429149b8bd4ae0`.

It nevertheless fails the causal metric: jerk improves to
`6254388000 µrad/s³` (`-31.06%`), but acceleration worsens to
`79126200 µrad/s²` (`+1.44%`). The correction is smoother in pose space, yet
the frozen emitted-velocity stencil changes from backward difference on the
last contact frame to centered difference in flight. The objective therefore
still optimizes a proxy rather than the emitted acceleration seen by fixed PD.
Ordinal `10` is not authorized for R97 from this artifact.

The next derivative experiment must operate directly on the first difference
of the emitted hybrid-stencil velocity, using one predeclared dimensionless
coefficient and no grid search. It must strictly reduce both acceleration and
jerk, preserve byte-identical modes/unselected channels and pass all unchanged
offline limits before fresh PhysX is considered.

### Direct emitted acceleration: qualified offline

Clean commit `03175f9cb0fef2afed9f80289231496063217dec` adds the exact
sparse acceleration operator induced by the same frozen hybrid velocity
stencil used by the emitted reference. The profile SHA-256 is
`80334fd709d25054d5e6e7bb61606d26e89cec0ed685e974e4dec5e4bb43ddbf`;
its single predeclared dimensionless coefficient is `1.0`, with no sweep.
The full lab suite passes `173/173`.

The clean `cmu16` complete-clip build passes in two SQP iterations with zero
contact-point deletion. It retains maximum residual `4914 µm`, finite
tangential/normal steps `1981/984 µm`, analytic steps `1980/997 µm`, collider
minimum `+49 µm`, joint velocity `2500 bp` and root vertical velocity
`199770 µm/s`. On exact case `10`, peak emitted acceleration falls
`78001200 -> 44697600 µrad/s²` (`-42.70%`) and jerk falls
`9073080000 -> 4112424000 µrad/s³` (`-54.67%`). Contact modes, contact bytes
and unselected joint channels are byte-identical. The case artifact SHA-256 is
`4cac5ffc36c227fd71d5245d282c218cfd948ecdfe632e516ab92cac35a00b63`.

This satisfies the predeclared offline causal metric and admits only case `10`
to the two-case R97 discriminator. It is not dynamic-feasibility proof.

## R97 fresh-scene result

Commit `590bb9b5cf1b9848b37420b4b254600482b660d2` freezes the two exact
counterfactual artifacts in a hash-closed bundle. Its canonical/file SHA-256 is
`02d502877e94e1d9d5402fd29fb624c6e5545e94d1fc141934d585abe558466d` /
`69d5a29010754935fbdb8f221cf5e36000c2c4ff33ae35d1b690c07df958b58c`.
The R97 probe profile SHA-256 is
`1d91f7c63f7bf98108175ae2b62365ba5fe6e577854bc780253db82ddd272cd6`;
the run uses two independent fresh worker processes from clean commit
`3b0840110c72c74eaa1da25c2c0b8558d9724b4c`. Indexed partial reset is
`NOT_RUN`; optimizer steps and training runs are zero.

The contact-reserve case `cmu05@25` passes all `11` motor ticks. Its maximum
left-foot impulse is `4466405 µN·s`, below the unchanged hard limit and below
R94's tick-2 failure `6440089 µN·s`. Reference clearance stays
`487..505 µm`; observed support contact is present throughout. This supports
H23 for this one selected case but does not authorize the other R94 contact
failures.

The emitted-acceleration case `cmu16@238` fails at tick `9`, frame `247`, on a
new `joint_safety/joint_velocity` event in `actuator.left-ankle-roll` with
`775377 µrad/s` excess. The old right-ankle-pitch hard-ROM event occurred at
tick `10`, so the earlier termination does not prove that it would remain
closed. Reference contact modes and observed contact bytes match the R94 V9
run through the shared prefix, yet the actual left ankle-roll state diverges
strongly before the flight transition. The direct left-ankle-roll target is
nearly unchanged, making this evidence of whole-chain physical coupling rather
than a local target violation.

R97 therefore rejects the merged candidate and authorizes neither all-17,
full V19 nor training. Its explicit decision is `STOP_AND_RESEARCH`.

## R98 physical-substep result

Commit `c8ea848fe359d78b1cf264428ff35fd2f2ba0d4d` adds an explicitly
report-only probe mode. Its bounded acceptance is always `NOT_APPLICABLE`, both
profile dispositions are `STOP_AND_RESEARCH`, and merge/all-17/V19/training
authority is false regardless of the observed outcome. The V9/V11 profile
SHA-256 values are
`4183e55eb9f8e2601a4a0638ff0f8fe18ffa03a1059e0411a01b0ae55022bbd5` /
`1f8fdd1d7aee99a10ec4e6f786485060cbcbc39fe498e0de759ea89ff1568112`.
The full lab suite passes `177/177`.

The V9 worker reproduces R94 exactly: right ankle-pitch hard-ROM excess
`13193 µrad` at tick `10`. The V11 workers reproduce R97 exactly: contact case
`2` passes `11/11`, while case `10` reaches left ankle-roll excess
`775377 µrad/s` at tick `9`. This outcome identity plus complete `40`, `44`
and `36` sample inventories rejects instrumentation perturbation.

The substep order changes the causal conclusion:

1. At the initial physical state, V9/V11 left ankle-roll differs by only
   `70 µrad` position, `600 µrad/s` velocity and `18000 µN·m` requested/
   published effort. The largest action-target difference is only `2791 µrad`
   in right ankle-pitch.
2. By tick `3`, substep `0`, while the left foot is still airborne in both
   runs, V9 left ankle-roll is `+7199121 µrad/s` and V11 is
   `-7198550 µrad/s`. The phase divergence therefore precedes touchdown and
   falsifies contact as the first cause.
3. V9 first observes left-foot contact at tick `9`, substep `0`; V11 does so
   two physical substeps later at tick `9`, substep `2` (`8.33 ms`). At the
   V11 contact step, ankle-roll is `+7196128 µrad/s`, requested effort is
   `-233603940 µN·m`, but the feasible rate-limited published effort is still
   `+1616400 µN·m`.
4. Immediately after that contact response, pre-substep `3` velocity reverses
   to `-8775377 µrad/s`. The outer-limit excess is `775377 µrad/s`; the
   environment publishes zero effort because safety is already blocked.
   Effort-envelope infeasibility remains false and contact impulse
   `1395479 µN·s` is far below the hard-impact limit.

Thus V11 did not simply trade one excessive reference derivative for another.
Small reference changes move a guard/rate-limited closed-loop oscillation to a
different phase; touchdown then exposes it as a terminal event. A third local
pose/derivative smoother would optimize the wrong abstraction again.

## R99 matched passing-control contract

The missing discriminator is the exact V7/R49 `cmu16@238` trajectory: it is a
same-case fresh PASS, whereas both R98 variants fail. R99 may trace only that
existing immutable case under the identical R98 instrumentation and frozen
environment. It must reproduce the R49 PASS, capture `44/44` physical
substeps, remain report-only and change no trajectory, controller, reset or
limit. The comparison will ask which pre-contact state, solver-guard
utilization and effort-slew phase distinguish the successful control. Only
those observed margins may define the next solver locality/native-stability
anchor.

## R99 matched passing-control result

Clean commit `fad451be1ba7da62d41495ee8607a1b6d29f43f7` binds only V7/R47
ordinal `10`; profile SHA-256 is
`4a1ac6899aa22542ee8bf57b4a3d7aeff1a3c4101ea7e1ed220dd2a76dfe8425`.
The lab suite remains `177/177`. R99 reproduces the exact R49
`cmu16@238` PASS for all `11` motor ticks, captures `44/44` substeps and
leaves partial reset `NOT_RUN`, bounded acceptance `NOT_APPLICABLE`, and both
optimizer/training counts at zero. The worker file SHA-256 is
`25f2315c6a16172563a548d3a019c10aec6ca74fa697917b368f4292b178f1d6`.

R99 rejects a tempting but over-broad fix: avoiding the inner PhysX guard is
not necessary for success. V7 left ankle-roll has `18` samples at or above
`7.1 rad/s` and reaches `7.290231 rad/s`, or `9112 bp` of the unchanged outer
limit. It first contacts at tick `10`, substep `2`, with
`-7.185320 rad/s`; the contact response reverses it to `+7.290231 rad/s`,
leaving `0.709769 rad/s` outer reserve. V11 instead contacts at tick `9`,
substep `2`, with `+7.196128 rad/s` and reverses to `-8.775377 rad/s`.
The switching phase and response magnitude, not mere guard use, distinguish
the terminal event.

The matched V7/V9 reference provides a smaller natural discriminator. Their
left-ankle-roll position/target sequence is byte-identical over all `12`
frames. At the first frame only, V7 initializes that channel at
`-40080 µrad/s`, while V9 initializes at `-53280 µrad/s`; the difference is
`13200 µrad/s`, and fixed damping changes initial requested/published effort
from `1202400` to `1598400 µN·m`. By tick `3` the closed-loop phases differ.
This is a boundary-derivative locality effect, not a target-pose effect.

The right support chain shows the downstream cost. V7 right ankle-pitch has
maximum requested-to-published effort debt `24212772 µN·m`, maximum speed
`1629719 µrad/s` and minimum traced position `-602111 µrad`. V9 reaches
`136680080 µN·m`, `3337332 µrad/s` and `-710034 µrad` before its hard-ROM
termination, even though its late reference target is less negative than
V7's. Target geometry alone therefore has the wrong sign as an explanation.

## R100 one-scalar boundary discriminator

R100 may copy exact V9/R93 case `10` and replace only
`joint_velocity_urad_s[0, 5]` (`joint.left-ankle-roll`) from `-53280` with the
matched V7 value `-40080 µrad/s`. The builder must prove that the direct
12-frame joint-position target is byte-identical between V7 and V9, exactly
one scalar changes, and every other array element remains identical to V9.
The resulting case receives one fresh R98-style trace with unchanged
controller, limits, contact modes, pose trajectory and ADR-070 lifecycle.

This is a discriminator, not an admissible hand edit. If it moves the guard
phase and materially improves the native outcome, the next complete-clip
construction may add a boundary-velocity locality anchor derived from that
effect. If it does not, the scalar hypothesis is rejected and the next anchor
must cover the coupled boundary state. Both outcomes remain
`STOP_AND_RESEARCH`; no all-17, V19, optimizer or training is authorized.

## R100 one-scalar result

Clean commit `0bf2a69130b5b8f179064ac38d9ddb2e76bbab96` builds the exact
one-cell input; clean commit `a11158610d2f062b3cf4744a254b789006fd10ca`
binds its only fresh trace. Builder/trace profile SHA-256 is
`15dfb154d76213ce590dc95f5cd7176a47b6537fa27769f54cd3ee185c034763` /
`00542b76d6dad5a4fd10249e43baa27450734b47a1577ab4afce47d85a28ae02`;
the full lab suite passes `178/178`.

The scalar is causal for its local phase. Across the first `20` physical
substeps, R100 left-ankle-roll velocity has RMS distance
`177826 µrad/s` from V7 but `6542737 µrad/s` from V9. At tick `3`, substep
`0`, R100 is `-7197047 µrad/s`, matching V7's negative phase rather than
V9's `+7199121 µrad/s`. Maximum left-ankle speed stays within the unchanged
outer limit at `7199996 µrad/s`.

It is nevertheless unsafe and is rejected as a construction anchor. R100
terminates at tick `5`, frame `243`, on a new
`ground:body.right-ankle-roll` hard impact of `6092658 µN·s`; the matched V9
impulse at the same motor tick is `4521872 µN·s`. The single left-channel
edit therefore increases a remote right-foot impulse by `1570786 µN·s` and
moves termination earlier than either prior failure. Partial reset remains
`NOT_RUN`; optimizer/training remain zero.

## R101 coherent velocity-vector discriminator

R100 proves both sides of the question: boundary velocity controls phase, but
per-channel repair is nonlocal and unsafe. The smallest coherent escalation is
the complete frame-0 joint-velocity vector, not another joint scalar or a pose
smoother. At zero position error, this vector determines the initial damping
request for every actuator under the frozen controller.

R101 may copy exact V9/R93 case `10` and replace only
`joint_velocity_urad_s[0, :]` with the matched V7/R47 vector. The builder must
record the exact changed-cell inventory (`18` expected), prove all other frames
and arrays identical to V9, and prove the 12-frame direct left-ankle target
still byte-identical. One fresh physical-substep trace then asks whether
coherent initial actuator phase removes the R100 remote regression and moves
the original V9 outcome. It remains a report-only discriminator, not an
admissible hand edit.

If R101 does not improve both local phase and remote contact safety, manual
boundary-state substitution is exhausted and the roadmap must move to a
native-rollout/coupled construction or an explicit architecture decision. No
root/pose substitution, coefficient sweep or third scalar is allowed.

## R101 coherent velocity-vector result

Clean commit `da0d6959eb7dfde327437f824261da93a9a5af92` builds the exact
input and clean commit `9bbfbc1e08111fb1223df4e48b68e93b0813b5de` binds its only
fresh trace. Builder/trace profile SHA-256 is
`9214622cb85baba71e09ce88289633eb34555642286a98f98c95a7c9b1aa1b1e` /
`1c0d94663c863564a4c1f144f14df2c358ff2bba701d93e4354c4b88b1b8c786`.
The full lab suite passes `178/178`; initial state is exact, partial reset is
`NOT_RUN`, and optimizer/training remain zero.

R101 proves that coherent initialization is causal but not sufficient. Its
first requested-effort vector is exactly equal to V7 across all `23`
actuators. Across the first `20` physical substeps, its left-ankle-roll
velocity RMS distance is `267756 µrad/s` from V7 and `6461306 µrad/s` from
V9; the all-actuator velocity RMS is `606637` versus `1623104 µrad/s`.
Thus the passing local phase survives the vector replacement.

Every applied joint-position target remains byte-identical to V9 for the
complete shared `40`-substep trace. The remote response improves only relative
to the immediately rejected scalar edit: at tick `5`, right-foot impulse is
`5707869 µN·s`, below R100's `6092658` and the unchanged `6000000` limit, but
still above V9's `4521872`. At tick `10`, substep `3`, it reaches
`6006560 µN·s` and becomes a hard impact. The same motor tick also reaches
right-ankle-pitch position `-709427 µrad`, producing hard-ROM excess
`11295 µrad`. Maximum right-ankle-pitch requested-to-published effort debt is
`186668760 µN·m`, worse than V9's `136680080` and V7's `24212772`.

R101 therefore delays R100's remote failure but neither restores V7 safety nor
closes V9's original support-chain terminal. Its outcome satisfies the
predeclared stopping condition: no third scalar, boundary-vector, root or pose
hand edit is permitted.

## R102 native-rollout construction decision

The smallest architecture-preserving next mechanism is a report-only
simulator-in-the-loop construction contract over the existing fixed PD and
fresh PhysX plant. It is selected over an immediate full kinodynamic NLP
because the observed guard, effort clipping/slew and contact response are the
plant being qualified; an approximate inverse-dynamics model could reproduce
the same proxy mismatch already rejected by R96/R97. It is selected over
DiffMimic or PPO because TRAIN-4 must remain optimizer-free with respect to
learned policy weights.

R102 first builds a hash-closed audit/evaluator, not a candidate search. It
must consume the immutable V7/V9/R100/R101 traces, reproduce the phase, target
identity, effort-debt and remote-contact facts above, and define the future
rollout objective lexicographically: any required-safety event rejects the
candidate before tracking cost, every candidate uses a fresh scene and full
physical-substep evidence, and controller/limits/reset semantics never become
variables. Candidate-generation variables, knot count and search budget remain
`NOT_AUTHORIZED` until that evaluator is reproducible from a clean commit.

If the evaluator closes, the next bounded design may parameterize a complete
future joint-target sequence with low-dimensional time knots and incremental
horizon growth, following the black-box rollout shape of DynaRetarget. Frame-0
state and all frozen safety/controller identities remain fixed. A full
kinodynamic state/torque/contact-force NLP remains the fallback if black-box
rollout cannot produce a deterministic, bounded one-case construction. A
controller change requires a separate architecture decision and is not the
R102 fallback.

## R102 native-rollout audit result

Clean commit `32258edf08eaf3bf946afc19c30e64b0a8d0fe39` closes the
report-only evaluator. Its profile SHA-256 is
`ea7159cf69d530e3242aab03638415212ebeb7b2be0f369b8ae13ab5446d167c`;
the canonical/file report SHA-256 values are
`b881f8a7a70542f07c045e2451458a427a38067078520a07dc02dac4034546f6` /
`ef5f95eafe18f513abfa90803bc7ff67e8db756f327bd2b4d27f27c320b8ebee`.
The audit recomputes the four physical-substep trace hashes and their frozen
profile, worker and manifest identities rather than trusting copied summaries.

The decisive comparisons reproduce exactly. R101's first requested-effort
vector has zero disagreement with V7 across all `23` actuators, while its
command and applied targets have zero disagreement with V9 across all `40`
substeps. Its first-20 left-roll RMS distance is `267756 µrad/s` to V7 versus
`6461306 µrad/s` to V9; whole-action RMS is `606637` versus `1623104 µrad/s`.
The remote tick-5 impulse improves relative to R100 but regresses relative to
V9, and R101 still reaches required-safety failure at tick `10`.

R102 executes zero PhysX runs, candidate evaluations, trajectory mutations,
optimizer steps and training runs. It therefore closes the measurement and
priority contract, not a trajectory candidate. Its gate remains
`STOP_AND_RESEARCH`: all-17, full V19 and training remain `NOT_AUTHORIZED`.
The only admitted next action is R103 formulation of low-dimensional future
joint-target time knots, interpolation, incremental horizon, exact budget and
rollback/non-regression checks. No candidate search or new native run begins
until that tracked formulation validates from a clean commit.

## R103 target-knot formulation result

R103 closes the smallest DynaRetarget-shaped formulation without executing a
search. The selected `cmu16@238..249` case keeps offsets `0/1` byte-exact V9
and defines three scalar convex V9→V7 anchors at offsets `2/6/11`. Each
coefficient is restricted to `{0, 5000, 10000}` basis points, uses piecewise
linear time interpolation and integer ties-to-even target rounding. This
reduces `253` unconstrained future joint-target cells to three temporal
variables and a predeclared `27`-point lattice. A target is the reference
joint-position sequence itself; a separate hidden controller target is
forbidden by ADR-070.

The first emitted R103/R104 reports used the numerically correct NPZ array but
described its channels with controller-action order. Review found that
`joint_position_urad` is descriptor `dof_ordinal` order. No target value, FK
result or R104 outcome changed, but the semantic labels were not admissible.
Commits `209a8064bb578dac10fec557c0f28689ac629f24` and
`22f2e21b5d91f5b3a1c1e4da481541faea6d9ae4` therefore bind the descriptor
SHA-256 `f1f2be6a486367038f605709ebf54edb4fa6ef400fa797dd772d7590e06f3014`,
validate its exact ordered joint IDs, and reproduce both reports as v2. Only
the v2 identities in the immutable-evidence table are authoritative; the two
earlier output directories are superseded evidence and must not be cited as a
current result.

The actual nonzero anchor support is the descriptor ordinals `6,7,9,10,11`:
right hip pitch/roll, right knee and right ankle pitch/roll. R103 itself builds
zero candidate artifacts and performs zero offline evaluation, PhysX,
optimizer or training work. It authorizes only the exact offline R104 lattice
preflight.

## R104 exact offline preflight result

Clean R104 reconstructs all `27` declared targets in memory and applies the
unchanged V9 full-clip hybrid-stencil velocity, FK, CoM, active-contact,
stable-foot-box collider, ROM and root/joint-velocity audit. It emits no
candidate artifact and executes zero PhysX, native-candidate, optimizer or
training work. The all-zero control byte-reproduces V9 and is the only PASS;
all `26/26` nonzero lattice points fail:

- `24` fail finite tangential contact velocity and `24` fail analytic
  tangential contact velocity;
- `9` fail finite normal contact velocity and `3` fail analytic normal contact
  velocity;
- `6` fail the unchanged joint-velocity bound.

The closest nonzero point is `a02-00000-a06-00000-a11-05000`. Every contact,
collider, ROM, root-velocity and residual metric remains inside its frozen
bound, but source frame `245` drives descriptor joint
`joint.right-hip-pitch` to `-2024700 µrad/s`, versus the unchanged
`2000000 µrad/s` outer bound (`2531/2500 bp`). A middle-only half anchor moves
the shared right sole/heel contact by about `8166 µm` across source frames
`241→242`, versus `2000 µm`; early anchors are worse. Thus the failure is not
just a splice artifact or an unlucky large grid point: the raw V7↔V9 scalar
line leaves the exact V9 contact/velocity feasible set in different ways at
all three time regions.

R104 activates its declared failure disposition. Do not refine the scalar
grid, shrink coefficients until a numerically trivial edit passes, repair each
knot with an untracked post-pass, or launch R105 PhysX. The anchor family is
rejected and `STOP_AND_RESEARCH` remains authoritative.

## Post-R104 research decision and R105 contract

KDMR and SPARK explain the structural mismatch: dynamically feasible
retargeting varies a coupled trajectory, not an isolated joint target, and
represents contact kinematics plus state/acceleration/force variables in the
same constrained problem. A full kinodynamic NLP is therefore the principled
fallback. It is not yet the smallest discriminator for NextEngine because V9
already supplies a feasible exact-quantized kinematic point and the immediate
unknown is whether the desired V7-like target correction has any nonzero local
component that preserves those constraints.

R105 is consequently report-only. Around the exact V9 complete-clip state it
must reconstruct the same V9 SQP linearization over root translation and the
ten leg DoFs, bind the three R103 anchor basis vectors, and measure:

1. which contact, collider, ROM or velocity rows each raw basis pushes toward
   or through its bound;
2. the rank and numerical conditioning of the binding/near-binding row set;
3. the closest constraint-feasible projected direction for each basis and the
   retained anchor norm/cosine after projection;
4. whether quantization collapses every useful projected joint-target
   component to zero.

R105 may solve only deterministic local linear algebra/QPs and emit one
hash-closed report. It may not emit a candidate artifact, mutate the corpus,
run exact nonlinear candidate repair, invoke PhysX, search a coefficient grid,
change controller/limits/reset/contact semantics, or train. A reproducible
nonzero projection permits a separately profiled exact-offline construction;
projection collapse or severe loss of the target direction selects the
progressive full kinodynamic formulation, beginning with KTO/ID variables and
still no native run.

## R105 local feasible-direction result

Clean commit `3f5a32142e207afbc4eb682fc3e78991dec7e917` reconstructs the
complete V9 root-plus-ten-leg SQP Jacobian (`10413` variables, `51881` rows,
`229845` nonzeros) but permits changes only in the `130` variables at source
frames `240..249`. Exactly `600` rows depend on that support, and byte-exact V9
has zero normalized violation across them. Of those rows, `69` are within the
predeclared normalized `0.05` near-binding band; they have rank `69`, local
nullity `61` and retained-spectrum condition number `16750.23`. This confirms
that nonzero local freedom exists, but the nearby feasible space is strongly
constrained and ill-conditioned enough that raw coordinate edits are unsafe.

The complete SQP component-box proxy reports maximum zero-direction violation
`0.0172276` on rows outside the allowed support. It has no acceptance authority:
R93/R104's exact nonlinear norm-based gate accepts the same byte-exact V9, and
rows with zero coefficients in the local support cannot discriminate a local
direction. R105 therefore records this full-proxy value explicitly, drops only
rows which cannot depend on frames `240..249`, and requires zero violation on
all retained rows. This is the reason the clean implementation fix
`3f5a321` scopes the projection to locally relevant rows; no tolerance or
acceptance limit changed.

All three raw R103 bases violate at least one retained row:

- offset `2` violates `27` rows and reaches normalized violation `30.729`;
  projection retains only `510 bp` of the anchor component with cosine
  `2304 bp`, so it is rejected;
- offset `6` violates `24` rows and reaches `9.014`; projection retains only
  `569 bp` with cosine `2492 bp`, so it is rejected;
- offset `11` violates exactly one joint-velocity row by normalized `0.0257`.
  Its projection retains `9722 bp` of the anchor component with cosine
  `9872 bp`, needs at most `5688 µrad` joint and `41 µm` root correction after
  quantization, and has projected row violation below `4e-17`.

Only `anchor-offset-11` clears the predeclared usefulness thresholds. R105
therefore returns
`PERMIT_R106_EXACT_OFFLINE_PROJECTED_DIRECTION_FORMULATION_ONLY`. It constructs
three mathematical bases and solves three local projection QPs, but constructs
zero candidate targets/artifacts, runs zero exact nonlinear candidate audits,
PhysX candidates, learned/trajectory optimizer steps or training runs. R106 may
only freeze reconstruction, quantization and the later exact-offline audit
contract for the late direction. It cannot yet emit or test a candidate.

## R106 projected-direction formulation result

Clean R106 v2 at commit `a5f63fc2748f2d0e61266f4c0fe149c73a582898`
selects exactly `anchor-offset-11` and carries forward the R105 late-direction
facts without re-solving a QP. It freezes a deterministic R107 recipe: rebuild
the R105 projection from the hash-bound V7/V9/R103–R105 lineage, ties-to-even
round root and selected-leg increments, add them only to V9 source frames
`240..249`, and recompute velocities, FK, effectors and CoM through the frozen
V9 kernels. The fixed-PD target remains the candidate reference joint position;
no separate control array or desired-velocity/feed-forward path exists.

Pre-release review found one wording bug in the first R106 report: changing
frame `240` position may legitimately change a stencil-derived velocity at
frame `239`. Commit `a5f63fc` clarifies that root/joint *position inputs* are
immutable outside `240..249`, frame-238 initial position/velocity must remain
exact V9, and other dependent values may change only through the declared
stencil/FK/CoM recomputation. The first R106 output is superseded; only v2 and
the hashes in the immutable-evidence table are authoritative.

R106 performs zero projection solves, target constructions, candidate
artifacts, exact evaluations, PhysX runs, optimizer steps and training runs. It
returns `PERMIT_R107_EXACT_OFFLINE_PROJECTED_DIRECTION_AUDIT_ONLY`. R107 may
reconstruct exactly one late projected target in memory, verify the R105
continuous and quantized identities, and run the exact V9 nonlinear gate. It
must emit only metrics. Failure selects the progressive KTO→inverse-dynamics→
kinodynamic formulation; shrinking, scaling, repair and grid search are
forbidden. PASS can permit only a separate bounded native-discriminator
formulation, never PhysX or training directly.

## R107 exact projected-direction result

Clean R107 at commit `34cd81c9235f27beacd4ea5a8c8ec1383dd47715`
reconstructs the R105 late direction exactly: the solve again takes `6500`
iterations, continuous local violation is below `4e-17`, anchor component and
cosine remain `9722/9872 bp`, and ties-to-even quantization changes `7` root
cells plus `45` selected-joint cells with maxima `41 µm` and `5688 µrad`.
Position inputs outside frames `240..249`, every non-selected joint, contact
mode/source point, quaternion and yaw-velocity input remain byte-exact V9;
frame `238` position and emitted velocity also remain exact. Declared stencil
dependents change only where expected: joint velocity spans frames `239..250`.

The unchanged exact gate returns `FAIL` only for joint velocity at `2501 bp`
against the frozen `2500 bp` limit. Contact metrics remain V9 PASS
(`4913/982/1982 µm` residual/normal/tangential, `996/2000 µm` analytic), the
minimum collider height is `49 µm`, root vertical velocity is `199800 µm/s`,
and soft-ROM excess is zero. Integer quantization raises the report-only local
linear proxy violation to `0.000411892`; the exact gate, not that proxy, is
authoritative. The result is close but nonzero, so exact-zero rejects it.

R107 performs exactly one local projection QP and one in-memory exact candidate
evaluation. It emits no candidate artifact and runs zero PhysX, learned/
trajectory optimizer steps or training. Per the predeclared R106 disposition,
shrinking, scaling, rounding repair and another anchor sweep are forbidden.
The gate selects a separate progressive formulation: first quantization-aware
kinematic trajectory variables and exact stencil closure, then fixed-PD inverse
dynamics/contact-wrench feasibility, then full kinodynamic optimization only if
the cheaper stage cannot certify a safe reference. R108 may formulate that
ladder only; it cannot run a solver, build an artifact or invoke PhysX.

## R108 progressive formulation result

Clean R108 at commit `3e4457a570212f82dc289285781c50cb32ae6851`
binds R107 and the primary-source KDMR/SPARK/MIT rationale into four explicit
stages. Stage 0 is an R109 model-identity preflight. Stage 1 lifts floating-base
and all `23` joint `q/v/a` at `60 Hz`, keeps V9 contact modes and exact emitted
integer closure, and treats V7 only as a tracking/safety prior. Stage 2 solves
`240 Hz` generalized acceleration, actuator effort and scheduled contact
wrench against rigid-body dynamics plus the descriptor's fixed-PD effort,
slew, power and work limits. Stage 3 couples substep state, fixed-PD effort
state, scheduled wrenches and held `60 Hz` targets. Every stage remains subject
to the unchanged exact offline gate and later fresh-scene authority.

The descriptor has the basic expected inventory (`24` bodies, `23` joints,
`23` actuators, `19` colliders, `60/240 Hz`), but that count is explicitly not
model-identity evidence. Before equations or a solve exist, R109 must bind body
mass/CoM/inertia and principal frames; joint axes/frames/signs; actuator gains,
clips and effort slew; collision exclusions/material/friction/gravity; target
hold/integration cadence; and root/contact-wrench conventions to the exact
descriptor, native PhysX and derived-USD lineage.

R108 runs zero identity preflights, KTO/inverse-dynamics/kinodynamic solves,
candidate constructions/artifacts, PhysX, optimizer steps and training. It
returns `PERMIT_R109_DYNAMICS_MODEL_IDENTITY_PREFLIGHT_ONLY`; KTO execution is
still unauthorized. A missing or contradictory R109 mapping stops with
`STOP_INVALID_MODEL_LINEAGE`, rather than filling the gap with an assumed
constant or imported simulator convention.

## R109 dynamics-model identity result

Clean R109 at commit `8c4b7036b419bea5dc555131b28ec9588441376f`
hash-binds R108, the V2 descriptor, byte-exact derived USD, native PhysX 5.9.0
sources, the reference controller and pinned Isaac Lab `v2.3.2`. Body mass,
CoM, full/principal inertia, joint hierarchy/frames/axes, collider geometry and
exclusions, `60/240 Hz` cadence, gravity coordinate transform and the fixed-PD
target/effort/slew/power/work path close. The stored USD exactly equals a fresh
translation of the descriptor (`24` bodies, `23` joints, `19` colliders).

The required material identity does not close. The descriptor names `17`
colliders as `physics-material.humanoid-body.v1` and two soles separately, but
contains no `PhysicsMaterialDescriptorV1` coefficients or combine profile; the
derived USD contains no material binding. The native bridge therefore applies
one hard-coded `0.8/0.7/0.0` material to every shape, while Isaac's unoverridden
default is `0.5/0.5/0.0` with average combination. This is not the allowed GPU
floating-point tolerance boundary: SPEC-26 explicitly requires exact material
descriptors/combine semantics and forbids a backend default from substituting
them. Solver iteration/scene-flag differences remain separately declared
non-byte-exact mirror facts, with final authority still canonical CPU PhysX.

No engine-owned contract also defines the future scheduled contact-wrench
frame, component order or application-point convention required by R108.
R109 therefore returns `FAIL / STOP_INVALID_MODEL_LINEAGE`; it does not invent
either coefficients or wrench coordinates. Canonical/file/profile SHA-256 is
`2867aecd144d7996d3bf5bd0b6498dc1a5d480f7b8060106a97c5c6fc07da784` /
`97f149b12f5f4a6694da04298df774f33fd4854e93b54f3096d8882f2c1efc85` /
`961926664ca8ff08a4c384180092dcbb7cb6591880bb8501148fef04d1e65ed0`.
Exactly one report-only identity preflight and zero solver, candidate, PhysX,
optimizer or training work occurred. The next action is bounded research and
repair formulation for canonical material/combine and wrench ownership; KTO
formulation/execution remains unauthorized until that lineage closes.

## R110 material-lineage and point-force formulation result

Clean R110 v2 at commit `965c256b8b18e82bca0e8bfee73e8f4438bb12e6`
turns the three R109 gaps into a bounded implementation contract. The selected
body, sole and new ground material IDs each receive exact Q16 static/dynamic/
restitution values `52429/45875/0`, corresponding to the nearest ties-to-even
successors of the old native `0.8/0.7/0.0` literals. Rolling and spinning
friction plus surface velocity are all exact zero. One
`ArithmeticMeanTiesToEven` rule is frozen for every scalar coefficient and
surface velocity uses canonical participant order. The resulting material/
combine table root is
`70486c8405cfc1c1f2a937fb33ddf94a3ef0a5a647ebcc17eb28f9b7121113dc`.

The review caught a deeper pre-implementation mismatch: Accepted SPEC-26
contains rolling friction, spinning friction and surface velocity, while the
implemented `PhysicsMaterialDescriptorV1` canonical record omits all three and
the world descriptor omits the combine-profile hash. Same-version mutation is
forbidden, so R110 v2 requires a `PhysicsMaterialDescriptorV2` successor plus
`PhysicsMaterialCombineProfileV1`; nonzero extended fields fail closed for this
generation. The first R110 report at commit `3ba6928` covered only the
incomplete implemented V1 fields and is explicitly superseded. Its canonical/
file/profile SHA-256 is
`6a8aa992d517265f9e27fcdd5daad36d97a6bed34f91fc96bddd8ee143dde185` /
`25bdbd57bde4329bbba41d33090b7dde650663aed7cafe48818c3df234c57653` /
`04e216ab4226033708bbf9f4e3a123f76a716747ff6f419a893e241d6020b3d2`
and must not be used as authority.

The backend mapping is intentionally narrow and fail-closed. Native may
deduplicate the three coefficient-identical descriptors into one cached
`PxMaterial`, but only after resolving every material ID, checking complete
equality, converting from Q16 and explicitly retaining zero torsional patch
radii/surface velocity; hard-coded literals are removed. Any future unequal
descriptor or nonzero rolling/spinning/surface-velocity value is rejected
before adapter construction until a successor ABI supports it. Derived USD
must author physics-purpose material bindings and exact Next Engine metadata;
Isaac ground/articulation construction may not fall back to its defaults.

This follows [SPEC-26](../architecture/26-physics-world-collision-constraints-queries-and-canonical-snapshots.md),
the pinned PhysX `5.9.0` headers, NVIDIA's
[USD/PhysX material binding and combine documentation](https://nvidia-omniverse.github.io/PhysX/ovphysx/latest/simulation_setup/rigid_bodies.html)
and OpenUSD's [physics-material schema](https://openusd.org/22.08/api/usd_physics_page_front.html).
The pinned SDK manifest/PxMaterial/PxShape/PxContactModify header SHA-256 values
are `e3f774e1fa6aface0060f56a534d26056b2fd7d064d16a75e80d133b00ceed42`,
`803a88f43e1979163cc0809537d346fa679ca5ab3b583efdbcd3a7576b2fa9de`,
`0b1084a0e80f84850d7c2697f32eb7647810744c18c1282b1388d37e7b14a660`
and `d86239214812cb22511b9eec681628eb2eb3f5b158675d285eb0104d7092c135`.

R110 also removes the ambiguous six-dimensional “wrench” from the private
future solver. At each `240 Hz` interval it permits only three force components
`[normal, right, forward]` at the frozen V9 points ordered left heel, left
forefoot, right heel, right forefoot. Engine-world axes are normal `+Y`, right
`+X`, forward `+Z`; the generalized contribution is `J_contact(q)^T lambda`.
Inactive points have exact zero force, active sticking points use the exact
static-friction cone, and no independent torque, yaw moment, center of pressure
or point relocation exists. This is the standard point-contact construction
described in the [MIT multibody notes](https://underactuated.mit.edu/multibody.html),
but its exact axes/order/application identities are owned by the R110 profile.

R110 v2 returns
`PERMIT_R111_CANONICAL_MATERIAL_LINEAGE_IMPLEMENTATION_ONLY`. R111 may close
the architecture/schema version, contracts/compiler/mirror and native
equal-material boundary with static/golden tests. It may not run a PhysX scene.
USD/Isaac implementation is separately gated as R112 and a clean model-identity
recheck as R113. KTO, inverse dynamics, kinodynamics, candidate construction,
all-17 and training remain unauthorized. R110 itself performs one research
report and zero runtime changes, solver runs, PhysX runs, optimizer steps or
training runs.

## R111 canonical material-lineage implementation result

R111 separates the new authority from every immutable legacy identity. Accepted
[ADR-071](../architecture/adr/071-canonical-physics-material-lineage.md) adds
`PhysicsMaterialDescriptorV2`, `PhysicsMaterialCombineProfileV1`,
`CompiledBodySchemaV3` and mirror V2; V1 material bytes, V2 compiled bytes and
mirror V1 remain historical and unchanged. The body-schema hash therefore stays
`e2460e7dc4af93538ae4b0b68a9e1bf74b2b7990161e08e441d58687e953c43d`,
while the new compiled descriptor and material-lineage hashes are
`6751853a812f549866f1db9d3662d8115b18db9b6d73beabd7221bb9f972f027`
and `2d13e197f766e6a24090edf396dfc2fb6cbbf4c578ea9868dffa06ab7adab751`.
The generated mirror V2 SHA-256 is
`7928fe23affaf9dd16a0c82db1d7da85e61af2ad0f071f9423c6df1121ba50a3`;
the tracked mirror V1 stays byte-exact at
`f1f2be6a486367038f605709ebf54edb4fa6ef400fa797dd772d7590e06f3014`.

Native Bridge ABI 4 removes the ambient hard-coded material from world
construction and requires an explicit 72-byte profile before any scene can be
created. The current generation accepts only the coefficient-identical Q16
projection, explicitly sets arithmetic-mean friction/restitution and zero
torsional patch radii, and fails closed on unequal descriptors or nonzero
rolling/spinning/surface-velocity fields. Legacy Stage 0 remains behaviorally
stable because it now requests its named frozen profile explicitly instead of
depending on a backend default. The pinned PhysX 5.9.0 ABI-4 SDK profile hash
is `f259d3da157cc6120b378b53ee14c10805be89698242b03d7417f699ca711c3b`;
the actual bridge compiled and linked, but no PhysX scene was executed.

Architecture, implementation and audit commits are
`14eb0b2cac25173dc137b64201732fae66ad5a27`,
`6b87f78c1211ca354416c091789522aecfabd2c4` and
`080a5b8a93fa283eb53b14f9f066661b947374e4`. The clean external R111 report
passes all eight frozen validation commands and has canonical/file/profile
SHA-256
`eafc8fc7f5bc64706b53c313cff143e0c0f8bd7684371e714d94bdef3e86f058` /
`c8b5663c86fdf89ba4f8729fdccb2860328fd5e7b6144388ab892d5d4bc97bd7` /
`a5cb5a3330eddefaeff33639e79c885ecbace7dfb8bddbf86de32f11b04bf44e`.
It records zero scene, solver, candidate, optimizer and training work and
returns
`PERMIT_R112_DERIVED_USD_MATERIAL_LINEAGE_IMPLEMENTATION_ONLY`. R112 is
restricted to generated physics-material prims/bindings, exact lineage
metadata and explicit Isaac ground consumption; R113 identity recheck, KTO and
all runtime/training work remain separately blocked.

## R112 derived-USD and Isaac material-lineage result

R112 completes the separately gated derived half without reinterpreting any
legacy artifact. Mirror V2 is now mandatory for the successor translator, and
strict validation rejects unknown fields, malformed integer encodings, changed
material rows, combine rules or assignment counts. Generated output lives under
the body-schema plus compiled-descriptor identity. The humanoid USD contains
two physics-material prims and exact `17` body plus `2` sole physics-purpose
bindings; the separate ground USD contains one ground-material prim and one
binding. Both encode the Q16-derived `0.800003052/0.699996948/0` coefficients,
explicit PhysX arithmetic-mean combine modes and exact zero-extension metadata.

Isaac construction now validates the mirror, humanoid USD, ground USD and
translation manifest as one immutable bundle before scene setup. It consumes
the derived ground USD explicitly; `GroundPlaneCfg`, `RigidBodyMaterialCfg` and
ambient material defaults are unreachable on this path. A byte mismatch at an
existing derived identity fails closed. The body-schema, compiled-descriptor
and material-lineage hashes remain
`e2460e7dc4af93538ae4b0b68a9e1bf74b2b7990161e08e441d58687e953c43d`,
`6751853a812f549866f1db9d3662d8115b18db9b6d73beabd7221bb9f972f027`
and `2d13e197f766e6a24090edf396dfc2fb6cbbf4c578ea9868dffa06ab7adab751`.
Humanoid/ground/translation-manifest file SHA-256 is
`5ea8a3b9b4e745461fd02bda7823cb1ffb2a1f372c16a9987f385e2697493834` /
`e82398add4570bc0696929081b4602ca16c8e3cf434a73b61c7f780ca6e9d033` /
`28fc9d97d42e071773057a159a0404d658d145f89eed90ee73b0571e8bfb6f6b`.

Implementation and audit-tooling commits are
`61e02484290be11059433acd8c5da434d7a0fc93` and
`12fb82fe0925f1707f90fde84a23eb775d4c5268`. The first audit attempt exposed
only the workspace `1000`-line source boundary in the R111 PhysX adapter; the
behavior-preserving module split at `9cd3f79` reduced `lib.rs` to `926` lines,
after which focused adapter tests and strict clippy passed. The clean external
R112 report at repository commit `5d168b1cac1a06220ad678804f1b11ecb0285ba5`
passes Ruff, all `222` lab tests, motor tests and full `host-check`. Its
canonical/file/profile SHA-256 is
`dfb3bd892b04023054ce947127743e7ada78b40000a93e22887101c544c493f2` /
`a615359b7855ca270dacced899960d71ab4a43c8aab69403fd154a55a0067778` /
`c98c383117f03b5bb594855c831f2aa3a31453ac94b6f4fa2dc483015a3b7f0a`.
It records zero model-identity preflights, scene, solve, candidate, optimizer
and training work and returns
`PERMIT_R113_CLEAN_DYNAMICS_MODEL_IDENTITY_PREFLIGHT_ONLY`.

## R113 clean dynamics-model identity result

R113 performs the independent report-only recheck required by R109/R112. It
binds the exact R112 mirror V2, humanoid USD, ground USD and translation
manifest; the exact R110 v2 solver-private point-force report; current
structural/compiler/native/controller sources; and pinned clean Isaac Lab
`v2.3.2` at `37ddf626871758333d6ed89cf64ad702aef127d0`. It constructs no
scene or dynamics model and solves no equation.

All eight R108 identity groups close together. The descriptor still has
`24` bodies, `23` joints, `23` actuators, `19` colliders, `10` non-colliding
carriers, `30` exclusions and mass `75337000 µkg`; its compiled/material hashes
are the exact R111 successors. The three Q16 material rows and combine profile
match native ABI 4 and all `19+1` derived bindings, while both native ambient
`0.8/0.7` literals and Isaac `0.5/0.5` defaults are unreachable. R110's four
ordered `[normal,right,forward]` point forces, application points, cadence and
no-independent-moment rule close the former wrench-ownership gap.

R113 deliberately preserves three nonblocking mirror divergences: native/Isaac
position iterations `16/8`, scene determinism/contact-cache flags, and flat-box
versus derived-plane ground representation. SPEC-35 correspondence remains
tolerance-based and final authority remains canonical CPU PhysX, so the report
makes no runtime numerical-equivalence, PhysX-behavior or correspondence claim.
The R109 blocking set is now empty, but dynamic feasibility is still untested.

The clean report at commit `2bec28f5de9a97f8e49198e58cc55a1d7dd26ebd`
passes Ruff, all `226` lab tests, motor tests and full `host-check`. Its
canonical/file/profile SHA-256 is
`3ac92ae2ca508234a52d77f0414ad5557f1164028e51a3938cc045ac4c5147cf` /
`de584af485e789a19e457708cf154193c5d870dd1deb8ee29a61b6abddb6541c` /
`fa52cf18be25144893fb1d4da57bbae2fc2056d00d13e4bac012276ccc8cdf5d`.
It records one model-identity preflight and zero solver, KTO, inverse-dynamics,
kinodynamic, candidate, PhysX, optimizer or training work. The only successor
gate is `PERMIT_SEPARATE_BOUNDED_KTO_EXECUTION_FORMULATION_ONLY`: R114 may
freeze a bounded KTO execution contract, but may not execute it.

## R114 quantization-aware KTO execution formulation result

Clean R114 revision 2 at commit `8b0ffc998dbed63ea37f07ca79d7dbe21f8dfc8f`
turns the R108 stage-1 sketch into one executable, fail-closed contract without
running a solver. It independently binds the exact R113 PASS, the R108 ladder,
the current material-complete descriptor, the byte-identical legacy kinematic
projection and the V9/R93 complete `cmu16-walk-nominal-b` artifact.

The source roles are deliberately asymmetric. V9/R93 is the only admissible
`801`-knot initialization and whole-horizon reference. The matched V7/R47
`cmu16@238..249` PASS case is only a twelve-frame joint-position tracking and
safety prior. It differs from V9 in `61` cells with exact L1/squared distances
`527631 µrad` / `17142428727 µrad²`. It cannot be a splice or complete
candidate: the later V7/R57 complete `cmu16` trajectory itself fails the
complete-clip contact gate. This distinction prevents a locally passing V7
control from silently acquiring full-trajectory authority.

The frozen decision vector has `87` scalars at each of `801` knots, or `69687`
total: floating-base position/SO(3) tangent, linear/angular velocity and
acceleration plus all `23` joint q/v/a channels. Velocity and acceleration are
joined to configuration by the exact `60 Hz` hybrid stencil. Its byte hash is
`00f8e3acc9fb19131cfa290109f391b0f0ed05b5c2f44bcec66c73d3e20d63ca`;
the fixed V9 schedule selects `10` forward, `10` backward and `781` centered
rows with entry precedence. Both endpoints remain exact V9 after emission.

Acceptance has no continuous-solver shortcut. Each proposed iterate is emitted
with ties-to-even root, quaternion and joint quantization; dependent velocities,
FK, CoM and every collider sample are recomputed; and the unchanged V9 contact,
collider `-2 µm`, root vertical `200060 µm/s`, joint `2500 bp` and soft-ROM
limits must all PASS. Exact V7 progress additionally requires a positive
integer directional dot, a strict squared-distance decrease from
`17142428727`, and at least one changed joint cell. Rounding repair, margins,
coefficient search, schedule/point changes and post-emission mutation are
forbidden.

The first clean formulation report at commit `77be95d` left the emitted
`root_yaw_urad` branch implicit. A solver-free implementation audit exposed the
problem: V9 yaw is continuous/unwrapped, while direct frozen XZY decomposition
wraps at `±pi` and falsely reports a large tangential velocity. Revision 1,
canonical/file/profile `53f77c…` / `2f5c6c…` / `ec0a98…`, is therefore marked
`SUPERSEDED_BEFORE_KTO_EXECUTION_ROOT_YAW_BRANCH_WAS_UNDERDEFINED`; no OSQP or
KTO solve had run. Revision 2 freezes a unique rule: decompose emitted and
source-V9 quaternions, wrap their angular difference to `[-pi,pi]`, add it to
the continuous source-V9 yaw, then ties-to-even quantize. This is part of
emission semantics and cannot be used as a later repair.

The same solver-free real-data preflight constructs `69687` variables,
`131180` constraints and `426210` sparse nonzeros with no lower-bound/upper-
bound contradiction. At zero increment, canonical matrix-to-quaternion
normalization changes `17` internal cells by exactly one Q1.30 LSB, leaves both
endpoints byte-exact, and the corrected audit reproduces V9 PASS metrics:
contact residual/normal/tangential `4913/982/1982 µm`, analytic normal/
tangential `996/2000 µm`, collider `+49 µm`, root velocity `199800 µm/s`,
joint velocity `2500 bp`, and zero descriptor/effective ROM excess. Only strict
nonzero V7 tracking progress fails, as required for the zero control. This
preflight invokes neither OSQP nor a scene and has no candidate authority.

R115 is limited to one single-threaded CPU SQP process, one solve, twelve major
iterations/QPs, at most `72` exact emission audits, four hours and `16 GiB`.
It uses the already pinned NumPy/SciPy/OSQP versions and frozen finite-difference,
step, line-search and OSQP settings; no restart or manual intervention exists.
Exact PASS may retain one external transient solver-private q/v/a warm-start
cache for the separately gated inverse-dynamics stages. That cache has no
candidate, corpus, controller, runtime or acceptance authority and is deleted
on failure or after downstream use. No candidate artifact is authorized.

The clean revision-2 report passes Ruff, all `235` lab tests, motor tests and full
`host-check`. Its canonical/file/profile SHA-256 is
`7a735320509a303f9feacba79087f2042d451d526b57585d4fe4041603b46ae4` /
`2666a275180b222c014eea91051ff4d3ebdb16a5740eb742ed2cca3a83b3d958` /
`8e26b84de2a25e07d19cd93400bc8c04a6bc4840fb9a4d8eb6ab88237db59a43`.
It records one formulation and zero solver, KTO, candidate, cache, PhysX,
learned-optimizer or training work. Its only successor gate is
`PERMIT_R115_SINGLE_BOUNDED_QUANTIZATION_AWARE_KTO_EXECUTION_ONLY`.

## R115 single bounded KTO result

R115 ran once from clean commit `e34f463263b54f2ea7612f4cd4a452b99667c1be`
under the exact deterministic environment and R114 revision-2 hashes. OSQP
solved the first `69687`-variable, `131180`-row, `426210`-nonzero QP in `2300`
iterations; its primal/dual residuals were `1.7951e-5 / 2.3869e-6`. The full
KTO stage took `138.009 s`, peaked at `1738563584` bytes and remained inside
the four-hour/16-GiB contract. This is the one consumed KTO solve; it cannot be
repeated.

All six predeclared emitted fractions make strict V7 progress and preserve
endpoint identity. The full step additionally fails contact, collider, root
vertical velocity and joint velocity; `1/2` fails contact and collider. The
important isolation begins at `1/4`: fractions `1/4`, `1/8`, `1/16` and
`1/32` pass collider, ROM, root/joint velocity, endpoint and V7-progress gates,
and their finite contact residual/step metrics pass. Their sole remaining
failure is analytic tangential contact velocity: `5498`, `3673`, `2791` and
`2368 µm/frame` against the unchanged `2000` limit. At `1/32`, collider is
`+47 µm`, finite tangent/normal are `1982/982 µm`, root velocity is
`199800 µm/s`, joint velocity is `2500 bp`, and V7 squared-distance improves
by `615 bp`; analytic tangent still exceeds the limit by `368 µm/frame`.

R115 therefore records `FAIL / no_exact_progress_step / STOP_AND_RESEARCH`.
Canonical/file/profile SHA-256 is
`b7baa0f4337597c1c61748255535d1102bbff35d0ce560a8a661ed6be7685433` /
`ed89c2733a376f5f50694fc01029abcf9791f2a186054196461ce2a787e8caa8` /
`427958340debb62a3dfc7b9309411184c8e922965a5cef694163cae61d0b9e75`.
All five validations pass. Exactly one KTO/QP solve and six in-memory emitted
audits occurred; warm-start caches, candidate artifacts, inverse-dynamics/
kinodynamic solves, PhysX scenes, optimizer steps and training runs are zero.
The conditional inverse-dynamics formulation branch is not authorized.

## R115-RC1 analytic-contact linearization research result

The fail shape does not support an intrinsic-feasibility conclusion. Analytic
tangent falls almost affinely with line-search fraction (`R²=0.9999147`), yet
remains the only failed gate across the four smallest steps. A clean report-only
audit at commit `20b52c8608507b91cddc96ce013f7d7379a7f149` inspected the
frozen sparse system and exact kernel without invoking OSQP or reconstructing
the discarded R115 direction.

The QP's `3723` `contact_analytic_velocity` rows contain `33509` nonzeros, all
in velocity columns: configuration and acceleration nonzero counts are both
zero. That model constrains `J(q0) Δv` but omits the configuration part of the
linearization of point velocity. At the byte-exact V9 hotspot
`frame 328 / left forefoot`, symmetric differences of the exact frozen kernel
find nonzero tangential configuration sensitivity for all three root-orientation
and all six same-side leg variables. The maximum is
`1105327.0702 µm/s/rad` at left hip yaw. Thus a configuration-changing QP
direction can satisfy the implemented analytic rows while violating the exact
`J(q)v`-like emitted diagnostic at first order.

The audit also finds a smaller baseline convention mismatch. The continuous
R115 sole-Jacobian model and the frozen emitted contact kernel share the same
`2000 µm/frame` maximum and hotspot, but differ in `3168` active components by
more than `1 µm/s`; maximum/RMS component differences are
`3032.888 / 108.783 µm/s`. This comparison is diagnostic: it combines the
solver's full spatial/angular velocity convention with the exact kernel's
emitted integer, yaw-plus-leg convention. It is not itself an acceptance fail,
but proves that the two functions were not identity-bound.

This diagnosis matches primary methods rather than inventing a new tolerance.
[Pinocchio's frame-derivative API](https://docs.ros.org/en/rolling/p/pinocchio/generated/function_namespacepinocchio_1a0d05b1c07362deab78a8855467b354ef.html)
returns frame-velocity partials with respect to both `q` and `v` separately.
[MIT's trajectory-optimization notes](https://underactuated.mit.edu/trajopt.html)
require the full chain rule for constraint gradients. Fletcher and Leyffer's
[filter-SQP method](https://doi.org/10.1007/s101070100244) and
[TrajOpt](https://doi.org/10.1177/0278364914528132) further support making
nonlinear iterate acceptance/restoration explicit across repeated
linearizations. The bounded inference for NextEngine is narrower: R115 proves
the frozen implementation direction failed, while R115-RC1 confirms a
linearization-identity gap; neither proves KTO feasibility or infeasibility.

R115-RC1 canonical/file/profile SHA-256 is
`5edfe0613e2e4f9327cd1bfb6922c96e84ce8056144b05f9617787de0f883475` /
`74f6b7f7c13565618a2972d83b568d411ae8f4598b76699cd176b328f4a42f51` /
`7e4186d0e44049102ab1d61e0cf49f51f3682e88557f9c8c2dd2d92c5c53c3f4`.
Five validations pass, including `239/239` lab tests and full `host-check`.
The audit records one research audit and zero QP/KTO/ID/kinodynamic solves,
caches, candidates, PhysX scenes, optimizer steps or training runs. It permits
only one report-only R117 KTO linearization-repair formulation. R117 must bind
the exact contact function, complete q/v derivative, modeled-versus-exact row
diagnostics and an explicit nonlinear iterate acceptance/restoration rule; it
must not run a solver.

## R117 exact-kernel linearization-repair formulation result

Clean report-only R117 at commit
`41b72395d7a7f6c5c4eb2b8f19bcb86dbc594321` closes the authorized formulation
step without invoking OSQP or constructing a trajectory. It inherits R114
revision 2's `801` knots, `69687` q/v/a scalars, hybrid stencil, endpoints,
ROM/collider/finite-contact/velocity limits, V7 integer progress gate,
quantization, objective and resource ceilings unchanged. Controller gains,
effort caps, the right-ankle `6000000 µN·s` impulse limit, fresh-scene
authority and report-only partial reset are unchanged.

The repaired continuous constraint is defined as the physical-unit extension
of the emitted exact kernel rather than a generic frame Jacobian. It uses
engine-world root linear velocity, the same world-Y one-sided `1e-4 rad` root
lever with continuous-V9-lifted yaw stencil velocity, and only the six
same-side leg-joint one-sided levers. The derivative covers the whole function:
configuration-dependent levers, the yaw lift's neighboring orientation knots,
explicit root-linear velocity and same-side joint velocities. The tangential
row is `vx² + vz² <= 0.12²`, exactly corresponding to the `2000 µm/frame`
norm at `60 Hz`; R115's component box is explicitly rejected.

R117 also freezes globalization before any implementation. Each major
iteration must start from an emitted/rederived anchor and audit the six fixed
fractions. Exact PASS terminates first. At most one intermediate may bridge
from strict V7 progress when every exact gate except analytic tangent passes;
then each restoration anchor must preserve that progress and every other gate
while strictly decreasing the lexicographic pair of maximum/summed normalized
exact tangent excess. No acceptable fraction means `STOP_AND_RESEARCH`, not a
restart or relaxation. Metric-only replay of this policy selects R115's
`1/32` summary (`368 µm` excess, `615 bp` progress), but reconstructs neither
its discarded direction nor any arrays.

R117 canonical/file/profile SHA-256 is
`b0a9f07012e0f43c660019df8a1f31e7368f6130312231cf6c2bddb0f59fb7c7` /
`f32f31d895ed26f1f7ec842b2125ce0329a0d5802435aaebc5973d198dc06ed4` /
`eaabc22958324504756d95239ceefec3597523cf01e2d3c8633d6942db23da55`.
Five validations pass, including `243/243` lab tests, `56/56` motor tests and
full `host-check`. Formulation reports are one; QP/KTO/ID/kinodynamic solves,
caches, candidates, PhysX scenes, optimizer steps and training runs are zero.

R117 permits only report-only R118 implementation and deterministic numerical
conformance. R118 must establish exact-kernel baseline identity within
`1 µm/s/component`, compare the implementation Jacobian with an independently
implemented symmetric whole-function reference within `5 µm/s/unit + 1e-4`
relative tolerance, cover the frame-328 left-forefoot hotspot plus both-side
entry/exit/centered anchors, prove the required q/neighbor-yaw and explicit v
dependencies, and preserve the norm-squared tangent row. R118 may not import or
run OSQP, execute KTO, emit a candidate, or directly authorize execution; a
later execution would first require a separate report-only formulation.

## R118 exact-kernel full-q/v conformance result

Clean report-only R118 at commit
`990f1e9045d045ba6586962e751980a0a6c8649d` implements and audits the R117
continuous contact function without importing OSQP or executing a solver. The
zero perturbation preserves the emitted V9 yaw-rate sample exactly, while the
frozen hybrid stencil acts only on continuous nearest-branch yaw deltas from
configuration perturbations. This is necessary because the stored V9 yaw-rate
samples are not generally equal to a freshly reconstructed contact-transition
stencil, and replacing them would break exact-kernel identity.

Across all `1241` active point-frames (`3723` vector components), the maximum
absolute difference from the emitted one-sided kernel is
`0.0000110342 µm/s`; no component exceeds the frozen `1 µm/s` tolerance. The
fixed hotspot plus entry/exit/centered anchors on both sides all pass. Their
full variable inventories contain `61` or `64` columns, independently nonzero
configuration counts are `10` or `11`, all current/root-neighbor yaw
dependencies are present, all three root-linear and six same-side joint-
velocity dependencies are present, and all `20` unrelated velocity columns
per anchor remain zero. The separately implemented whole-function symmetric
reference has zero violating Jacobian components under
`5 µm/s/unit + 1e-4` relative tolerance. The tangent row remains
`vx² + vz² <= 0.12²`; its worst scalar derivative disagreement is
`2.6353e-8 m²/s²/unit` against the `5e-6` absolute tolerance.

R118 canonical/file/profile SHA-256 is
`23d9d556d8be630f7f2e9fe9907f394b74186ef545d0c46f7484f0efc1df45ca` /
`f4d186a476c6bb73f1f001ac6e991088b7563d2aeeca49f63db6e872afb7e861` /
`d97340714757c1ead27b9f571332f926690104a89fed8a2d3a6ef357c9de88f3`.
The conformance module/tool SHA-256 is
`1eb2016c467f3505b2a0834c8d3c83b142b963f317644ceaa9d106ae3a269ede` /
`0b22d1360985289ced647d60703f7cea9a6be8e7d58d0871ba8db2053a9869d9`.
All six validations pass, including solver-free import, `248/248` lab tests,
`56/56` motor tests and full `host-check`. QP/KTO/ID/kinodynamic solves,
caches, candidates, PhysX scenes, optimizer steps and training runs are zero.

R118 authorizes only one separate report-only R119 repaired-KTO execution
formulation. R119 may bind the repaired implementation, exact nonlinear
iteration/quantization policy, diagnostic rows, resource ceilings and a future
single-execution gate. It may not import or run a solver, reconstruct R115's
discarded direction, emit a candidate, run PhysX or begin training.

## R119 repaired-KTO execution formulation result

Clean report-only R119 at commit
`322896b1c67376337eacfb1eab9335ef66b12776` closes the execution contract
without loading OSQP or either prior solver/execution module. R114 revision 2
remains hash-authoritative for all `801` knots, `69687` q/v/a variables,
stencils, objectives, trust bounds, public limits, quantization, controller and
resource ceilings. Only the four R117 repair-boundary items change: analytic
contact function, complete q/v derivative and scalar chain rule, nonlinear
bridge/restoration globalization, and contact-row diagnostics.

For every one of the `1241` active point-frames, R119 replaces R115's three
velocity-component rows with one two-sided normal row and one upper-bounded
`vx² + vz²` row. Thus `3723` retired rows become `2482` repaired rows and the
otherwise unchanged matrix has `129939` rows. Projection of the seven R118
anchors is PASS: every normal row has `15` nonzero coefficients; tangent rows
have `18..20`, including `10..12` configuration and exactly `8` explicit
velocity coefficients. R119 also freezes the R118 finite-difference probes,
physical row scales, coefficient-elision threshold and global q/v column map.

Every major iteration is local to the current accepted emitted-and-rederived
integer anchor. Its zero perturbation uses that anchor's actual root-linear,
root-yaw and joint velocities; the yaw stencil acts only on local continuous
orientation deltas. Before every QP setup, R120 must repeat all-active
function identity and the seven derivative guards around that anchor. An
accepted fraction is ties-to-even quantized, yaw-lifted and fully rederived
before another linearization; a rejected float state is discarded. The R115
`1/32` result remains a metric-only policy discriminator: no R115 array,
direction, state or cache is an R120 input.

R119 permits one new single-threaded R120 process from byte-exact V9, not an
R115 retry. It may solve at most one QP in each of `12` major iterations and
audit the six fixed fractions, for at most `12` QPs and `72` exact emissions
inside one KTO execution, bounded by four hours and `16 GiB`. Exact PASS stops
first; the single bridge and strict exact tangent funnel govern intermediate
anchors; invalid evidence, no eligible fraction or resource exhaustion stops
without restart or tuning. Candidate arrays, PhysX and training remain
forbidden.

R119 canonical/file/profile SHA-256 is
`ac38e3f0a9dfb5e900373bc5a4908168a49f3c3b799fb431bd6e5c1306a4dd1b` /
`e233f9dd60ba8056e55b132167e5dbd6e952781fb15bece56cbb23e62b0c1882` /
`3b48c618cffc5494601a40ac15b04df0ffba2a1b7d6aadbdfd4309b6c167dcd1`.
The formulation module/tool SHA-256 is
`8babe0e250fd7beaacd9f39febc2c8c8120ddebf73b817a5962c8e79c25d1755` /
`90f8c30af24485bea3788a2e86da3338f8c4568f4c87c63f8f55017e5c1aca1e`.
All six validations pass, including solver-free import, `253/253` lab tests,
`56/56` motor tests and full `host-check`; every R119 execution/work counter
is zero.

## R120 repaired-KTO execution result

The sole clean R120 process at commit
`39708da5036a2f5a2f3bd4f1b4d2f840efe17088` starts from byte-exact V9 and
passes every pre-solver discriminator. The repaired sparse matrix is
`129939 × 69687` with `434866` nonzeros: all `1241` normal and `1241`
tangent norm-squared rows are present, the retired `3723` component boxes are
absent, and all-active function identity plus all seven R118 Jacobian anchors
pass before the only QP setup.

That QP reaches `solved` in `3075` iterations. Its six immutable emitted
fractions are audited exactly. Fractions `1` through `1/16` remain diagnostic
failures; `1/32` is a direct exact `PASS`, not a bridge. The accepted result
has contact residual `4913 µm`, finite tangent/normal steps `1982/982 µm`,
analytic tangent/normal steps `2000/994 µm`, collider clearance `+47 µm`,
root vertical velocity `199800 µm/s`, joint velocity `2500 bp`, zero
descriptor/effective-ROM violations and matching endpoints. It strictly moves
toward V7 by `614 bp`, with `142` changed target joint cells. Maximum emitted
root-position, orientation and joint-position steps are respectively
`0.000230419 m`, `0.001478205 rad` and `0.001521844 rad`.

R120 consumes exactly one KTO process, one KTO solve, one QP and six emitted
audits. It creates no candidate artifact and runs no inverse dynamics,
kinodynamics, PhysX scene, optimizer or training. The external NPZ is a
solver-private q/v/a warm-start cache only; it has no corpus-admission or
runtime authority. Peak RSS is `1841029120` bytes, below the `16 GiB` ceiling.

R120 canonical/file/profile/cache SHA-256 is
`dfcb05e006467ee30bab70aac00f4408acce26782fbdbf5d1cea89821c06953b` /
`35e35581b4062ce3048cb564a7856787efdd59e1fa12b64eef9526200ea4f2fc` /
`3dfa2f1b8357cd3452481c9518e8d1ca0ce5c0bb664b3a024fc5ce2653837d55` /
`e305fc5888a1cf1dff238c32ac07707a284b215bb49d25920dfb5ee8f097afc5`.
The emitted aggregate SHA-256 is
`06e6113d862dcf6acd827962b8a580013f9e3689902b90c9189efaab641841cf`.
All validations pass, including `257/257` lab tests, `56/56` motor tests and
full `host-check`.

## R121 fixed-PD inverse-dynamics formulation result

Clean report-only R121 at commit
`a20e9bcf04f7c8137955d102fa3f2b64ffb77d1d` binds the R108 stage-2
contract, the complete R113 model identity and exact R120 PASS/cache without
importing OSQP, Pinocchio or an inverse-dynamics implementation. The four
directly emitted cache projections — root/joint position and root/joint
velocity — reproduce the R120 array hashes, and all `801` reference rotations
remain orthogonal with maximum error below `2e-15`.

R121 maps the `800` motor intervals to four pre-integration collocations each,
for `3200` fixed `q/v` samples. Configuration, velocity and acceleration warm
starts are independently affine between neighboring R120 knots; root rotation
uses the principal SO(3) geodesic. This deliberately claims no kinematic
derivative or discrete-integration identity: every motor boundary reanchors to
the next exact R120 knot, while state integration remains exclusive to the
later full-kinodynamic stage.

Each collocation has `64` ordered unknowns: `29` generalized accelerations,
`23` applied efforts and `12` scalar components for the four R113 point forces.
It has `29` rigid-body equations, `23` exact fixed-PD effort identities and
`12` point rows, which are active-contact acceleration closure or inactive
zero force. Across the clip this is `204800` variables and `204800` equality
rows. The frozen V9/R120 schedule yields `4956` active point-collocations,
`7844` inactive ones and `4956` static-friction second-order cones.

The exact controller schedule preflight finds zero intermediate hard-ROM,
velocity, static-effort, power, positive-work or infeasible-envelope events.
Target slew activates once on DoF `6`, by at most `1672 µrad`; the effort-rate
limit activates `641` times on ten DoFs. Maximum requested/applied effort is
`145527100 µN·m`, power `359610852.5932 µW` and positive work `56348 µJ` per
motor tick. These are input-schedule facts only, not rigid-dynamics or native
feasibility evidence.

R121 canonical/file/profile SHA-256 is
`4e6e9494cd7695208aa893fb898003a74f6d3f91fd1ecab583c026509c298ce3` /
`5be2a83f03fd6eb29b61992cfe0995ddb2f419110ba6078bd107a21489343bf7` /
`9145d5f3312d5615df84f5bef6444210e7a7a110b5d41206b0ce582bf45a8c93`.
The formulation module/tool SHA-256 is
`99de51fe96bdaed8efc8c5671272b0cf432011aa5e7318ec81a806385499716f` /
`8e01c6c8b25478747eb565dd9ffb8f2815bf26342560ef99b2d1446674e5d4ae`.
All six validations pass, including solver-free import, `264/264` lab tests,
`56/56` motor tests and full `host-check`. Formulation count is one; local
system, inverse-dynamics, kinodynamic, candidate, PhysX, optimizer and training
counts are zero.

## R122 fixed-PD inverse-dynamics conformance result

Clean report-only R122 at commit
`621ed03c0aa0e459134ef1d09efe9baaf0ab6cf7` implements the R121 equations as
a transparent pure-NumPy descriptor-derived world-coordinate kernel. It uses
engine-world root linear/angular coordinates, exact semantic joint axes and
frames, engine gravity `[0,-9.81,0]`, and reconstructs each computational body
tensor from the descriptor solver principal inertia/frame. Every reconstructed
tensor stays within its authoritative declared projection error; the maximum
actual/declared error is exactly `1785 µkg·m²`.

All seven frozen frames `0/238/244/249/328/626/800` pass. A kinetic-energy
Jacobian sum and independent inertial-wrench inverse-dynamics columns construct
the same mass matrix within `6.72e-15`; maximum relative symmetry error is
`3.54e-18`, minimum eigenvalue `0.0022854963`, and maximum condition number
`34246.81`. Independent inverse/forward acceleration round-trip error is at
most `1.0325e-13`, well inside the frozen `1e-9` scaled bound.

Body/contact positions agree with the pre-existing frozen FK to floating-point
closure. The separately finite-differenced frozen-FK contact Jacobian differs
by at most `4.0309e-10`, against `1e-7 + 1e-6 relative`. Nine active point-
anchor Jdot-v comparisons use a symmetric second derivative of the frozen
point trajectory; maximum error is `4.7426e-8 m/s²` against `1e-5`. R113
normal/right/forward ordering, world-force mapping, application points and
generalized projection all pass independently.

R122 also byte-recomputes all `3200` R121 affine-lift samples, all `800` exact
left-boundary reanchors, the `4956/7844` active/inactive inventory and the
complete fixed-PD schedule. The lift SHA-256 is
`7d0f698fe10803df2becc3aeddf2361277762057fd2a8a0feadf41e0ddb33935`;
contact, inventory and PD structures equal R121 exactly. A single `64 × 64`
float64 matrix plus right-hand side is instrumented at `33280` bytes without
factorization or solve.

R122 canonical/file/profile SHA-256 is
`a03f0a7e605a7e35c370e3ee12dcb7e737c24ee928d00ca92f33c2ff8958d309` /
`8bb3f9cbe371f679ffa3d782ee4662de3fedc586ccf70c9d19536594c95afb81` /
`21303443993a34bd527e735961f0e790aa0da88f2d94b46a4c6e1f85557e0ab2`.
Kernel/conformance/tool SHA-256 is
`220e2db2113196031082eb9fa0b7e3d8614582438da7aaee3f2b6a986bac2a02` /
`25c65634bebeca852519bacfea9c45a8d116f25edfb7e283f70930d6fe71a63e` /
`ed515ba52064d18eebcc54c8d7fd81c63c77431054e8c1e1482156152801b4cc`.
All six validations pass, including solver-free import, `270/270` lab tests,
`56/56` motor tests and full `host-check`. Seven conformance forward-dynamics
probes and `224` inverse-dynamics evaluations are reported explicitly; R123
local-system solves, ID execution, kinodynamics, candidate, PhysX, optimizer
and training counts are zero.

## R123 invalid execution and research gate

The sole R123 process at clean commit `7e9e93c` passed all six validations, then
stopped before its first local solve. Collocation `0` activates right heel and
forefoot on the same `body.right-ankle-roll`; its scaled `64 × 64` matrix has
condition `6.874956301874059e16` against the frozen `1e12` limit. The external
report is canonical/hash-closed, no cache was emitted, and every downstream
work counter remains zero.

The clean [R123 redundant-contact research](humanoid-train4-r123-redundant-contact-research-2026-08-15.md)
confirms the causal mechanism. The two points are separated by a nonzero
line on one rigid body. Equal and opposite forces along that line have zero
resultant force and moment, so their six multiplier components contain an
exact one-dimensional nullspace. The corresponding two 3D sticking Jacobians
have rank at most five. Scaling cannot remove this gauge; the first square KKT
was structurally singular by construction.

R123 therefore supplies no fixed-PD feasibility or infeasibility evidence.
R124 remains unauthorized because its prerequisite was valid R123 completion.
R123-RC1 canonical/file/profile SHA-256 is
`ebf257991c36970e9ccf9501fe4175fc0efa0efed2e3b1a8ac1e176acef045cf` /
`a35d408a901ea2c439ac387287fa03aa77c669863211fb6b0d7634ce3b0836f9` /
`4ee1fd77701e638a3087cbeaa6498482e073c33133bbb89a1a0b6d134d2ee8b7`.
All six validations pass; the research itself reconstructs/factors/solves zero
local systems. Clean R125 now freezes the gauge-aware formulation. It removes
fixed-effort and inactive-force variables algebraically while preserving their
exact identities, every active closure row and all individual cones. The
unchanged inventory is `560` flight, `324` single-point and `2316` flat-foot
collocations; local dimensions are `29/32/35`, maximum gauge dimension is one,
and exact-rational line-cone intersection classifies the whole gauge family
rather than one pseudoinverse witness. Canonical/file/profile SHA-256 is
`ddf443610315680d0326b478212105c846a0cfda2b8bcda95567556ea1773080` /
`bdc5cd5388005bbb549df7bfb723dda49a1535d2a6a39c4e31da58340e3ac0db` /
`ca9cc4019e45ea316072378422e7aab304664b24034f204e41b3ccbe30e7da05`.
Clean R126 at commit `e963599` now conforms that implementation. All seven
frozen real anchors have the exact predeclared rank/nullity, analytic gauge
residual at most `9.056e-16`, and analytic/SVD projector error at most
`1.037e-12`. Synthetic full-rank, one-null, extra-nullity and ambiguity cases
plus independent decimal line-cone oracles all pass. Canonical/file/profile
SHA-256 is
`2a500b6e6514e3a5cc8cec453756089f235d66d8684718a56678c471202f3e8f` /
`4931ff4c96e8bb062bed64a45681097ae70b23c615c022ba267c0ae6edb6ffd3` /
`23007c0455fef7cf84da411528f9f6162cd04d97e8be86baa7be2c19ae7e87cb`.
The sole R127 at clean `44b536b` stops invalid at the first flat-foot state.
Rank/nullity and analytic/SVD gauge agreement pass, but the equality RHS is not
in the matrix range: scaled residual `6.2044653e-5` versus `1e-9`. Clean
report-only R127-RC1 at `9d5cdbf` confirms perpendicular foot angular speed
`0.131848744 rad/s` and line compatibility `-0.003737579615 m/s²`, matching
`-L||omega×d||²` within `4.34e-19`. Its canonical/file/profile SHA-256 is
`a1028728e50050747c2167b45d76726aceebd24a7c881bd11bc9eb1e2c8dcc62` /
`a32f84719b72bf826255287209c596c4faba99646e16112f970acc3c4a5b63ba` /
`1e0137b3c1ba37cedd62da9f20e71616b8cc3a52d2f3e36df49ed3613549ec70`.
Clean report-only R128 at `0220493` selects a mass-metric tangent-velocity
projection as a pointwise diagnostic and leaves position/integration unclaimed.
Canonical/file/profile SHA-256 is
`32a9e278f0c1afe74b646daaf9ef2e446e04e1e56c101caa4e8221cab76cd3d0` /
`15cba3557e487b3af33cb9b9b281501f90aea2fa662151dcbf99277bab46f56e` /
`4a6caece3113ad622ad8a2481bb07849b139eab626015f68511c66cabfe74035`.
Clean R129 at `b81270f` passes all seven frozen anchors and every synthetic
projection guard. The six contact states close active-point velocity to at
most `9.281e-16 m/s`, scaled KKT residual to `1.469e-14`, and projected flat
rigid-line incompatibility to `2.277e-17 m/s²`; the flight state is unchanged.
It performs zero full-schedule projection, inverse dynamics or downstream
work. Canonical/file/profile SHA-256 is
`b34eb4727165e9b16ef82f597138fde33993c12efa8e3177776254237c6fbb99` /
`490b32f67c98d07d9daf0fe9c301372d69b8b85774227658b942b05210531829` /
`d25557c5a07cc243570c1a2b57d9ecb6c8c9ac12bdd31c1ec950f4cfbfc4a376`.
The sole R130 at `4dbd0ac` passes all projection numerics but stops immutable
`INVALID` before inverse dynamics: four speed violations and two empty effort
envelopes occur on right-ankle-roll DoF `11`. Clean report-only R130-RC1 at
`74275e3` confirms that the four largest projection corrections form two pairs
immediately before right-forefoot exit to flight. Exact row/event mapping is
unavailable, so the [R130 research decision](humanoid-train4-r130-projected-schedule-research-2026-08-15.md)
records the boundary. Clean R131 at `9f54f3a` selects a mode-owned left velocity
trace on nine complete exits. Clean R132 at `5c4cb55` passes `36/36`, changes
exactly `27` rows, lowers every changed correction and finds zero local speed
violations. Clean R133 at `03f8e0b` then passes all `3200/3200` projections and
the complete fixed-PD schedule with zero unsafe actuator categories; its
`1011` events are exactly `1010` effort-rate clamps and one target slew. It
still claims no `qdot=v`, integration or dynamic feasibility. Canonical/file
SHA-256 is `f1fad2ca3c7abd49acaefd9fcd37873d02fb1d289a3081fd2039b0b1192fd3b6` /
`2ddef1cfae193eb0e32f4d001aba6a8b744056b700fa2ecbe533434bc8e5f7c2`.
Clean R134 at `5a4085b` freezes its exact projected q/v/effort with the R126
gauge-aware reduced dynamics. It describes `3200` systems, `107668` equality
rows, `4956` cones and `2316` force gauges with zero numeric execution;
canonical/file SHA-256 is
`17d696166a05a24bd50a545ca31e4aabd15a72ed49444a09983c472ff6b13783` /
`0f7ccdcdcb4fbcf1b3e73e1e8f5565b0387c15a4bd1cfa3c50fbd6754ba176be`.

## Decision

Freeze R92–R123, retain the contact result as bounded support for H23, and
reject V11 plus every manual boundary-state or open-loop derivative smoother
as a merged/full-corpus direction. Do not tune controller or solver-limit
values and do not begin training. Reject the raw three-knot V7↔V9 anchor family
and the late projected direction after its exact `2501/2500 bp` failure.
R115 consumed and failed its sole solve; R120 consumed its separate sole solve
and passed; R123 consumed its sole execution and stopped invalid before solve.
None may be retried. Candidate artifacts, native scenes and all-17 remain
blocked. R126 consumed and passed its report-only authority; R127 consumed its
single execution and is invalid without retry. R127-RC1/R128/R129 close the
selected tangent-projection path within their exact claims. R130 consumed its
sole execution and is invalid without retry; R130-RC1 consumed its static
research authority and confirms the actuator conflict plus contact-exit
hotspots. R131 consumes its formulation authority and selects the bounded
nine-exit lift. R132 consumes and passes its `36`-row conformance authority;
R133 consumes and passes the sole full projected-schedule authority; R134
consumes and completes its formulation authority. Neither can retry. Only
report-only R135 implementation conformance is authorized; numeric R136
ID/kinodynamic execution, R124, candidate construction, PhysX and training
remain forbidden.
