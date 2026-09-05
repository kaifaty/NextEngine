use super::*;

/// BODY-BANDWIDTH-01: a fixed native diagnostic, not a selected calibrated body.
#[must_use]
pub fn biomechanics_humanoid_body_schema_v11() -> BodySchemaV2 {
    let mut body = biomechanics_humanoid_body_schema_v8();
    body.schema_id = id("nextengine.body.humanoid-biomechanics-raja-1700.v11");
    body.schema_revision = 11;
    body.source_provenance_hash = domain_hash(b"nextengine.source.raja-1700.coupled-bandwidth.v11");
    for a in &mut body.actuators {
        let joint = a.joint_id.as_str();
        let pair = joint
            .strip_prefix("joint.left-")
            .or_else(|| joint.strip_prefix("joint.right-"));
        let (k, d) = match pair.unwrap_or(joint) {
            "hip-pitch" => (9_785_229, 476_871),
            "hip-roll" => (22_937_600, 1_352_316),
            "hip-yaw" => (3_929_139, 191_482),
            "knee" => (8_604_411, 419_325),
            "ankle-pitch" => (988_832, 48_189),
            "ankle-roll" => (258_345, 12_590),
            "mtp" => (27_535, 1_342),
            "shoulder-pitch" => (9_764_453, 475_859),
            "shoulder-roll" => (10_485_760, 947_784),
            "shoulder-yaw" => (479_093, 23_348),
            "elbow" => (1_767_848, 86_154),
            "joint.torso-pitch" => (14_600_182, 711_522),
            "joint.torso-roll" => (19_660_800, 1_285_267),
            "joint.torso-yaw" => (13_253_768, 645_906),
            _ => panic!("unrecognized canonical V11 joint"),
        };
        a.stiffness_q16 = k;
        a.damping_q16 = d;
    }
    body.validate().expect("V11 bandwidth diagnostic is valid");
    body
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn only_gains_and_identity_change_and_material_admission_is_exact() {
        let old = biomechanics_humanoid_body_schema_v8();
        let body = biomechanics_humanoid_body_schema_v11();
        let mut restored = body.clone();
        let mut k_changes = 0;
        for (a, b) in restored.actuators.iter_mut().zip(&old.actuators) {
            assert!(a.stiffness_q16 <= b.stiffness_q16);
            k_changes += usize::from(a.stiffness_q16 != b.stiffness_q16);
            assert_ne!(a.damping_q16, b.damping_q16);
            a.stiffness_q16 = b.stiffness_q16;
            a.damping_q16 = b.damping_q16;
        }
        assert_eq!(k_changes, 20);
        restored.schema_id = old.schema_id.clone();
        restored.schema_revision = old.schema_revision;
        restored.source_provenance_hash = old.source_provenance_hash;
        assert_eq!(restored, old);
        assert_ne!(body.schema_hash().unwrap(), old.schema_hash().unwrap());
        let subject = next_contracts::ids::PersistentId::from_bytes([0; 16]);
        let compiled = crate::CompiledBodySchemaV4::compile(&body, subject).unwrap();
        assert_eq!(
            compiled
                .base
                .physics_descriptors
                .collider_material_assignment_counts[&id(crate::BIOMECHANICS_SOLE_MATERIAL_ID)],
            4
        );
        let mut altered = body.clone();
        altered.actuators[0].stiffness_q16 += 1;
        assert!(crate::CompiledBodySchemaV4::compile(&altered, subject).is_err());
        altered = body;
        altered.schema_id = id("body.renamed-v11");
        assert!(crate::CompiledBodySchemaV4::compile(&altered, subject).is_err());
    }
}
