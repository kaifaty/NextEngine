# Architecture promotion review: packet 1.6

| Field | Value |
|---|---|
| Record ID | ARCH-REVIEW-1.6 |
| From packet | 1.5.1 |
| To packet | 1.6 |
| Status | Approved |
| Candidate root algorithm | sha256-path-nul-file-sha256-lf-v1 |
| Candidate scope | docs/architecture/**/*.md |
| Candidate root SHA-256 | 0826bd1f005676eec92e19ee70c1d0f176ac8f883fbfc2422027526e30927cd8 |

## Candidate file manifest

| Path | SHA-256 |
|---|---|
| docs/architecture/00-product-contract.md | 0d43957cbadfc8c6639f6604285624d5451df4e55fdb7746e2737d86e5229aa6 |
| docs/architecture/01-system-architecture.md | 559f4199ef751fa7c5a0abbce77dae983dd339ed6e9161e15a5e679384c3c0df |
| docs/architecture/02-runtime-ecs-and-data.md | 66167355465d28dd91c00fec3858eff70206d4cdbb473c08696d9785a73f1db3 |
| docs/architecture/03-assets-world-streaming-and-persistence.md | 03041d6bf1b200585ba42210958e458d787ee6c8193c6f6b2b8edcec2f96e2a7 |
| docs/architecture/04-rendering-and-platform.md | 33083e678530f269886a647c575c853bd1215b21a04404c7d8ad260628b3b644 |
| docs/architecture/05-physics-animation-and-motor-control.md | 90749b6e3877a5975161066ca5b3435d0436323aed737f4561f5c5e33f49d6f7 |
| docs/architecture/06-ai-agents-perception-and-memory.md | 3033e96547097afff0e91d33345f8264cf3876f10036cf3a29bc53d0e298eb36 |
| docs/architecture/07-rpg-scripting-and-plugins.md | f982880cfc94a76a318efe47397793b090cd6756e1906220512a055ca2e987c7 |
| docs/architecture/08-audio-navigation-and-world-services.md | 34570b046e8e8c6693798113c7562b65a08a197783a09027f5a0de7c7849759d |
| docs/architecture/09-tooling-sdk-and-observability.md | bf4648e689b6c30f1661c15cfb6865cab8743b6e08d38676a86c52cc5c2c5094 |
| docs/architecture/10-gothic-importer-boundary.md | 70c95d09baf07be7ab83e15e08d31b5db394f061cdb7580e28429e74b68be98b |
| docs/architecture/11-security-licensing-and-governance.md | d255e61001c1c2903524eda307431528fae2f9be34eaa5fd703a1f350804cee2 |
| docs/architecture/12-vertical-slice-conformance.md | df3c7266a66734d0d970673629c6bddc2f7c383e1cea3319c155545293b19c5c |
| docs/architecture/13-gameplay-mechanics-mod-packages-and-agent-authoring.md | ee464b3a58c117ad8a594d70346cea9813698dc3a3c798f4a3c10e45750cd48e |
| docs/architecture/14-physical-archetypes-motor-skills-and-policy-lifecycle.md | 151e72d5455e59fd9b9249c4f943ad7d94e14adb266b63b794747caa5add949a |
| docs/architecture/15-headless-testing-agent-validation-and-human-evidence.md | 3f21f05a18a89874c9e7ec76eb2d98a73cf94bfe0a8c0a0253b3dce5ba1b487e |
| docs/architecture/16-text-canonical-multimodal-dialogue-and-model-packs.md | 5af39dc7c7677315d6cfefbae90395cf703c115863bdfe361ba60592a83d8017 |
| docs/architecture/17-project-composition-configuration-and-application-lifecycle.md | 2680982ddebe51c5c13a31cc9bd41cb834ad75765d3388415b1983c4225eff4a |
| docs/architecture/18-player-interaction-ui-camera-localization-and-accessibility.md | 65e9227f2a470174be77d02ab174aa42c6187e163aa7e8a634d1bc6d2cf8e7c5 |
| docs/architecture/19-rpg-domain-and-narrative-state.md | f1fd14bca0ffc705d257fa7952b7d0dd151ed410c802a4f4185152e4dd31afe1 |
| docs/architecture/20-world-simulation-and-population-lifecycle.md | bce9eb77facc8469eadd87486c831dc1eecc883cc6266524b8162620c946ec12 |
| docs/architecture/21-deterministic-runtime-primitives-command-ledger-and-causal-identity.md | adc183e5f3f57d9b6d9cac04e0e24e45474884754e7ad8b531e7f834b954c64c |
| docs/architecture/README.md | e21bc62785e196e8bc1b77940d41bc1542dba9cf5172f415297ff20736ce9e88 |
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
| docs/architecture/adr/018-authoritative-project-composition-and-configuration.md | be8264fc02e7cce21f980901303cb35049a0824017b03d617b5bcbe3a7a5f4cd |
| docs/architecture/adr/019-canonical-player-actions-and-presentation-authority.md | bd2a6afa6ebe741993223916e801d1d741f42b84b9891dc3cf22b75eb708cfe8 |
| docs/architecture/adr/020-rpg-domain-authority-and-extension-boundary.md | 99bbb82341d73fd076afb7e39dacfb7d9ed8592de5ec66eb4385f2334f7d192e |
| docs/architecture/adr/021-deterministic-population-residency-and-time-advance.md | 1530d9f07c2c485316fbacbd81d0a597afd93f935051c6371f6cee80f8c956c0 |
| docs/architecture/adr/022-deterministic-command-identity-ledger-and-causal-identity.md | 66697739f03cc0403d5513e23ee110fc363e71161daeaa22410d9afcd6ce7e0b |
| docs/architecture/adr/023-human-review-decision-v2-and-offline-attestation.md | 48d0f808d3b5ee5b16b06513d432da1c6256fd5fadab688627949d0ff4c28f48 |
| docs/architecture/adr/024-requirement-gate-evidence-and-profile-closure.md | 17cda07acc89e40eb8aa2b5c4c6fe1e0de9d243c43f51736116dd71f746f06e4 |
| docs/architecture/evidence-register.md | 3084cbe57db111e430393a2b54e69040abb99238ca1d59b92affcb162d0cb27d |
| docs/architecture/glossary.md | 2624563752a7b6895fb8fcb3824e85898b36cb10f9d311d94cafc2aa7361ce8e |
| docs/architecture/research/npc-dialogue-model-landscape.md | 4b9a27e90ea5659b224a2595c9980e28fdc1396158a28b0a80ebe7931021c30c |
| docs/architecture/research/physical-avatar-research-spec.md | 90533ed15c4c1a5ef41a24f26f4d17cf8c59f467e07619316d3c9744f4d2d79b |
| docs/architecture/traceability.md | aeee47786b357b8d7cd3c834d7002dbb5a06634b1c8b095ed4f3b547a92e458d |

## Automatic checks

| Check | Result | Evidence reference |
|---|---|---|
| cargo fmt --all -- --check | PASS | local transcript 2026-07-24: rustfmt check passed |
| cargo clippy --workspace --all-targets -- -D warnings | PASS | local transcript 2026-07-24: workspace clippy passed with warnings denied |
| cargo test --workspace | PASS | local transcript 2026-07-24: 171 tests passed |
| cargo run -p xtask -- boundary-scan | PASS | local transcript 2026-07-24: boundary-scan PASS |
| git diff --check | PASS | local transcript 2026-07-24: no whitespace errors |
| cargo run -p xtask -- architecture-review-preflight 1.6 | PASS | local transcript 2026-07-24: candidate root 0826bd1f005676eec92e19ee70c1d0f176ac8f883fbfc2422027526e30927cd8 |

## Bootstrap capability decisions

| Capability | Decision | Reviewer | Decision reference |
|---|---|---|---|
| architecture.promote | Approved | Kaifaty | 0826bd1f005676eec92e19ee70c1d0f176ac8f883fbfc2422027526e30927cd8 |

The Repository Owner approved promotion of the exact candidate root recorded above. Automatic verification did not create or substitute this human decision.
