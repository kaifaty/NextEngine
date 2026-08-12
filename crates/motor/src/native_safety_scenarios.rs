use serde_json::Value;

use crate::{
    BiomechanicsContactClassV1, BiomechanicsTerminalReasonV1,
    PROCEDURAL_STANDING_SCENARIO_MOTOR_TICKS, biomechanics_native_safety_review_json_v1,
};

#[test]
fn native_procedural_standing_reaches_exact_thirty_second_timeout() {
    let text = biomechanics_native_safety_review_json_v1().expect("native safety review");
    let review: Value = serde_json::from_str(&text).expect("valid review JSON");
    assert_eq!(review["status"], "PASS");
    assert_eq!(
        review["samples"]
            .as_array()
            .expect("samples")
            .iter()
            .map(|sample| sample["motor_tick"].as_u64().expect("motor tick"))
            .collect::<Vec<_>>(),
        [0, 600, 1_200, PROCEDURAL_STANDING_SCENARIO_MOTOR_TICKS]
    );
    let final_sample = review["samples"]
        .as_array()
        .expect("samples")
        .last()
        .expect("final sample");
    assert_eq!(final_sample["terminal"]["disposition"], 2);
    assert_eq!(
        final_sample["terminal"]["reason"],
        BiomechanicsTerminalReasonV1::Timeout as u8
    );
    assert_eq!(final_sample["joints"].as_array().map(Vec::len), Some(23));
    for sample in review["samples"].as_array().expect("samples") {
        for contact in sample["classified_contacts"]
            .as_array()
            .expect("classified contacts")
        {
            assert_eq!(contact["primary_role"], 8);
            assert_eq!(
                contact["class"],
                BiomechanicsContactClassV1::SoleSupport as u8
            );
            assert_eq!(contact["hard_impact_violation"], false);
        }
    }
}
