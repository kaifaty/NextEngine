# Architecture promotion review: packet 1.5.1

| Field | Value |
|---|---|
| Record ID | ARCH-REVIEW-1.5.1 |
| From packet | 1.5 |
| To packet | 1.5.1 |
| Status | Pending |
| Candidate root algorithm | sha256-path-nul-file-sha256-lf-v1 |
| Candidate scope | docs/architecture/**/*.md |
| Candidate root SHA-256 | c0f76ba2eb594160a55608a5e029753144746f33d4f267d47c29c9f4e77ec443 |

## Candidate file manifest

| Path | SHA-256 |
|---|---|
| docs/architecture/00-product-contract.md | f8182b74a0133ccfec0237d31956d8ceab73d17d3f76d43387e443146c1e0929 |
| docs/architecture/01-system-architecture.md | c31312164aa75908fb2df3d9353713cdf2b83e2e3101c9c578dfdabe1cbf3349 |
| docs/architecture/02-runtime-ecs-and-data.md | faf7e858b97292f6dbb36f95aa8e2e6b18d698294d2471274a9a1c4f0c1ce255 |
| docs/architecture/03-assets-world-streaming-and-persistence.md | 78b0512e82996b6dedc152eb3e77ca5fb41f0df61acbe7697e380a752a055fbc |
| docs/architecture/04-rendering-and-platform.md | 941c20259a02e94b68ab0658cbca34914f92eddd48d4d78ab07a21062eca4c92 |
| docs/architecture/05-physics-animation-and-motor-control.md | 6d2fc960a778c73f3aeba9e7932ebf255deeade4447a5a5e08ec7b2892823752 |
| docs/architecture/06-ai-agents-perception-and-memory.md | 3033e96547097afff0e91d33345f8264cf3876f10036cf3a29bc53d0e298eb36 |
| docs/architecture/07-rpg-scripting-and-plugins.md | e13440b6269e2494b29cd7fb39917109608d126e8b8d9e4879cc7dc86dcbd517 |
| docs/architecture/08-audio-navigation-and-world-services.md | faab4ca364f8aaeddb9f42c63c2a966030171dae88c5413ab8eef86ed2e7fc4b |
| docs/architecture/09-tooling-sdk-and-observability.md | deae192ca6717ec4116eae61f8ac26c64fa07fd569e78867a9b90f290b6d0785 |
| docs/architecture/10-gothic-importer-boundary.md | 7b99c58494c8378e76156c11dcea0edc7e25403498db80db16db55d6f31af25b |
| docs/architecture/11-security-licensing-and-governance.md | cd67537d5723d58f18db38a82471824c6303fa8ceeb836b6463de8ac79b4f244 |
| docs/architecture/12-vertical-slice-conformance.md | 306ccbd52e8efdd4ebc3674c6e24a87d11b1ce224c2c809f9e590c0b3718d810 |
| docs/architecture/13-gameplay-mechanics-mod-packages-and-agent-authoring.md | 9f2e9b0d724cfce3d31c04f25cc4eedde4c553f4e109754d44be025be9e07dbe |
| docs/architecture/14-physical-archetypes-motor-skills-and-policy-lifecycle.md | feb747c710accd39d4e69d08fe8bb9aee39d49aa997bfc614c1fef15848d6b90 |
| docs/architecture/15-headless-testing-agent-validation-and-human-evidence.md | f72341c9544bd8a322696fdbd8fa0d2598c9a1b705dfd35d583d1c8ab02fb4fe |
| docs/architecture/16-text-canonical-multimodal-dialogue-and-model-packs.md | 3248c95ece7274a2c6d3711569d20dba61819e5048ba40a5bbc46e934f7d6cec |
| docs/architecture/17-project-composition-configuration-and-application-lifecycle.md | 8426b1cc6fd7623241db254b6746d8a80aa4b2a7f780ebaf87a1b73378d3a9b2 |
| docs/architecture/18-player-interaction-ui-camera-localization-and-accessibility.md | d3ea1b452af7145923991988d8d785504c09148132e891410858a861c9edb0c7 |
| docs/architecture/19-rpg-domain-and-narrative-state.md | 51c28ee5431206beb7b53a60c82217f698c88e955ecababb3d0cecd4d2a4f192 |
| docs/architecture/20-world-simulation-and-population-lifecycle.md | 9fc751587f928e992397703f27b7b30d3dae072d96a56fe40677a832422973a0 |
| docs/architecture/README.md | ef57564f8e00b460a90a2e40afe169285e7b9e8f318d2d9bc70caad9c9524b18 |
| docs/architecture/adr/000-template.md | 786a3bbd63a429ddddd09505381e5502d5a57fdf469b3fddf4ca629553ebe78b |
| docs/architecture/adr/001-product-repository-license-and-platforms.md | 081dce87d1ba6e54526aa0f733b0e1e111443950144a64f19d19f81e1482bd91 |
| docs/architecture/adr/002-rust-first-ffi-and-ecs-facade.md | 9a11b679a15eea1c12171813a664de4c5f06613b7c317c81f929e49eae502196 |
| docs/architecture/adr/003-vulkan-renderer-and-shader-toolchain.md | e1b23ff81bc3dca375063f8c86ef84c0e31ccb4699273f13cc76d4a6c26cf7d8 |
| docs/architecture/adr/004-physics-avatar-backend-boundary.md | 79beaa24c8134dabe7cde3ab71b777afb9db33fc0579b60210498f422bd96cf8 |
| docs/architecture/adr/005-offline-first-ai-process-boundary.md | bd72d1011fffee006d3c0bb98178d4465987c6d848435fab5ee4b3f41b32ddf3 |
| docs/architecture/adr/006-scripting-and-plugin-model.md | 7159fb17b5e4f46bd453199ad1e687672d1db270eaa153e8c7a76cbdd21480f1 |
| docs/architecture/adr/007-identities-persistence-and-replay.md | e3e94d1d67455be4cdf3c05b9d299fe3f69769d0477c6ec4d4184f7352332c96 |
| docs/architecture/adr/008-mechanics-mod-package-and-agent-authoring-model.md | e389b92e846808c05b3e8f8d14e81ca7b29e9236975bf83531a08bc79cd98242 |
| docs/architecture/adr/009-pretrained-foundation-policies-and-progressive-motor-skills.md | 0cc2800b5bb2c225a6536e1e8486104790fa97db0b651166b759b4b430917539 |
| docs/architecture/adr/010-artifact-first-headless-validation-and-review.md | 99256b81614b1b9d6c585dee414287e7b873e9c5c1a54063973f356034f1d363 |
| docs/architecture/adr/011-macos-developer-host-local-verification-and-staged-training.md | 6d52d5f50d01013464db147ef81e90e03330d417daba88991f30e4690beecadc |
| docs/architecture/adr/012-deterministic-command-identity-and-replay.md | 24dda5e497075fe7a5e5cfabdbb56d608fa00346e19c4e07d9f95b6388e27bb8 |
| docs/architecture/adr/013-self-contained-physical-avatar-boundary.md | e908f0324f0ab46c0b8ee05a2dc19c1fe1e06c891d9dd8304f2103db1403846e |
| docs/architecture/adr/014-deterministic-extensions-and-package-trust.md | f30700d9cb238c7f081d8ccfaa2b47425476a8866335beb633c12a75301c482f |
| docs/architecture/adr/015-evidence-trust-fixture-separation-and-attestation.md | 95a2d04dc568946cdb90b9ab98f40c0c8c3f15be667155bd1828380df0f66257 |
| docs/architecture/adr/016-compositional-gameplay-budgets.md | d78219dcbb3fae3961ecd520928a767d4027a6c380318ce47a3569617d4b61ef |
| docs/architecture/adr/017-text-canonical-multimodal-dialogue-and-replaceable-model-packs.md | a604dfb824073a289935942d339cfb07804ef386a04553b8fb291d0b4c36f59f |
| docs/architecture/adr/018-authoritative-project-composition-and-configuration.md | a04e5e251969bd0928acb5b7cbf13ce12254f448cf72fe918c0b3062f1299273 |
| docs/architecture/adr/019-canonical-player-actions-and-presentation-authority.md | c5b019e189d19c718953372fbc9ba23f9f19ad279c4f48ce6b8ddd2608bce2ad |
| docs/architecture/adr/020-rpg-domain-authority-and-extension-boundary.md | 4b2a35d8eda3501eac464af4accaa694dfededea300e496d237e622883894129 |
| docs/architecture/adr/021-deterministic-population-residency-and-time-advance.md | 08690718f7c7a98e52adb1289a668a4c45d0615cf0cdd401c082500be27ef7cf |
| docs/architecture/evidence-register.md | 7e2fc04d724668ac62b24f2761c1b1900d85c5a7a5f640ee86862d327cf14241 |
| docs/architecture/glossary.md | e5e31fad9e1970a546556da5c24445696fece4418d55d0aebc60a3aefdb65561 |
| docs/architecture/research/npc-dialogue-model-landscape.md | 4b9a27e90ea5659b224a2595c9980e28fdc1396158a28b0a80ebe7931021c30c |
| docs/architecture/research/physical-avatar-research-spec.md | 90533ed15c4c1a5ef41a24f26f4d17cf8c59f467e07619316d3c9744f4d2d79b |
| docs/architecture/traceability.md | d6b1bdbebe794144916e409cd395fc607f83498fd65de71262ab998ccf5c016d |

## Automatic checks

| Check | Result | Evidence reference |
|---|---|---|
| cargo fmt --all -- --check | PASS | local transcript 2026-07-24: rustfmt check passed |
| cargo clippy --workspace --all-targets -- -D warnings | PASS | local transcript 2026-07-24: workspace clippy passed with warnings denied |
| cargo test --workspace | PASS | local transcript 2026-07-24: 107 tests passed |
| cargo run -p xtask -- boundary-scan | PASS | local transcript 2026-07-24: boundary-scan PASS |
| git diff --check | PASS | local transcript 2026-07-24: no whitespace errors |
| cargo run -p xtask -- architecture-review-preflight 1.5.1 | PASS | local transcript 2026-07-24: candidate root c0f76ba2eb594160a55608a5e029753144746f33d4f267d47c29c9f4e77ec443 |

## Bootstrap capability decisions

| Capability | Decision | Reviewer | Decision reference |
|---|---|---|---|
| architecture.promote | Pending | absent | absent |

This record is intentionally Pending. Automatic verification cannot create or substitute a human decision.
