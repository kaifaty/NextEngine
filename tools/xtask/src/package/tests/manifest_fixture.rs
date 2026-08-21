use super::super::*;

pub(super) fn fixture_manifest() -> PackageManifestV6 {
    let project_lock = "1".repeat(64);
    let state = "2".repeat(64);
    let ledger = "3".repeat(64);
    let game_hash = "4".repeat(64);
    let headless_hash = "5".repeat(64);
    let tool_hash = "a".repeat(64);
    PackageManifestV6 {
        binaries: PackageBinariesV3 {
            game: PackagedRunV2 {
                authoritative_state_root: state.clone(),
                binary_path: "bin/next_game".to_owned(),
                binary_sha256: game_hash.clone(),
                command_ledger_hash: ledger.clone(),
                composition_root: "Game".to_owned(),
                launch_status: "PASS".to_owned(),
                project_composition_lock_hash: project_lock.clone(),
            },
            headless: PackagedRunV2 {
                authoritative_state_root: state,
                binary_path: "bin/next_headless".to_owned(),
                binary_sha256: headless_hash.clone(),
                command_ledger_hash: ledger,
                composition_root: "Headless".to_owned(),
                launch_status: "PASS".to_owned(),
                project_composition_lock_hash: project_lock.clone(),
            },
            tools: PackagedToolValidationV1 {
                authoring_sha256: "b".repeat(64),
                binary_path: "bin/next".to_owned(),
                binary_sha256: tool_hash.clone(),
                command: "project.validate".to_owned(),
                content_entry_count: 2,
                launch_status: "PASS".to_owned(),
                neutral_record_count: 3,
                project_id: "reference-alpha".to_owned(),
                project_composition_lock_hash: project_lock.clone(),
                project_revision: 1,
                publication_file_count: 4,
                publication_state: "validated-not-written".to_owned(),
                render_asset_count: 5,
                root_asset_count: 1,
                source_project_path: "source/reference-alpha".to_owned(),
                world_chunk_count: 1,
            },
        },
        distribution: PackageDistributionV1 {
            cargo_lock_path: "Cargo.lock".to_owned(),
            cargo_lock_sha256: "c".repeat(64),
            dependency_count: 1,
            dependency_inventory_path: "DEPENDENCY_INVENTORY.jcs".to_owned(),
            dependency_inventory_sha256: "d".repeat(64),
            getting_started_path: "GETTING_STARTED.md".to_owned(),
            license_file_count: 1,
            protected_data_scan: PackageProtectedDataScanV1 {
                scanned_byte_count: 1,
                scanned_file_count: 1,
                scanner_id: "nextengine-protected-data-v1".to_owned(),
                status: "PASS".to_owned(),
            },
            release_name: "nextengine".to_owned(),
            release_version: env!("CARGO_PKG_VERSION").to_owned(),
            troubleshooting_path: "TROUBLESHOOTING.md".to_owned(),
        },
        file_inventory: vec![
            PackageFileV2 {
                path: "bin/next".to_owned(),
                sha256: tool_hash,
                size_bytes: 12,
            },
            PackageFileV2 {
                path: "bin/next_game".to_owned(),
                sha256: game_hash,
                size_bytes: 10,
            },
            PackageFileV2 {
                path: "bin/next_headless".to_owned(),
                sha256: headless_hash,
                size_bytes: 11,
            },
        ],
        required_notices: required_notice_paths(),
        runtime_profile: PackageRuntimeProfileV3 {
            abi: PackageRuntimeAbiV3::LinuxGnuX64 {
                minimum_glibc: "2.35".to_owned(),
            },
            binaries: vec![
                PackageBinaryRuntimeV3 {
                    binary_path: "bin/next".to_owned(),
                    direct_libraries: vec!["libc.so.6".to_owned()],
                    maximum_required_glibc: Some("2.35".to_owned()),
                },
                PackageBinaryRuntimeV3 {
                    binary_path: "bin/next_game".to_owned(),
                    direct_libraries: vec!["libc.so.6".to_owned()],
                    maximum_required_glibc: Some("2.35".to_owned()),
                },
                PackageBinaryRuntimeV3 {
                    binary_path: "bin/next_headless".to_owned(),
                    direct_libraries: vec!["libc.so.6".to_owned()],
                    maximum_required_glibc: Some("2.35".to_owned()),
                },
            ],
            external_prerequisites: Vec::new(),
        },
        schema_version: PACKAGE_MANIFEST_SCHEMA_VERSION,
        target_neutral_roots: PackageTargetNeutralRootsV3 {
            content_manifest_sha256: "6".repeat(64),
            mechanics_lock_sha256: "7".repeat(64),
            project_lock_sha256: project_lock,
            schema_registry_sha256: "8".repeat(64),
            world_partition_sha256: "9".repeat(64),
        },
        target_triple: "x86_64-unknown-linux-gnu".to_owned(),
    }
}
