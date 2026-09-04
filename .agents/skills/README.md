# Project Skills

Codex discovers these repository-scoped skills from `.agents/skills`.

| Skill | Source path | Commit | License |
| --- | --- | --- | --- |
| `stable-baselines3` | `K-Dense-AI/scientific-agent-skills/skills/stable-baselines3` | `3f825caafe149b7853ec8c4d1dd7f4553ea6b2a5` | MIT |
| `using-deep-rl` | `tachyon-beep/skillpacks/plugins/yzmir-deep-rl/skills/using-deep-rl` | `a86e7855ace8659b13147cbf439cfcf8e93916ed` | CC-BY-SA-4.0 |
| `blender-scripting` | `TerminalSkills/skills/skills/blender-scripting` | `f34cb0e65433b95db3006b1ee8655c06b8d89d53` | Apache-2.0 |
| `onnx` | `TerminalSkills/skills/skills/onnx` | `f34cb0e65433b95db3006b1ee8655c06b8d89d53` | Apache-2.0 |
| `mlflow` | `TerminalSkills/skills/skills/mlflow` | `f34cb0e65433b95db3006b1ee8655c06b8d89d53` | Apache-2.0 |
| `using-rust-engineering` | [upstream](https://github.com/tachyon-beep/skillpacks/tree/a86e7855ace8659b13147cbf439cfcf8e93916ed/plugins/axiom-rust-engineering/skills/using-rust-engineering) | `a86e7855ace8659b13147cbf439cfcf8e93916ed` | [CC-BY-SA-4.0](https://creativecommons.org/licenses/by-sa/4.0/) |
| `using-rust-workspaces` | [upstream](https://github.com/tachyon-beep/skillpacks/tree/a86e7855ace8659b13147cbf439cfcf8e93916ed/plugins/axiom-rust-workspaces/skills/using-rust-workspaces) | `a86e7855ace8659b13147cbf439cfcf8e93916ed` | [CC-BY-SA-4.0](https://creativecommons.org/licenses/by-sa/4.0/) |
| `using-determinism-and-replay` | [upstream](https://github.com/tachyon-beep/skillpacks/tree/a86e7855ace8659b13147cbf439cfcf8e93916ed/plugins/axiom-determinism-and-replay/skills/using-determinism-and-replay) | `a86e7855ace8659b13147cbf439cfcf8e93916ed` | [CC-BY-SA-4.0](https://creativecommons.org/licenses/by-sa/4.0/) |
| `cli-creator` | [upstream](https://github.com/openai/skills/tree/49f948faa9258a0c61caceaf225e179651397431/skills/.curated/cli-creator) | `49f948faa9258a0c61caceaf225e179651397431` | Apache-2.0 (`cli-creator/LICENSE.txt`) |

## Project-Authored Skills

| Skill | Purpose |
| --- | --- |
| `nextengine-training-runner` | Fail-closed preflight and claim-safe preparation of hash-closed reference PPO runs |
| `nextengine-training-diagnostics` | Deterministic artifact, safety, PPO and evaluation diagnosis with one-variable next experiments |
| `nextengine-isaac-correspondence` | CPU-canonical/Isaac-mirror identity and MODEL-MIRROR-P1/P2 evidence audit |
| `maintain-task-context` | Recover task state and preserve decisions or constraints that would otherwise be lost |
| `nextengine-architecture` | Route changes to governed contracts, authority boundaries and product-level roadmap facts |

These skills are native NextEngine guidance. Their workflow design applies the
useful parts of the imported deep-RL, experiment-tracking and determinism
skills, while replacing generic framework assumptions with the repository's
SPEC/ADR/profile/generation authority. Their scripts use only the Python
standard library, are read-only, and emit machine-readable JSON.

## Third-Party Attribution

The `tachyon-beep/skillpacks` skills above are locally adapted copies from the
recorded commit, attributed to the upstream authors and redistributed under
[CC BY-SA 4.0](https://creativecommons.org/licenses/by-sa/4.0/). Their exact
source directories and commit are linked in the table. Preserve the upstream
[license](https://github.com/tachyon-beep/skillpacks/blob/a86e7855ace8659b13147cbf439cfcf8e93916ed/LICENSE)
and [license addendum](https://github.com/tachyon-beep/skillpacks/blob/a86e7855ace8659b13147cbf439cfcf8e93916ed/LICENSE_ADDENDUM.md)
when redistributing these skills; they are not relicensed under the engine's
Apache-2.0 license.

`cli-creator` is a locally adapted copy from the recorded OpenAI Skills commit. Its
upstream Apache-2.0 license text is preserved at
`cli-creator/LICENSE.txt`.

## Local Adaptations

- Moved the upstream `compatibility` frontmatter field under `metadata` so the
  skills pass the current Codex skill validator.
- Removed TerminalSkills `_scores.json` files because their content hashes no
  longer match the locally adapted `SKILL.md` files.
- Narrowed imported skill triggers and workflows for NextEngine; upstream
  attribution and license terms remain attached to the adapted files.
