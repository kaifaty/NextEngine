use serde::{Deserialize, Serialize};

use super::sha256_hex;

pub const LOGICAL_RESOURCE_CHARGES_SCHEMA_VERSION: u32 = 1;
pub const LOGICAL_RESOURCE_ACCOUNTING_PROFILE_ID: &str =
    "nextengine.performance.logical-resource-charges.v1";

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PerformanceLogicalResourceChargesV1 {
    pub schema_version: u32,
    pub accounting_profile_id: String,
    pub accounting_profile_hash: String,
    pub authoritative_state_bytes: u64,
    pub required_staging_bytes: u64,
    pub reconstructible_host_cache_bytes: u64,
    pub reconstructible_device_cache_bytes: u64,
    pub presentation_transient_bytes: u64,
    pub tooling_transient_bytes: u64,
    pub total_host_charged_bytes: u64,
    pub total_device_charged_bytes: u64,
    pub charge_root_sha256: String,
}

impl PerformanceLogicalResourceChargesV1 {
    #[allow(
        clippy::too_many_arguments,
        reason = "the constructor covers the complete closed logical charge class set"
    )]
    pub fn new(
        accounting_profile_hash: impl Into<String>,
        authoritative_state_bytes: u64,
        required_staging_bytes: u64,
        reconstructible_host_cache_bytes: u64,
        reconstructible_device_cache_bytes: u64,
        presentation_transient_bytes: u64,
        tooling_transient_bytes: u64,
    ) -> Result<Self, String> {
        let total_host_charged_bytes = authoritative_state_bytes
            .checked_add(required_staging_bytes)
            .and_then(|total| total.checked_add(reconstructible_host_cache_bytes))
            .and_then(|total| total.checked_add(presentation_transient_bytes))
            .and_then(|total| total.checked_add(tooling_transient_bytes))
            .ok_or_else(|| "PERF_LOGICAL_RESOURCE_CHARGE_OVERFLOW".to_owned())?;
        let mut charges = Self {
            schema_version: LOGICAL_RESOURCE_CHARGES_SCHEMA_VERSION,
            accounting_profile_id: LOGICAL_RESOURCE_ACCOUNTING_PROFILE_ID.to_owned(),
            accounting_profile_hash: accounting_profile_hash.into(),
            authoritative_state_bytes,
            required_staging_bytes,
            reconstructible_host_cache_bytes,
            reconstructible_device_cache_bytes,
            presentation_transient_bytes,
            tooling_transient_bytes,
            total_host_charged_bytes,
            total_device_charged_bytes: reconstructible_device_cache_bytes,
            charge_root_sha256: String::new(),
        };
        charges.charge_root_sha256 = charges.canonical_root();
        charges
            .validate()
            .map_err(|diagnostics| diagnostics.join("; "))?;
        Ok(charges)
    }

    pub fn validate(&self) -> Result<(), Vec<String>> {
        let mut diagnostics = Vec::new();
        if self.schema_version != LOGICAL_RESOURCE_CHARGES_SCHEMA_VERSION {
            diagnostics.push("PERF_LOGICAL_RESOURCE_SCHEMA_MISMATCH".to_owned());
        }
        if self.accounting_profile_id != LOGICAL_RESOURCE_ACCOUNTING_PROFILE_ID {
            diagnostics.push("PERF_LOGICAL_RESOURCE_PROFILE_ID_MISMATCH".to_owned());
        }
        if !is_sha256(&self.accounting_profile_hash) {
            diagnostics.push("PERF_LOGICAL_RESOURCE_PROFILE_HASH_INVALID".to_owned());
        }
        let host_total = self
            .authoritative_state_bytes
            .checked_add(self.required_staging_bytes)
            .and_then(|total| total.checked_add(self.reconstructible_host_cache_bytes))
            .and_then(|total| total.checked_add(self.presentation_transient_bytes))
            .and_then(|total| total.checked_add(self.tooling_transient_bytes));
        match host_total {
            Some(total) if total != self.total_host_charged_bytes => {
                diagnostics.push("PERF_LOGICAL_RESOURCE_HOST_TOTAL_MISMATCH".to_owned());
            }
            None => diagnostics.push("PERF_LOGICAL_RESOURCE_CHARGE_OVERFLOW".to_owned()),
            Some(_) => {}
        }
        if self.total_device_charged_bytes != self.reconstructible_device_cache_bytes {
            diagnostics.push("PERF_LOGICAL_RESOURCE_DEVICE_TOTAL_MISMATCH".to_owned());
        }
        if self.charge_root_sha256 != self.canonical_root() {
            diagnostics.push("PERF_LOGICAL_RESOURCE_ROOT_MISMATCH".to_owned());
        }
        finish_validation(diagnostics)
    }

    fn canonical_root(&self) -> String {
        let mut preimage = b"nextengine.performance.logical-resource-charges.v1\0".to_vec();
        append_length_prefixed(&mut preimage, self.accounting_profile_id.as_bytes());
        append_length_prefixed(&mut preimage, self.accounting_profile_hash.as_bytes());
        for value in [
            self.authoritative_state_bytes,
            self.required_staging_bytes,
            self.reconstructible_host_cache_bytes,
            self.reconstructible_device_cache_bytes,
            self.presentation_transient_bytes,
            self.tooling_transient_bytes,
            self.total_host_charged_bytes,
            self.total_device_charged_bytes,
        ] {
            preimage.extend_from_slice(&value.to_le_bytes());
        }
        sha256_hex(&preimage)
    }
}

