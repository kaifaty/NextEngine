# Creator SDK examples

These files are the source-level Luau and Wasm examples used by the production
extension hosts and the `content-package` ProductCheck:

- `luau/scripted-melee.lua` proposes the same semantic melee action used by the
  data-first mechanics path;
- `wasm/increment-component.wat` is the bounded reference Wasm Component.

Start with the complete [Creator SDK beta
guide](../../docs/creator-sdk.md). The current WIT world is
[`nextengine-extension.wit`](../../crates/plugin-host/wit/v3/nextengine-extension.wit).

These examples do not grant ambient filesystem/network/WASI access and do not
create a second gameplay mutation path. Their outcomes still pass the common
capability, mechanics and `WorldCommand` validators.
