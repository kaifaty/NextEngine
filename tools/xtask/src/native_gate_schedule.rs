use std::fs;
use std::path::{Path, PathBuf};
use std::time::Instant;

use xtask::native_gate::{NativeGateCheckNameV1, NativeGateCheckRecordV1};

use crate::native_gate_publish::path_exists_without_following;
use crate::{NativeGateCheckExecutionFailure, NativeGateCheckFailure};

pub(crate) struct NativeGateMatrixSchedule<'a> {
    staging: &'a Path,
    next_check: usize,
    records: Vec<NativeGateCheckRecordV1>,
    stopped_error: Option<String>,
}

impl<'a> NativeGateMatrixSchedule<'a> {
    pub(crate) fn new(staging: &'a Path) -> Self {
        Self {
            staging,
            next_check: 0,
            records: Vec::with_capacity(NativeGateCheckNameV1::ORDERED.len()),
            stopped_error: None,
        }
    }

    pub(crate) fn run<T>(
        &mut self,
        check: NativeGateCheckNameV1,
        operation: impl FnOnce(
            &Path,
            Instant,
        ) -> Result<
            (T, NativeGateCheckRecordV1),
            Box<NativeGateCheckExecutionFailure>,
        >,
    ) -> Result<T, NativeGateCheckFailure> {
        if let Some(error) = &self.stopped_error {
            return Err(NativeGateCheckFailure {
                records: self.records.clone(),
                error: error.clone(),
            });
        }

        let Some(expected) = NativeGateCheckNameV1::ORDERED.get(self.next_check).copied() else {
            return Err(NativeGateCheckFailure {
                records: self.records.clone(),
                error: "NATIVE_GATE_CHECK_FAILED: native gate matrix is already complete"
                    .to_owned(),
            });
        };
        if check != expected {
            let error = format!(
                "NATIVE_GATE_CHECK_FAILED: native gate matrix expected {}, got {}",
                expected.as_str(),
                check.as_str()
            );
            let failure = *crate::native_gate_runner::native_gate_execution_failure(
                expected,
                error,
                Instant::now(),
            );
            return Err(self.stop(failure));
        }

        let state_root = native_gate_state_root(self.staging, check);
        let started = Instant::now();
        if let Err(error) = prepare_native_gate_state_root(&state_root) {
            let failure =
                *crate::native_gate_runner::native_gate_execution_failure(check, error, started);
            return Err(self.stop(failure));
        }

        match operation(&state_root, started) {
            Ok((value, record)) => {
                self.records.push(record);
                self.next_check += 1;
                Ok(value)
            }
            Err(failure) => Err(self.stop(*failure)),
        }
    }

    pub(crate) fn finish(self) -> Result<Vec<NativeGateCheckRecordV1>, NativeGateCheckFailure> {
        if let Some(error) = self.stopped_error {
            return Err(NativeGateCheckFailure {
                records: self.records,
                error,
            });
        }
        if self.next_check == NativeGateCheckNameV1::ORDERED.len() {
            return Ok(self.records);
        }

        let expected = NativeGateCheckNameV1::ORDERED[self.next_check];
        let error = format!(
            "NATIVE_GATE_CHECK_FAILED: native gate matrix stopped before {}",
            expected.as_str()
        );
        let failure = *crate::native_gate_runner::native_gate_execution_failure(
            expected,
            error,
            Instant::now(),
        );
        let mut records = self.records;
        records.push(failure.record);
        Err(NativeGateCheckFailure {
            records,
            error: failure.error,
        })
    }

    fn stop(&mut self, failure: NativeGateCheckExecutionFailure) -> NativeGateCheckFailure {
        self.records.push(failure.record);
        self.stopped_error = Some(failure.error.clone());
        NativeGateCheckFailure {
            records: self.records.clone(),
            error: failure.error,
        }
    }
}

pub(crate) fn native_gate_state_root(staging: &Path, check: NativeGateCheckNameV1) -> PathBuf {
    staging.join(".state").join(check.as_str())
}

fn prepare_native_gate_state_root(state_root: &Path) -> Result<(), String> {
    if path_exists_without_following(state_root)? {
        return Err(format!(
            "NATIVE_GATE_REPORT_INVALID: check state root already exists: {}",
            state_root.display()
        ));
    }
    fs::create_dir(state_root).map_err(|error| {
        format!(
            "NATIVE_GATE_REPORT_INVALID: failed to create check state root {}: {error}",
            state_root.display()
        )
    })
}
