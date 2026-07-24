# Architecture promotion review: packet 1.8

| Field | Value |
|---|---|
| Record ID | ARCH-REVIEW-1.8 |
| From packet | 1.7 |
| To packet | 1.8 |
| Status | Approved |
| Candidate root algorithm | sha256-path-nul-file-sha256-lf-v1 |
| Candidate scope | docs/architecture/**/*.md |
| Candidate root SHA-256 | e04a4ef255599df1d3f8f6a96b9f9e0b6c0bf745a57ca63e68d5c0df5107ce5f |

## Promotion scope

This consolidated candidate admits the remaining decision-complete P0
architecture in one review set:

- SPEC-22…25 and ADR-025/026: schema/compatibility/migration,
  jobs/resources/backpressure, neutral content/bundles/variants and world
  partition/streaming/persistent spatial objects;
- SPEC-26…28 and ADR-027: engine-owned physics/query/contact/snapshot, exact
  motor inference/safety/state and deterministic animation/retarget/root-motion/
  physical-versus-presentation IK contracts;
- SPEC-29/30 and ADR-028: normalized platform facts, one application-session
  lifecycle with exactly-once close/save, immutable presentation extraction,
  neutral material/shader/color/VFX and reconstructible caches.

It converts `REQ-112`…`REQ-147` and `FAIL-044`…`FAIL-061` to Accepted
traceability. SPEC-16/ADR-017 remain Deferred Proposed with only
`REQ-079`…`REQ-086` and `FAIL-025`…`FAIL-030` reserved.

No external technology status changes. Acceptance would admit architecture
contracts only; it would not create runtime implementation, automatic or human
gate PASS, `vertical-v1`, `PhysicalCertified`, shipping, release or importer
claims.

## Candidate file manifest

