# Creator Smoke

`creator-smoke` is the independent data-only acceptance project for the first
R6 creator workflow. It is deliberately small, but it exercises the current
authoring closure: character, item, ability, dialogue, quest, relationship,
interaction, three streamed chunks, minimal render content and world services.

From the repository root:

```text
cargo run --locked -p next_cli -- project validate --project projects/creator-smoke
cargo run --locked -p next_cli -- project cook --project projects/creator-smoke --output target/creator-smoke-cooked
```

Both commands emit exactly one Creator Command Report V1 JSON object on
stdout. The project is current-only pre-v1 authoring data; the tools reject
retired versions rather than migrating or rewriting them.
