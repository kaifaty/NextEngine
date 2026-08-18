# Creator Smoke

`creator-smoke` is the independent data-only acceptance project for the public
R6 creator workflow. It is deliberately small, but it exercises the current
authoring closure: character, item, ability, dialogue, quest, relationship,
interaction, three streamed chunks, minimal render content and world services.

From the repository root:

```text
cargo run --locked -p next_cli -- project validate --project projects/creator-smoke
cargo run --locked -p next_cli -- project cook --project projects/creator-smoke --output target/creator-smoke-cooked
cargo run --locked -p next_cli -- project run --project projects/creator-smoke
cargo run --locked -p next_cli -- project package --project projects/creator-smoke --output target/creator-smoke-package
cargo run --locked -p next_cli -- project run --package target/creator-smoke-package
```

Each command emits exactly one versioned JSON object on stdout: Creator Command
Report V1 for validate/cook, Creator Run Report V1 for either run spelling and
Creator Package Report V1 for package. Run executes one project-neutral
production headless tick and ordinary final save-on-close. The package contains
the exact published ContentStore plus root `NOTICE` and is meant for a
compatible `next` runtime, not as a standalone native executable bundle.

The authoring and package formats are current-only pre-v1 data. The tools
reject retired versions, linked/escaped input, changed package inventory and an
existing package destination rather than migrating, overwriting or rewriting
them.
