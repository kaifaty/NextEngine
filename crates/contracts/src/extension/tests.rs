use super::{
    EXTENSION_CIRCUIT_WINDOW_TICKS, ExtensionBudgetPolicyV1, ExtensionPackageStateV1,
    LuauPackageManifestV1,
};
use crate::canonical::CanonicalDecodeLimits;
use crate::ids::{CapabilityId, ContentHash, MechanicPackageId, SchemaId};

#[test]
fn circuit_window_and_package_state_round_trip_are_exact() {
    let manifest = manifest();
    let mut state = ExtensionPackageStateV1::new(&manifest, b"ready".to_vec()).expect("state");
    state.record_violation(0).expect("first");
    state
        .record_violation(EXTENSION_CIRCUIT_WINDOW_TICKS - 1)
        .expect("second inclusive");
    state
        .record_violation(EXTENSION_CIRCUIT_WINDOW_TICKS)
        .expect("oldest expires");
    assert_eq!(
        state.violation_ticks,
        [
            EXTENSION_CIRCUIT_WINDOW_TICKS - 1,
            EXTENSION_CIRCUIT_WINDOW_TICKS
        ]
    );
    state
        .record_violation(EXTENSION_CIRCUIT_WINDOW_TICKS + 1)
        .expect("third opens");
    assert_eq!(
        state.circuit_opened_at_tick,
        Some(EXTENSION_CIRCUIT_WINDOW_TICKS + 1)
    );
    let bytes = state.canonical_bytes().expect("bytes");
    assert_eq!(
        ExtensionPackageStateV1::from_canonical_bytes(&bytes, CanonicalDecodeLimits::default())
            .expect("decode"),
        state
    );
}

fn manifest() -> LuauPackageManifestV1 {
    LuauPackageManifestV1::new(
        MechanicPackageId::new("org.nextengine.test.luau").expect("package"),
        1,
        ContentHash::from_bytes([1; 32]),
        vec![CapabilityId::new("mechanics.effect.propose").expect("capability")],
        SchemaId::new("org.nextengine.test.state").expect("schema"),
        1,
        false,
        ExtensionBudgetPolicyV1::new(10, 100, 1_000, 3, 1, 100).expect("budget"),
    )
    .expect("manifest")
}
