use super::*;

/// Fixed BODY-GAIN-02 candidates, quantized once to Q16 for native diagnostics.
/// No inference of implicit-drive stability for the canonical explicit PD.
#[must_use]
pub fn biomechanics_humanoid_body_schema_v10() -> BodySchemaV2 {
    let mut schema = biomechanics_humanoid_body_schema_v8();
    schema.schema_id = id("nextengine.body.humanoid-biomechanics-raja-1700.v10");
    schema.schema_revision = 10;
    schema.source_provenance_hash =
        domain_hash(b"nextengine.source.raja-1700.screened-eight-damping.v10");
    for actuator in &mut schema.actuators {
        actuator.damping_q16 = match actuator.joint_id.as_str() {
            "joint.left-ankle-pitch" | "joint.right-ankle-pitch" => 319_360,
            "joint.left-ankle-roll" | "joint.right-ankle-roll" => 118_376,
            "joint.left-shoulder-yaw" => 26_503,
            "joint.right-shoulder-yaw" => 26_504,
            "joint.left-elbow" | "joint.right-elbow" => 420_501,
            _ => actuator.damping_q16,
        };
    }
    schema
        .validate()
        .expect("V10 screened damping diagnostic is valid");
    schema
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn screened_damping_changes_only_eight_gains_and_identity() {
        let old = biomechanics_humanoid_body_schema_v8();
        let mut new = biomechanics_humanoid_body_schema_v10();
        assert_ne!(new.schema_hash().unwrap(), old.schema_hash().unwrap());
        let mut changed = 0;
        for (actual, original) in new.actuators.iter_mut().zip(&old.actuators) {
            let source_si = match actual.joint_id.as_str() {
                "joint.left-ankle-pitch" | "joint.right-ankle-pitch" => Some(4.873_045_444_488_525),
                "joint.left-ankle-roll" | "joint.right-ankle-roll" => Some(1.806_270_837_783_813_5),
                "joint.left-shoulder-yaw" => Some(0.404_399_216_175_079_35),
                "joint.right-shoulder-yaw" => Some(0.404_422_909_021_377_56),
                "joint.left-elbow" | "joint.right-elbow" => Some(6.416_338_920_593_262),
                _ => None,
            };
            if let Some(si) = source_si {
                assert_eq!(
                    actual.damping_q16,
                    (si * 65_536.0_f64).round_ties_even() as u64
                );
                assert_ne!(actual.damping_q16, original.damping_q16);
                changed += 1;
            } else {
                assert_eq!(actual.damping_q16, original.damping_q16);
            }
            actual.damping_q16 = original.damping_q16;
        }
        assert_eq!(changed, 8);
        new.schema_id = old.schema_id.clone();
        new.schema_revision = old.schema_revision;
        new.source_provenance_hash = old.source_provenance_hash;
        assert_eq!(new, old);
    }

    #[test]
    fn screened_damping_material_admission_is_exact() {
        let body = biomechanics_humanoid_body_schema_v10();
        let subject = next_contracts::ids::PersistentId::from_bytes([0; 16]);
        let compiled = crate::CompiledBodySchemaV4::compile(&body, subject).unwrap();
        assert_eq!(
            compiled
                .base
                .physics_descriptors
                .collider_material_assignment_counts[&id(crate::BIOMECHANICS_SOLE_MATERIAL_ID)],
            4
        );
        let mut changed = body.clone();
        changed.actuators[0].damping_q16 += 1;
        assert!(crate::CompiledBodySchemaV4::compile(&changed, subject).is_err());
        changed = body;
        changed.schema_id = id("body.renamed-v10");
        assert!(crate::CompiledBodySchemaV4::compile(&changed, subject).is_err());
        let descriptor: serde_json::Value = serde_json::from_str(
            &crate::biomechanics_body_diagnostic_descriptor_json_v10().unwrap(),
        )
        .unwrap();
        assert_eq!(
            descriptor["backend_admission"],
            "native-body-diagnostic-only"
        );
        assert_eq!(descriptor["body_schema_revision"], 10);
    }
}
