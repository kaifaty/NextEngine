use super::*;
use crate::rpg::RPG_COMMAND_CAPABILITY_ID;

#[test]
fn invalid_transition_graph_is_rejected() {
    let transition_a = StateTransitionV1 {
        transition_id: SchemaId::new("nextengine.test.transition.a").expect("ID"),
        source_state_id: SchemaId::new("nextengine.test.state.entry").expect("ID"),
        target_state_id: SchemaId::new("nextengine.test.state.a").expect("ID"),
    };
    let transition_b = StateTransitionV1 {
        transition_id: SchemaId::new("nextengine.test.transition.b").expect("ID"),
        source_state_id: SchemaId::new("nextengine.test.state.entry").expect("ID"),
        target_state_id: SchemaId::new("nextengine.test.state.b").expect("ID"),
    };

    assert_eq!(
        validate_transitions(
            &[transition_a, transition_b],
            &SchemaId::new("nextengine.test.state.entry").expect("ID"),
        ),
        Err(MechanicsContractError::InvalidTransitionGraph)
    );
}

#[test]
fn package_capability_denial_is_rejected_before_activation() {
    let manifest = package_manifest();
    let lock = MechanicsLockV1::new(vec![LockedMechanicPackageV1 {
        package_id: manifest.package_id.clone(),
        package_manifest_sha256: manifest.package_manifest_sha256,
        granted_capabilities: Vec::new(),
    }])
    .expect("lock");

    assert_eq!(
        RpgDefinitionRegistryV2::new(
            Vec::new(),
            Vec::new(),
            Vec::new(),
            Vec::new(),
            Vec::new(),
            vec![manifest],
            lock,
        ),
        Err(MechanicsContractError::CapabilityDenied)
    );
}

#[test]
fn package_hash_mismatch_is_rejected_before_activation() {
    let manifest = package_manifest();
    let lock = MechanicsLockV1::new(vec![LockedMechanicPackageV1 {
        package_id: manifest.package_id.clone(),
        package_manifest_sha256: ContentHash::from_bytes([0x7f; 32]),
        granted_capabilities: vec![
            CapabilityId::new(RPG_COMMAND_CAPABILITY_ID).expect("capability"),
        ],
    }])
    .expect("lock");

    assert_eq!(
        RpgDefinitionRegistryV2::new(
            Vec::new(),
            Vec::new(),
            Vec::new(),
            Vec::new(),
            Vec::new(),
            vec![manifest],
            lock,
        ),
        Err(MechanicsContractError::PackageLockMismatch)
    );
}

#[test]
fn mutated_grant_order_invalidates_lock() {
    let manifest = package_manifest();
    let mut lock = MechanicsLockV1::new(vec![LockedMechanicPackageV1 {
        package_id: manifest.package_id,
        package_manifest_sha256: manifest.package_manifest_sha256,
        granted_capabilities: vec![
            CapabilityId::new("nextengine.capability.z").expect("capability"),
            CapabilityId::new("nextengine.capability.a").expect("capability"),
        ],
    }])
    .expect("lock");
    lock.packages[0].granted_capabilities.swap(0, 1);

    assert_eq!(lock.validate(), Err(MechanicsContractError::HashMismatch));
}

fn package_manifest() -> MechanicPackageManifestV1 {
    MechanicPackageManifestV1::new(
        MechanicPackageId::new("org.nextengine.test.package").expect("package ID"),
        1,
        vec![CapabilityId::new(RPG_COMMAND_CAPABILITY_ID).expect("capability")],
        Vec::new(),
        Vec::new(),
    )
    .expect("manifest")
}