#[derive(Clone, Debug, Default, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PerformanceResourceCountersV4 {
    pub process_peak_working_set_bytes: Option<u64>,
    pub device_resident_bytes: Option<u64>,
    pub io_read_bytes: Option<u64>,
    pub io_write_bytes: Option<u64>,
    pub logical_resource_charges: Option<PerformanceLogicalResourceChargesV1>,
    pub vulkan_timestamp_queries: u64,
    pub unavailable: Vec<String>,
}

impl PerformanceResourceCountersV4 {
    pub fn validate_report_evidence(&self) -> Result<(), Vec<String>> {
        match &self.logical_resource_charges {
            Some(charges) => charges.validate(),
            None => Ok(()),
        }
    }

    pub fn validate_for_hard_timing(
        &self,
        scenario: super::PerformanceScenarioV1,
    ) -> Result<(), Vec<String>> {
        let mut diagnostics = Vec::new();
        for (name, value) in [
            (
                "process_peak_working_set_bytes",
                self.process_peak_working_set_bytes,
            ),
            ("device_resident_bytes", self.device_resident_bytes),
            ("io_read_bytes", self.io_read_bytes),
            ("io_write_bytes", self.io_write_bytes),
        ] {
            if value.is_none() {
                diagnostics.push(format!("PERF_REQUIRED_COUNTER_MISSING: {name}"));
            }
        }
        if matches!(
            scenario,
            super::PerformanceScenarioV1::InteractiveFrameSoak
                | super::PerformanceScenarioV1::R2AlphaRender
        ) && self.vulkan_timestamp_queries == 0
        {
            diagnostics.push("PERF_REQUIRED_COUNTER_MISSING: vulkan_timestamp_queries".to_owned());
        }
        if !self.unavailable.is_empty() {
            diagnostics.push("PERF_REQUIRED_COUNTER_UNAVAILABLE".to_owned());
            diagnostics.extend(self.unavailable.clone());
        }
        match &self.logical_resource_charges {
            Some(charges) => {
                if let Err(errors) = charges.validate() {
                    diagnostics.extend(errors);
                }
            }
            None => diagnostics
                .push("PERF_REQUIRED_COUNTER_MISSING: logical_resource_charges".to_owned()),
        }
        finish_validation(diagnostics)
    }
}

fn append_length_prefixed(output: &mut Vec<u8>, value: &[u8]) {
    output.extend_from_slice(&(value.len() as u64).to_le_bytes());
    output.extend_from_slice(value);
}

fn finish_validation(mut diagnostics: Vec<String>) -> Result<(), Vec<String>> {
    diagnostics.sort();
    diagnostics.dedup();
    if diagnostics.is_empty() {
        Ok(())
    } else {
        Err(diagnostics)
    }
}

fn is_sha256(value: &str) -> bool {
    value.len() == 64
        && value
            .bytes()
            .all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte))
}
