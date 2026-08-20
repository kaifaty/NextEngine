use super::*;

pub(super) fn package_manifest_from_summary(
    report: &NativeGateTargetReportV1,
) -> crate::package::PackageManifestV5 {
    let summary = report.package.as_ref().expect("package summary");
    let roots = crate::package::PackageTargetNeutralRootsV3 {
        content_manifest_sha256: summary.content_manifest_sha256.clone(),
        mechanics_lock_sha256: summary.mechanics_lock_sha256.clone(),
        project_lock_sha256: summary.project_lock_sha256.clone(),
        schema_registry_sha256: summary.schema_registry_sha256.clone(),
        world_partition_sha256: summary.world_partition_sha256.clone(),
    };
    let packaged_run = |composition_root: &str,
                        binary_path: &str,
                        binary_sha256: &str,
                        launch: &NativeGatePackagedLaunchSummaryV1| {
        crate::package::PackagedRunV2 {
            authoritative_state_root: launch.state_root.clone(),
            binary_path: binary_path.to_owned(),
            binary_sha256: binary_sha256.to_owned(),
            command_ledger_hash: launch.ledger_hash.clone(),
            composition_root: composition_root.to_owned(),
            launch_status: "PASS".to_owned(),
            project_composition_lock_hash: summary.project_lock_sha256.clone(),
        }
    };
    crate::package::PackageManifestV5 {
        binaries: crate::package::PackageBinariesV3 {
            game: packaged_run(
                "Game",
                "bin/next_game.exe",
                &summary.game_binary_sha256,
                &summary.game,
            ),
            headless: packaged_run(
                "Headless",
                "bin/next_headless.exe",
                &summary.headless_binary_sha256,
                &summary.headless,
            ),
            tools: crate::package::PackagedToolRunV1 {
                authoritative_state_root: "a".repeat(64),
                binary_path: "bin/next.exe".to_owned(),
                binary_sha256: "b".repeat(64),
                close_receipt_hash: "c".repeat(64),
                command: "project.run".to_owned(),
                command_ledger_hash: "d".repeat(64),
                final_save_generation_hash: "e".repeat(64),
                launch_status: "PASS".to_owned(),
                project_composition_lock_hash: summary.project_lock_sha256.clone(),
                source: "authoring".to_owned(),
                source_project_path: "source/reference-alpha".to_owned(),
                ticks: 1,
            },
        },
        file_inventory: Vec::new(),
        required_notices: Vec::new(),
        runtime_profile: crate::package::PackageRuntimeProfileV3 {
            abi: crate::package::PackageRuntimeAbiV3::WindowsMsvcX64 {
                crt: crate::package::PackageWindowsCrtV3::DynamicSystem,
            },
            binaries: Vec::new(),
            external_prerequisites: Vec::new(),
        },
        schema_version: crate::package::PACKAGE_MANIFEST_SCHEMA_VERSION,
        target_neutral_roots: roots,
        target_triple: report.target_triple.clone(),
    }
}
