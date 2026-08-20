# Creator Smoke

`creator-smoke` is the independent data-only acceptance project for the public
R6 creator workflow. It is deliberately small, but it exercises the current
authoring closure: character, item, ability, dialogue, quest, relationship,
interaction, three streamed chunks, minimal render content and world services.

The canonical clean-checkout tutorial, post-create JSON exercise, extension
examples and failure/output rules are in the [Creator SDK beta
guide](../../docs/creator-sdk.md).

## Cold authoring

Create a new independently namespaced copy of this RPG starter in an absent
directory:

```text
cargo run --locked -p next_cli -- project create --template rpg-starter --project-id org.example.my-rpg --output target/my-rpg
```

The command writes `project.authoring.json`, `NOTICE`, `README.md` and the
referenced CC0 source only after the generated Project Authoring V7 document
loads and cooks successfully. Edit the generated manifest to change its one
NPC, ability, quest/dialogue interaction and three streamed chunks; project-
owned IDs are already under the requested namespace. Create never merges with
or overwrites an existing destination.

## Existing project workflow

From the repository root:

```text
cargo run --locked -p next_cli -- project validate --project projects/creator-smoke
cargo run --locked -p next_cli -- project cook --project projects/creator-smoke --output target/creator-smoke-cooked
cargo run --locked -p next_cli -- project run --project projects/creator-smoke
cargo run --locked -p next_cli -- project package --project projects/creator-smoke --output target/creator-smoke-package
cargo run --locked -p next_cli -- project run --package target/creator-smoke-package
cargo run --locked -p next_cli -- project inspect --project projects/creator-smoke
cargo run --locked -p next_cli -- project inspect --package target/creator-smoke-package
cargo run --locked -p next_cli -- project diff --base-project projects/creator-smoke --candidate-package target/creator-smoke-package
```

Each command emits exactly one versioned JSON object on stdout: Creator Command
Report V1 for validate/cook, Creator Run Report V1 for either run spelling and
Creator Package Report V1 for package, plus separate Creator Inspect/Diff Report
V1 contracts. Run executes one project-neutral
production headless tick and ordinary final save-on-close. The package contains
the exact published ContentStore plus root `NOTICE` and is meant for a
compatible `next` runtime, not as a standalone native executable bundle.
Inspect exposes only the source-neutral stable-ID composition; diff treats a
valid difference as `PASS` and uses `different` plus categorized changes.

The authoring and package formats are current-only pre-v1 data. The tools
reject retired versions, linked/escaped input, changed package inventory and an
existing package destination rather than migrating, overwriting or rewriting
them.
