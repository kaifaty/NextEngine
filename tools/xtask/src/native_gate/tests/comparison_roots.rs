use super::*;

#[test]
fn every_comparable_root_is_checked_in_declared_order() {
    type Mutator = fn(&mut NativeGateComparableRootsV1);
    let cases: [(&str, Mutator); 27] = [
        ("project_composition_lock_hash", |r| {
            r.project_composition_lock_hash = hash('f');
        }),
        ("schema_registry_hash", |r| {
            r.schema_registry_hash = hash('f');
        }),
        ("content_manifest_hash", |r| {
            r.content_manifest_hash = hash('f');
        }),
        ("mechanics_lock_hash", |r| {
            r.mechanics_lock_hash = hash('f');
        }),
        ("world_partition_hash", |r| {
            r.world_partition_hash = hash('f');
        }),
        ("luau_manifest_hash", |r| r.luau_manifest_hash = hash('f')),
        ("wasm_manifest_hash", |r| r.wasm_manifest_hash = hash('f')),
        ("wit_v2_hash", |r| r.wit_v2_hash = hash('f')),
        ("wit_v3_hash", |r| r.wit_v3_hash = hash('f')),
        ("extension_compatibility_hash", |r| {
            r.extension_compatibility_hash = hash('f');
        }),
        ("play_state_root", |r| r.play_state_root = hash('f')),
        ("play_ledger_hash", |r| r.play_ledger_hash = hash('f')),
        ("replay_state_root", |r| r.replay_state_root = hash('f')),
        ("replay_ledger_hash", |r| r.replay_ledger_hash = hash('f')),
        ("platform_state_root", |r| r.platform_state_root = hash('f')),
        ("platform_ledger_hash", |r| {
            r.platform_ledger_hash = hash('f');
        }),
        ("presentation_snapshot_hash", |r| {
            r.presentation_snapshot_hash = hash('f');
        }),
        ("streaming_performance_hash", |r| {
            r.streaming_performance_hash = hash('f');
        }),
        ("agent_performance_hash", |r| {
            r.agent_performance_hash = hash('f');
        }),
        ("audio_scene_pcm_digest", |r| {
            r.audio_scene_pcm_digest = hash('f');
        }),
        ("packaged_game_state_root", |r| {
            r.packaged_game_state_root = hash('f');
        }),
        ("packaged_game_ledger_hash", |r| {
            r.packaged_game_ledger_hash = hash('f');
        }),
        ("packaged_headless_state_root", |r| {
            r.packaged_headless_state_root = hash('f');
        }),
        ("packaged_headless_ledger_hash", |r| {
            r.packaged_headless_ledger_hash = hash('f');
        }),
        ("closure_hash", |r| r.closure_hash = hash('f')),
        ("windows_package_descriptor_hash", |r| {
            r.windows_package_descriptor_hash = hash('f');
        }),
        ("linux_package_descriptor_hash", |r| {
            r.linux_package_descriptor_hash = hash('f');
        }),
    ];

    for (field, mutate) in cases {
        let windows = report(WINDOWS_TARGET_TRIPLE);
        let mut linux = report(LINUX_TARGET_TRIPLE);
        let roots = linux.comparable_roots.as_mut().expect("roots");
        mutate(roots);
        match field {
            "project_composition_lock_hash" => {
                linux.package.as_mut().expect("package").project_lock_sha256 =
                    roots.project_composition_lock_hash.clone();
            }
            "schema_registry_hash" => {
                linux
                    .package
                    .as_mut()
                    .expect("package")
                    .schema_registry_sha256 = roots.schema_registry_hash.clone();
            }
            "content_manifest_hash" => {
                linux
                    .package
                    .as_mut()
                    .expect("package")
                    .content_manifest_sha256 = roots.content_manifest_hash.clone();
            }
            "mechanics_lock_hash" => {
                linux
                    .package
                    .as_mut()
                    .expect("package")
                    .mechanics_lock_sha256 = roots.mechanics_lock_hash.clone();
            }
            "world_partition_hash" => {
                linux
                    .package
                    .as_mut()
                    .expect("package")
                    .world_partition_sha256 = roots.world_partition_hash.clone();
            }
            "packaged_game_state_root" => {
                linux.package.as_mut().expect("package").game.state_root =
                    roots.packaged_game_state_root.clone();
            }
            "packaged_game_ledger_hash" => {
                linux.package.as_mut().expect("package").game.ledger_hash =
                    roots.packaged_game_ledger_hash.clone();
            }
            "packaged_headless_state_root" => {
                linux.package.as_mut().expect("package").headless.state_root =
                    roots.packaged_headless_state_root.clone();
            }
            "packaged_headless_ledger_hash" => {
                linux
                    .package
                    .as_mut()
                    .expect("package")
                    .headless
                    .ledger_hash = roots.packaged_headless_ledger_hash.clone();
            }
            "windows_package_descriptor_hash" => {
                linux
                    .closure_targets
                    .as_mut()
                    .expect("targets")
                    .windows
                    .package_descriptor_hash = roots.windows_package_descriptor_hash.clone();
            }
            "linux_package_descriptor_hash" => {
                linux
                    .closure_targets
                    .as_mut()
                    .expect("targets")
                    .linux
                    .package_descriptor_hash = roots.linux_package_descriptor_hash.clone();
            }
            _ => {}
        }

        if !matches!(
            field,
            "closure_hash" | "windows_package_descriptor_hash" | "linux_package_descriptor_hash"
        ) {
            roots.windows_package_descriptor_hash =
                target_package_descriptor_hash(WINDOWS_TARGET_TRIPLE, roots);
            roots.linux_package_descriptor_hash =
                target_package_descriptor_hash(LINUX_TARGET_TRIPLE, roots);
            roots.closure_hash = closure_hash(roots);
            let targets = linux.closure_targets.as_mut().expect("targets");
            targets.windows.package_descriptor_hash = roots.windows_package_descriptor_hash.clone();
            targets.linux.package_descriptor_hash = roots.linux_package_descriptor_hash.clone();
        }

        let error = comparison_error(&windows, &linux);
        if matches!(
            field,
            "windows_package_descriptor_hash" | "linux_package_descriptor_hash"
        ) {
            assert_eq!(error.code(), NATIVE_GATE_PACKAGE_INVALID, "{field}");
            assert!(
                error.detail().contains("package descriptor"),
                "{field}: {error}"
            );
        } else {
            assert_eq!(error.code(), NATIVE_GATE_ROOT_MISMATCH, "{field}");
            assert!(error.detail().contains(field), "{field}: {error}");
        }
    }
}
