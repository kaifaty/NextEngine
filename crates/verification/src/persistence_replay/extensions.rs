use next_contracts::ContentHash;

use super::PersistenceReplayCheckError;

pub(super) fn verify_wasm_state_round_trip() -> Result<ContentHash, PersistenceReplayCheckError> {
    let manifest = next_plugin_host::reference_wasm_manifest_v1(true).map_err(|error| {
        PersistenceReplayCheckError::new("create Wasm manifest", error.to_string())
    })?;
    let component = next_plugin_host::REFERENCE_COMPONENT_WAT
        .as_bytes()
        .to_vec();
    let grants = manifest.requested_capabilities.clone();
    let startup = next_plugin_host::WasmPluginRuntimeV1::activate(
        manifest.clone(),
        Some(component.clone()),
        grants.clone(),
    )
    .map_err(|error| PersistenceReplayCheckError::new("create Wasm runtime", error.to_string()))?;
    let next_plugin_host::WasmPluginStartupV1::Active(mut direct) = startup else {
        return Err(PersistenceReplayCheckError::condition(
            "required Wasm plugin activates",
        ));
    };
    let first = direct.execute_i32(1, 0).map_err(|error| {
        PersistenceReplayCheckError::new("execute Wasm before save", error.to_string())
    })?;
    let bytes = direct.state().canonical_bytes().map_err(|error| {
        PersistenceReplayCheckError::new("encode Wasm plugin state", error.to_string())
    })?;
    let state = next_contracts::WasmPluginStateV1::from_canonical_bytes(
        &bytes,
        next_contracts::CanonicalDecodeLimits::default(),
    )
    .map_err(|error| {
        PersistenceReplayCheckError::new("decode Wasm plugin state", error.to_string())
    })?;
    let mut restored =
        next_plugin_host::WasmPluginRuntimeV1::restore(manifest, component, grants, state)
            .map_err(|error| {
                PersistenceReplayCheckError::new("restore Wasm runtime", error.to_string())
            })?;
    let retried = restored.execute_i32(2, 0).map_err(|error| {
        PersistenceReplayCheckError::new("execute Wasm after load", error.to_string())
    })?;
    if first.value != retried.value || direct.state().state_bytes != restored.state().state_bytes {
        return Err(PersistenceReplayCheckError::condition(
            "Wasm plugin state and retry are exact after load",
        ));
    }
    Ok(restored.state().state_hash)
}

pub(super) fn verify_luau_state_round_trip() -> Result<ContentHash, PersistenceReplayCheckError> {
    let manifest = next_script_luau::reference_scripted_melee_manifest_v1().map_err(|error| {
        PersistenceReplayCheckError::new("create Luau manifest", error.to_string())
    })?;
    let source = next_script_luau::REFERENCE_SCRIPTED_MELEE_SOURCE
        .as_bytes()
        .to_vec();
    let granted_capabilities = manifest.requested_capabilities.clone();
    let mut direct = next_script_luau::LuauPackageRuntimeV1::new(manifest.clone(), source.clone())
        .map_err(|error| {
            PersistenceReplayCheckError::new("create Luau runtime", error.to_string())
        })?;
    let input = |gameplay_tick| next_script_luau::LuauCallbackInputV1 {
        gameplay_tick,
        target_health: 100,
        granted_capabilities: granted_capabilities.clone(),
    };
    let first = direct.execute(input(1)).map_err(|error| {
        PersistenceReplayCheckError::new("execute Luau before save", error.to_string())
    })?;
    let bytes = direct.state().canonical_bytes().map_err(|error| {
        PersistenceReplayCheckError::new("encode Luau package state", error.to_string())
    })?;
    let state = next_contracts::ExtensionPackageStateV1::from_canonical_bytes(
        &bytes,
        next_contracts::CanonicalDecodeLimits::default(),
    )
    .map_err(|error| {
        PersistenceReplayCheckError::new("decode Luau package state", error.to_string())
    })?;
    let mut restored = next_script_luau::LuauPackageRuntimeV1::restore(manifest, source, state)
        .map_err(|error| {
            PersistenceReplayCheckError::new("restore Luau runtime", error.to_string())
        })?;
    let retried = restored.execute(input(2)).map_err(|error| {
        PersistenceReplayCheckError::new("execute Luau after load", error.to_string())
    })?;
    if first.proposed_semantic_actions != retried.proposed_semantic_actions
        || direct.state().state_bytes != restored.state().state_bytes
    {
        return Err(PersistenceReplayCheckError::condition(
            "Luau package state and retry are exact after load",
        ));
    }
    Ok(restored.state().state_hash)
}