| Path | SHA-256 |
|---|---|
| docs/architecture/00-product-contract.md | b82ba8bc77a361f63b0e226a725751cf8db591a2b38580f9b38d0365649adb50 |
| docs/architecture/01-system-architecture.md | c036a427155309b9916ce65b4db47f0c2e124313e81350da2721d8305982e1a6 |
| docs/architecture/02-runtime-ecs-and-data.md | a1cd650cc620392fb33c4ab5d8feb260da22e7e10fc2bd7749a3eb3f6c9a9b02 |
| docs/architecture/03-assets-world-streaming-and-persistence.md | 5f81c4c7e41d17f34242f1c56c09eec404c92fa42516d2d75fe06c6b4922ef5b |
| docs/architecture/04-rendering-and-platform.md | 826c932ea02028bb64e0af2f24d0ca3387a846a2888b1c95beac933a8b31777f |
| docs/architecture/05-physics-animation-and-motor-control.md | 5c6f9cbeaa7e4c6b0a0eb61ae50f65393012ca3fdb93cdbdec974fac205dd8da |
| docs/architecture/06-ai-agents-perception-and-memory.md | d0c4cdb93bcd890e01caed94c5fc6f5fdafe0153e35108d6d93b52ce75bcbf43 |
| docs/architecture/07-rpg-scripting-and-plugins.md | 498857168196a2ef00d795261da65cedb8b396f5893371e8114b14bd4d896d91 |
| docs/architecture/08-audio-navigation-and-world-services.md | 411e14ba0b6d776afe60c0c4a18b0ef4b93128560e53dddf5e90e9a05bcfcc19 |
| docs/architecture/09-tooling-sdk-and-observability.md | b5a892d3cf0c742b0a68a5f95cbf3ff09616f90ae2816960df79f8ae8a0a8aad |
| docs/architecture/10-gothic-importer-boundary.md | 70c95d09baf07be7ab83e15e08d31b5db394f061cdb7580e28429e74b68be98b |
| docs/architecture/11-security-licensing-and-governance.md | 528bc1b651a5ce4c8571a4b84f92a3078faae00504defeb396a253e06a5162ec |
| docs/architecture/12-vertical-slice-conformance.md | 4212c1214271396f7616b5530421e97d3952a707df0bd58cb21daaa1b95b4e9a |
| docs/architecture/13-gameplay-mechanics-mod-packages-and-agent-authoring.md | 08b96e074050b68c4f32ef14121bd45b342d680ee539538d4e8a571355bd7641 |
| docs/architecture/14-physical-archetypes-motor-skills-and-policy-lifecycle.md | 1ad1ac4e4f371b237c1f8f77ec1786fe8470bde8a6c20961effbaffb0aa098d9 |
| docs/architecture/15-headless-testing-agent-validation-and-human-evidence.md | 5d9c7387280c11ecd7193b87b82ef2b7e666d17235bee7091d9c51d2f072a5d3 |
| docs/architecture/16-text-canonical-multimodal-dialogue-and-model-packs.md | 5af39dc7c7677315d6cfefbae90395cf703c115863bdfe361ba60592a83d8017 |
| docs/architecture/17-project-composition-configuration-and-application-lifecycle.md | 28edca3968a52729a754348b8afc4eeaf97943b1174f18f6b1ba340b1a9130bc |
| docs/architecture/18-player-interaction-ui-camera-localization-and-accessibility.md | c3fe5876bbd452bd2f4a88b1490ba6330041ad87c13abd690beabbfb85d13cdb |
| docs/architecture/19-rpg-domain-and-narrative-state.md | c2b1ad233cf2b73eba2ed7af1b21a274df3d2dff4dd7f2681e377d2bf0e6d446 |
| docs/architecture/20-world-simulation-and-population-lifecycle.md | 71b97752669eecc7a57da8eaf6e62f2fd2cfec7b30b216327f2b5480ee1fca7a |
| docs/architecture/21-deterministic-runtime-primitives-command-ledger-and-causal-identity.md | adc183e5f3f57d9b6d9cac04e0e24e45474884754e7ad8b531e7f834b954c64c |
| docs/architecture/22-schema-registry-compatibility-and-migration.md | c6fa5a3c292d76b364f0c62eca36708af90f7fb26df3eb71a91e1388739afd3f |
| docs/architecture/23-jobs-memory-resource-residency-and-io-backpressure.md | acbfb5815ad48f956dd15e07bf2a4c261fa7aa6378119d2e2559606777318e5b |
| docs/architecture/24-content-catalog-bundle-and-neutral-asset-schemas.md | fab8feb508e04fdbc5f23ee38e05376095960e55fbd456e4ad7296a91f45d05b |
| docs/architecture/25-world-partition-streaming-admission-and-persistent-spatial-objects.md | 7c737bf8ae1f5dcaa26b814e061affa2bbfa1d721fdb491b3085b7fce4179ff6 |
| docs/architecture/26-physics-world-collision-constraints-queries-and-canonical-snapshots.md | 37b9c7f37a4d6c71145e504fdf0d62abb789ec3d83ed465e59d765e81ca8a515 |
| docs/architecture/27-motor-observation-action-and-deterministic-inference.md | ea6f99a904f69d9772052767c873e13619fea60b1f8f795b141e4765660da1fa |
| docs/architecture/28-skeletal-animation-retargeting-and-ik.md | b1bc16b3bd467395fa772845724c13155c19aa018edc72f08f2fed56a66f5934 |
| docs/architecture/29-platform-host-and-application-session.md | 67d6ca08f25e3c876dec70d9042ecf8f1b11cf7b798e7edc4842a5549c0b1027 |
| docs/architecture/30-presentation-extraction-and-render-content.md | d5113de81c97da5f3c03c27cfe64c32d2a007dce6ad89b4643cdae0411f8cead |
| docs/architecture/README.md | dd4b893a3fa587264d5ed48caa18a1dc601628f318fa6bb12e7644a614ab26f1 |
| docs/architecture/adr/000-template.md | 786a3bbd63a429ddddd09505381e5502d5a57fdf469b3fddf4ca629553ebe78b |
| docs/architecture/adr/001-product-repository-license-and-platforms.md | 081dce87d1ba6e54526aa0f733b0e1e111443950144a64f19d19f81e1482bd91 |
| docs/architecture/adr/002-rust-first-ffi-and-ecs-facade.md | 9a11b679a15eea1c12171813a664de4c5f06613b7c317c81f929e49eae502196 |
| docs/architecture/adr/003-vulkan-renderer-and-shader-toolchain.md | 1966514fcdd0a52a4443fd6de0eb72373253ac5e50693fa37d157be654e279bf |
| docs/architecture/adr/004-physics-avatar-backend-boundary.md | 79beaa24c8134dabe7cde3ab71b777afb9db33fc0579b60210498f422bd96cf8 |
| docs/architecture/adr/005-offline-first-ai-process-boundary.md | bd72d1011fffee006d3c0bb98178d4465987c6d848435fab5ee4b3f41b32ddf3 |
| docs/architecture/adr/006-scripting-and-plugin-model.md | 7159fb17b5e4f46bd453199ad1e687672d1db270eaa153e8c7a76cbdd21480f1 |
| docs/architecture/adr/007-identities-persistence-and-replay.md | e3e94d1d67455be4cdf3c05b9d299fe3f69769d0477c6ec4d4184f7352332c96 |
| docs/architecture/adr/008-mechanics-mod-package-and-agent-authoring-model.md | e389b92e846808c05b3e8f8d14e81ca7b29e9236975bf83531a08bc79cd98242 |
| docs/architecture/adr/009-pretrained-foundation-policies-and-progressive-motor-skills.md | 0cc2800b5bb2c225a6536e1e8486104790fa97db0b651166b759b4b430917539 |
| docs/architecture/adr/010-artifact-first-headless-validation-and-review.md | 99256b81614b1b9d6c585dee414287e7b873e9c5c1a54063973f356034f1d363 |
| docs/architecture/adr/011-macos-developer-host-local-verification-and-staged-training.md | 6d52d5f50d01013464db147ef81e90e03330d417daba88991f30e4690beecadc |
| docs/architecture/adr/012-deterministic-command-identity-and-replay.md | 4e02ddff004c22f2e085b28c701c23f901dc05d8d625f6e1e3e870b02c291367 |
| docs/architecture/adr/013-self-contained-physical-avatar-boundary.md | aed13e6a6835c99f6f50ec067eb038095d9e8470372772431995c5ea912daa08 |
| docs/architecture/adr/014-deterministic-extensions-and-package-trust.md | 9817b8a99169b286e00c562d05bb976b2fefe9212c0ce04e08a7b5f7d595017a |
| docs/architecture/adr/015-evidence-trust-fixture-separation-and-attestation.md | 34e7702506911511374e6c0ebf95b5b03f75fd458599dfdc5eeaeb0923abb9e6 |
| docs/architecture/adr/016-compositional-gameplay-budgets.md | 313e9740c0675f653f991722d2e8d0faaac538dae93af817153467c7993d361e |
| docs/architecture/adr/017-text-canonical-multimodal-dialogue-and-replaceable-model-packs.md | 086b2b414f23b6b1175323b94ca184a21df91d8613bc1d2d723e2c6826527173 |
| docs/architecture/adr/018-authoritative-project-composition-and-configuration.md | eef9473da1b91f3da87fa4f87f1b8058cfb4b59a2a785b8c1005e46bb63e0fc9 |
| docs/architecture/adr/019-canonical-player-actions-and-presentation-authority.md | 97e591aba64360dbdb91fcf91f236e6436ebca74eef8727d6f382a42786e3697 |
| docs/architecture/adr/020-rpg-domain-authority-and-extension-boundary.md | 10f8d6a5ea72af35bf62ea8db810ef99c5e0e5b2d7c1be9ee4a366f1660b5518 |
| docs/architecture/adr/021-deterministic-population-residency-and-time-advance.md | c50c4ba421f9e9abb75dc7502b665ceaeb82e5afc7828d279c3c72c6eabfb443 |
| docs/architecture/adr/022-deterministic-command-identity-ledger-and-causal-identity.md | 66697739f03cc0403d5513e23ee110fc363e71161daeaa22410d9afcd6ce7e0b |
| docs/architecture/adr/023-human-review-decision-v2-and-offline-attestation.md | 48d0f808d3b5ee5b16b06513d432da1c6256fd5fadab688627949d0ff4c28f48 |
| docs/architecture/adr/024-requirement-gate-evidence-and-profile-closure.md | 17cda07acc89e40eb8aa2b5c4c6fe1e0de9d243c43f51736116dd71f746f06e4 |
| docs/architecture/adr/025-schema-content-and-migration-authority.md | c0d3ec8fe450ae3b69b710e9786b67ef7ec421033062963a195d05dc6d72e49a |
| docs/architecture/adr/026-deterministic-work-resource-and-streaming-admission.md | f283f293852ceba4dac15e3ff753802134730f175baba46f235a8efde113e94f |
| docs/architecture/adr/027-physics-motor-and-animation-layering.md | 7c388b10274d3465bafeaa6d5d1acd1909014d656c12425d735176351e865480 |
| docs/architecture/adr/028-platform-session-and-presentation-authority.md | 691753c73673f496939d40f94f4ffad9d4b4a8ce42600164bf6d4c0c6421cb4f |
| docs/architecture/evidence-register.md | f4264ef0b3da6b55e18e9724b2177ab7dfffd571eadf35cee6f4fdfac1913c42 |
| docs/architecture/glossary.md | 07ad7ef9bea7239bf231ed54b04b69ac37ea8c00f562f473a580d28ce24e0283 |
| docs/architecture/research/npc-dialogue-model-landscape.md | 4b9a27e90ea5659b224a2595c9980e28fdc1396158a28b0a80ebe7931021c30c |
| docs/architecture/research/physical-avatar-research-spec.md | 90533ed15c4c1a5ef41a24f26f4d17cf8c59f467e07619316d3c9744f4d2d79b |
| docs/architecture/traceability.md | 6c263440293e3354292ba6dbbd50edcda1040a058f6f59948d51b313bebedfaf |

## Automatic checks

| Check | Result | Evidence reference |
|---|---|---|
| cargo fmt --all -- --check | PASS | local transcript 2026-07-24: rustfmt check passed |
| cargo clippy --workspace --all-targets -- -D warnings | PASS | local transcript 2026-07-24: workspace clippy passed with warnings denied |
| cargo test --workspace | PASS | local transcript 2026-07-24: 193 tests passed |
| cargo run -p xtask -- boundary-scan | PASS | local transcript 2026-07-24: boundary-scan PASS |
| git diff --check | PASS | local transcript 2026-07-24: no whitespace errors |
| cargo run -p xtask -- architecture-review-preflight 1.8 | PASS | local transcript 2026-07-24: candidate root e04a4ef255599df1d3f8f6a96b9f9e0b6c0bf745a57ca63e68d5c0df5107ce5f |

## Bootstrap capability decisions

| Capability | Decision | Reviewer | Decision reference |
|---|---|---|---|
| architecture.promote | Approved | Kaifaty | e04a4ef255599df1d3f8f6a96b9f9e0b6c0bf745a57ca63e68d5c0df5107ce5f |

Repository Owner `Kaifaty` approved this exact candidate root. Automatic
verification did not create or substitute that human decision.
