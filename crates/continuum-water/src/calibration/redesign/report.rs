#![forbid(unsafe_code)]

use std::fs::OpenOptions;
use std::io::Write;
use std::path::Path;

use crate::error::{AUDIT_INVALID, SCENARIO_INVALID, WaterError};
use crate::oracle::command::validate_report_capacity;

use super::{RedesignEnvelope, RedesignReport};

pub(super) fn write_report(
    output: &Path,
    report: &RedesignReport,
    survived: bool,
) -> Result<(), WaterError> {
    let envelope = RedesignEnvelope {
        schema_version: 1,
        status: if survived { "REPORT_ONLY" } else { "FAIL" },
        command: "continuum water evaluate-hydro-redesign",
        details: report,
    };
    let mut bytes = serde_json::to_vec_pretty(&envelope).map_err(|error| {
        WaterError::new(
            AUDIT_INVALID,
            format!("cannot serialize hydro redesign report: {error}"),
        )
    })?;
    bytes.push(b'\n');
    validate_report_capacity(bytes.len())?;
    let mut file = OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(output)
        .map_err(|error| {
            WaterError::new(
                SCENARIO_INVALID,
                format!(
                    "cannot create redesign report {}: {error}",
                    output.display()
                ),
            )
        })?;
    file.write_all(&bytes).map_err(|error| {
        WaterError::new(
            SCENARIO_INVALID,
            format!("cannot write redesign report {}: {error}", output.display()),
        )
    })?;
    file.sync_all().map_err(|error| {
        WaterError::new(
            SCENARIO_INVALID,
            format!("cannot sync redesign report {}: {error}", output.display()),
        )
    })
}
