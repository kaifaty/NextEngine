use super::*;

/// Opt-in BODY-DAMPING-01 diagnostic; no training/default compatibility.
#[must_use]
pub fn biomechanics_humanoid_body_schema_v9() -> BodySchemaV2 {
    let mut schema = biomechanics_humanoid_body_schema_v8();
    schema.schema_id = id("nextengine.body.humanoid-biomechanics-raja-1700.v9");
    schema.schema_revision = 9;
    schema.source_provenance_hash =
        domain_hash(b"nextengine.source.raja-1700.stiffness-proportional-damping.v9");
    for actuator in &mut schema.actuators {
        // K Q16 * c seconds -> D Q16. Positive integer ties-to-even; the
        // u64 stiffness product fits u128 even at its type maximum.
        let numerator = u128::from(actuator.stiffness_q16) * 4_831;
        let denominator = 1_789_440;
        let quotient = numerator / denominator;
        let remainder = numerator % denominator;
        let increment =
            2 * remainder > denominator || (2 * remainder == denominator && quotient % 2 == 1);
        actuator.damping_q16 = u64::try_from(quotient + u128::from(increment))
            .expect("canonical stiffness-proportional damping fits u64");
    }
    schema.validate().expect("V9 damping diagnostic is valid");
    schema
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn only_damping_and_identity_change() {
        let old = biomechanics_humanoid_body_schema_v8();
        let mut candidate = biomechanics_humanoid_body_schema_v9();
        assert_ne!(old.schema_hash().unwrap(), candidate.schema_hash().unwrap());
        for (a, b) in candidate.actuators.iter_mut().zip(&old.actuators) {
            // Independent nearest-integer characterization, including ties.
            let error =
                (i128::from(a.damping_q16) * 1_789_440 - i128::from(b.stiffness_q16) * 4_831).abs();
            assert!(2 * error <= 1_789_440);
            if 2 * error == 1_789_440 {
                assert_eq!(a.damping_q16 % 2, 0);
            }
            a.damping_q16 = b.damping_q16;
        }
        candidate.schema_id = old.schema_id.clone();
        candidate.schema_revision = old.schema_revision;
        candidate.source_provenance_hash = old.source_provenance_hash;
        assert_eq!(candidate, old);
    }

    #[test]
    fn compiled_damping_vector_and_exact_material_admission() {
        let body = biomechanics_humanoid_body_schema_v9();
        let subject = next_contracts::ids::PersistentId::from_bytes([0; 16]);
        let compiled = crate::CompiledBodySchemaV4::compile(&body, subject).unwrap();
        let mut by_dof = [0; 25];
        for (a, dof) in compiled
            .base
            .base
            .actuator_definitions
            .iter()
            .zip(&compiled.base.base.actuator_dof_ordinals)
        {
            by_dof[*dof as usize] = a.damping_q16;
        }
        assert_eq!(
            by_dof,
            [
                79618, 61925, 53079, 88465, 70772, 53079, 4423, 79618, 61925, 53079, 88465, 70772,
                53079, 4423, 53079, 53079, 53079, 31847, 28309, 1548, 24770, 31847, 28309, 1548,
                24770
            ]
        );
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
        changed.schema_id = id("body.renamed-v9");
        assert!(crate::CompiledBodySchemaV4::compile(&changed, subject).is_err());
        let descriptor: serde_json::Value = serde_json::from_str(
            &crate::biomechanics_body_diagnostic_descriptor_json_v9().unwrap(),
        )
        .unwrap();
        assert_eq!(
            descriptor["backend_admission"],
            "native-body-diagnostic-only"
        );
        assert_eq!(descriptor["body_schema_revision"], 9);
    }
}
