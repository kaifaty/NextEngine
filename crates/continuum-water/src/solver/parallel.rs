#![forbid(unsafe_code)]

use std::panic::{AssertUnwindSafe, catch_unwind};

use crate::error::{WORKER_FAILURE, WaterError};

use super::*;

const LOGICAL_PARTITION_COUNT: usize = 64;
pub(crate) const MAXIMUM_WORKER_COUNT: usize = 8;

pub(crate) struct DeterministicWorkers {
    worker_count: usize,
    logical_partition_count: usize,
    pool: rayon::ThreadPool,
    #[cfg(test)]
    fault: WorkerFault,
}

#[derive(Clone, Copy, Debug)]
pub(super) struct LogicalPartition {
    pub(super) ordinal: usize,
    pub(super) start: usize,
    pub(super) end: usize,
}

impl DeterministicWorkers {
    pub(crate) fn new(worker_count: usize) -> Result<Self, WaterError> {
        if !(1..=MAXIMUM_WORKER_COUNT).contains(&worker_count) {
            return Err(WaterError::new(
                WORKER_FAILURE,
                format!("worker count must be in 1..={MAXIMUM_WORKER_COUNT}"),
            ));
        }
        let pool = rayon::ThreadPoolBuilder::new()
            .num_threads(worker_count)
            .thread_name(|index| format!("continuum-water-{index}"))
            .build()
            .map_err(|error| {
                WaterError::new(
                    WORKER_FAILURE,
                    format!("cannot create worker pool: {error}"),
                )
            })?;
        Ok(Self {
            worker_count,
            logical_partition_count: LOGICAL_PARTITION_COUNT,
            pool,
            #[cfg(test)]
            fault: WorkerFault::None,
        })
    }

    pub(crate) fn worker_count(&self) -> usize {
        self.worker_count
    }

    pub(crate) const fn logical_partition_count(&self) -> usize {
        self.logical_partition_count
    }

    pub(super) fn try_map<T, F>(
        &self,
        item_count: usize,
        operation: F,
    ) -> Result<Vec<T>, WaterError>
    where
        T: Clone + Default + Send,
        F: Fn(usize) -> Result<T, WaterError> + Send + Sync,
    {
        let partitions = logical_partitions(item_count, self.logical_partition_count)?;
        let mut result = filled_vec(item_count, T::default())?;
        let execution = catch_unwind(AssertUnwindSafe(|| {
            self.pool
                .install(|| fill_index_partitions(self, &partitions, 0, &mut result, &operation))
        }))
        .map_err(|_| WaterError::new(WORKER_FAILURE, "worker panicked before canonical merge"))?;
        execution?;
        Ok(result)
    }

    pub(super) fn try_map_partitions<T, F>(
        &self,
        item_count: usize,
        operation: F,
    ) -> Result<Vec<T>, WaterError>
    where
        T: Send,
        F: Fn(LogicalPartition) -> Result<T, WaterError> + Send + Sync,
    {
        let partitions = logical_partitions(item_count, self.logical_partition_count)?;
        let mut slots = Vec::new();
        slots
            .try_reserve_exact(partitions.len())
            .map_err(heap_error)?;
        slots.resize_with(partitions.len(), || None);
        catch_unwind(AssertUnwindSafe(|| {
            self.pool.install(|| {
                fill_partition_results(self, &partitions, &mut slots, &operation);
            });
        }))
        .map_err(|_| WaterError::new(WORKER_FAILURE, "worker panicked before canonical merge"))?;
        let mut result = Vec::new();
        result.try_reserve_exact(slots.len()).map_err(heap_error)?;
        for slot in slots {
            let fragment = slot.ok_or_else(|| {
                WaterError::new(WORKER_FAILURE, "worker fragment result is missing")
            })??;
            result.push(fragment);
        }
        Ok(result)
    }

    pub(super) fn try_fill_pair<A, B, F>(
        &self,
        first: &mut [A],
        second: &mut [B],
        operation: F,
    ) -> Result<(), WaterError>
    where
        A: Send,
        B: Send,
        F: Fn(usize) -> Result<(A, B), WaterError> + Send + Sync,
    {
        if first.len() != second.len() {
            return Err(WaterError::new(
                WORKER_FAILURE,
                "paired worker outputs have different lengths",
            ));
        }
        let partitions = logical_partitions(first.len(), self.logical_partition_count)?;
        catch_unwind(AssertUnwindSafe(|| {
            self.pool
                .install(|| fill_pair_partitions(self, &partitions, 0, first, second, &operation))
        }))
        .map_err(|_| WaterError::new(WORKER_FAILURE, "worker panicked before canonical merge"))?
    }

    pub(super) fn try_fill_partitioned_pair<A, B, F, O>(
        &self,
        item_count: usize,
        first: &mut [A],
        second: &mut [B],
        offsets: O,
        operation: F,
    ) -> Result<(), WaterError>
    where
        A: Send,
        B: Send,
        F: Fn(LogicalPartition, &mut [A], &mut [B]) -> Result<(), WaterError> + Send + Sync,
        O: Fn(usize) -> (usize, usize) + Send + Sync,
    {
        let partitions = logical_partitions(item_count, self.logical_partition_count)?;
        let expected_end = offsets(item_count);
        if offsets(0) != (0, 0) || expected_end != (first.len(), second.len()) {
            return Err(WaterError::new(
                WORKER_FAILURE,
                "partition output offsets do not cover their destination arrays",
            ));
        }
        catch_unwind(AssertUnwindSafe(|| {
            self.pool.install(|| {
                fill_partitioned_pair(self, &partitions, 0, first, 0, second, &offsets, &operation)
            })
        }))
        .map_err(|_| WaterError::new(WORKER_FAILURE, "worker panicked before canonical merge"))?
    }

    #[cfg(not(test))]
    fn inject_fault(&self, _partition: usize) -> Result<(), WaterError> {
        Ok(())
    }

    #[cfg(test)]
    fn inject_fault(&self, partition: usize) -> Result<(), WaterError> {
        match self.fault {
            WorkerFault::None => Ok(()),
            WorkerFault::ErrorAt(target) if partition == target => Err(WaterError::new(
                WORKER_FAILURE,
                format!("injected worker error in logical partition {partition}"),
            )),
            WorkerFault::PanicAt(target) if partition == target => {
                panic!("injected worker panic in logical partition {partition}")
            }
            WorkerFault::ErrorAt(_) | WorkerFault::PanicAt(_) => Ok(()),
        }
    }

    #[cfg(test)]
    pub(super) fn with_fault(worker_count: usize, fault: WorkerFault) -> Result<Self, WaterError> {
        let mut workers = Self::new(worker_count)?;
        workers.fault = fault;
        Ok(workers)
    }

    #[cfg(test)]
    pub(crate) fn with_partition_count(
        worker_count: usize,
        logical_partition_count: usize,
    ) -> Result<Self, WaterError> {
        if logical_partition_count == 0 || logical_partition_count > 256 {
            return Err(WaterError::new(
                WORKER_FAILURE,
                "test logical partition count must be in 1..=256",
            ));
        }
        let mut workers = Self::new(worker_count)?;
        workers.logical_partition_count = logical_partition_count;
        Ok(workers)
    }
}

fn fill_partition_results<T, F>(
    workers: &DeterministicWorkers,
    partitions: &[LogicalPartition],
    output: &mut [Option<Result<T, WaterError>>],
    operation: &F,
) where
    T: Send,
    F: Fn(LogicalPartition) -> Result<T, WaterError> + Send + Sync,
{
    let Some(partition) = partitions.first().copied() else {
        return;
    };
    if partitions.len() == 1 {
        output[0] = Some(
            workers
                .inject_fault(partition.ordinal)
                .and_then(|()| operation(partition)),
        );
        return;
    }
    let middle = partitions.len() / 2;
    let (left_output, right_output) = output.split_at_mut(middle);
    rayon::join(
        || {
            fill_partition_results(workers, &partitions[..middle], left_output, operation);
        },
        || {
            fill_partition_results(workers, &partitions[middle..], right_output, operation);
        },
    );
}

fn fill_index_partitions<T, F>(
    workers: &DeterministicWorkers,
    partitions: &[LogicalPartition],
    output_start: usize,
    output: &mut [T],
    operation: &F,
) -> Result<(), WaterError>
where
    T: Send,
    F: Fn(usize) -> Result<T, WaterError> + Send + Sync,
{
    let Some(partition) = partitions.first().copied() else {
        return Ok(());
    };
    if partitions.len() == 1 {
        workers.inject_fault(partition.ordinal)?;
        for (slot, index) in output.iter_mut().zip(partition.start..partition.end) {
            *slot = operation(index)?;
        }
        return Ok(());
    }
    let middle = partitions.len() / 2;
    let right_start = partitions[middle].start;
    let split = right_start - output_start;
    let (left_output, right_output) = output.split_at_mut(split);
    let (left, right) = rayon::join(
        || {
            fill_index_partitions(
                workers,
                &partitions[..middle],
                output_start,
                left_output,
                operation,
            )
        },
        || {
            fill_index_partitions(
                workers,
                &partitions[middle..],
                right_start,
                right_output,
                operation,
            )
        },
    );
    left?;
    right
}

fn fill_pair_partitions<A, B, F>(
    workers: &DeterministicWorkers,
    partitions: &[LogicalPartition],
    output_start: usize,
    first: &mut [A],
    second: &mut [B],
    operation: &F,
) -> Result<(), WaterError>
where
    A: Send,
    B: Send,
    F: Fn(usize) -> Result<(A, B), WaterError> + Send + Sync,
{
    let Some(partition) = partitions.first().copied() else {
        return Ok(());
    };
    if partitions.len() == 1 {
        workers.inject_fault(partition.ordinal)?;
        for ((first_slot, second_slot), index) in first
            .iter_mut()
            .zip(second.iter_mut())
            .zip(partition.start..partition.end)
        {
            let (first_value, second_value) = operation(index)?;
            *first_slot = first_value;
            *second_slot = second_value;
        }
        return Ok(());
    }
    let middle = partitions.len() / 2;
    let right_start = partitions[middle].start;
    let split = right_start - output_start;
    let (left_first, right_first) = first.split_at_mut(split);
    let (left_second, right_second) = second.split_at_mut(split);
    let (left, right) = rayon::join(
        || {
            fill_pair_partitions(
                workers,
                &partitions[..middle],
                output_start,
                left_first,
                left_second,
                operation,
            )
        },
        || {
            fill_pair_partitions(
                workers,
                &partitions[middle..],
                right_start,
                right_first,
                right_second,
                operation,
            )
        },
    );
    left?;
    right
}

#[allow(clippy::too_many_arguments)]
fn fill_partitioned_pair<A, B, F, O>(
    workers: &DeterministicWorkers,
    partitions: &[LogicalPartition],
    first_start: usize,
    first: &mut [A],
    second_start: usize,
    second: &mut [B],
    offsets: &O,
    operation: &F,
) -> Result<(), WaterError>
where
    A: Send,
    B: Send,
    F: Fn(LogicalPartition, &mut [A], &mut [B]) -> Result<(), WaterError> + Send + Sync,
    O: Fn(usize) -> (usize, usize) + Send + Sync,
{
    let Some(partition) = partitions.first().copied() else {
        return Ok(());
    };
    if partitions.len() == 1 {
        workers.inject_fault(partition.ordinal)?;
        return operation(partition, first, second);
    }
    let middle = partitions.len() / 2;
    let right_item = partitions[middle].start;
    let (right_first, right_second) = offsets(right_item);
    let first_split = right_first.checked_sub(first_start).ok_or_else(|| {
        WaterError::new(WORKER_FAILURE, "first partition offsets are not monotonic")
    })?;
    let second_split = right_second.checked_sub(second_start).ok_or_else(|| {
        WaterError::new(WORKER_FAILURE, "second partition offsets are not monotonic")
    })?;
    if first_split > first.len() || second_split > second.len() {
        return Err(WaterError::new(
            WORKER_FAILURE,
            "partition output offset exceeds its destination",
        ));
    }
    let (left_first, right_first_output) = first.split_at_mut(first_split);
    let (left_second, right_second_output) = second.split_at_mut(second_split);
    let (left, right) = rayon::join(
        || {
            fill_partitioned_pair(
                workers,
                &partitions[..middle],
                first_start,
                left_first,
                second_start,
                left_second,
                offsets,
                operation,
            )
        },
        || {
            fill_partitioned_pair(
                workers,
                &partitions[middle..],
                right_first,
                right_first_output,
                right_second,
                right_second_output,
                offsets,
                operation,
            )
        },
    );
    left?;
    right
}

fn logical_partitions(
    item_count: usize,
    logical_partition_count: usize,
) -> Result<Vec<LogicalPartition>, WaterError> {
    let count = item_count.min(logical_partition_count);
    let mut partitions = Vec::new();
    partitions.try_reserve_exact(count).map_err(heap_error)?;
    if count == 0 {
        return Ok(partitions);
    }
    let base = item_count / count;
    let remainder = item_count % count;
    let mut start = 0_usize;
    for ordinal in 0..count {
        let length = base + usize::from(ordinal < remainder);
        let end = start
            .checked_add(length)
            .ok_or_else(|| WaterError::new(WORKER_FAILURE, "logical partition range overflow"))?;
        partitions.push(LogicalPartition {
            ordinal,
            start,
            end,
        });
        start = end;
    }
    Ok(partitions)
}

pub(super) fn pressure_acceleration(
    reconstruction: &Reconstruction,
    multiplier: &[f64],
) -> Result<AccelerationBatch, WaterError> {
    pressure_acceleration_with_workers(reconstruction, multiplier, None)
}

pub(super) fn pressure_acceleration_with_workers(
    reconstruction: &Reconstruction,
    multiplier: &[f64],
    workers: Option<&DeterministicWorkers>,
) -> Result<AccelerationBatch, WaterError> {
    if let Some(workers) = workers {
        let mut total = filled_vec(reconstruction.rows.len(), Vec3f::ZERO)?;
        let mut boundary = filled_vec(reconstruction.rows.len(), Vec3f::ZERO)?;
        workers.try_fill_pair(&mut total, &mut boundary, |index| {
            pressure_acceleration_row(reconstruction, multiplier, index)
        })?;
        return Ok(AccelerationBatch { total, boundary });
    }
    let mut total = Vec::new();
    let mut boundary = Vec::new();
    total
        .try_reserve_exact(multiplier.len())
        .map_err(heap_error)?;
    boundary
        .try_reserve_exact(multiplier.len())
        .map_err(heap_error)?;
    for index in 0..reconstruction.rows.len() {
        let (value, boundary_value) = pressure_acceleration_row(reconstruction, multiplier, index)?;
        total.push(value);
        boundary.push(boundary_value);
    }
    Ok(AccelerationBatch { total, boundary })
}

fn pressure_acceleration_row(
    reconstruction: &Reconstruction,
    multiplier: &[f64],
    index: usize,
) -> Result<(Vec3f, Vec3f), WaterError> {
    let row = reconstruction.rows[index];
    let mut value = Vec3f::ZERO;
    for neighbor in &reconstruction.fluid[row.fluid_start..row.fluid_end] {
        let pressure_sum = checked_scalar(
            multiplier[index] + multiplier[neighbor.other],
            "fluid pressure sum",
        )?;
        if pressure_sum.abs() > SOLVER_EPSILON {
            let scale = checked_scalar(
                -(REST_VOLUME * pressure_sum),
                "fluid pressure acceleration scale",
            )?;
            value = value
                .add(neighbor.gradient.scale(scale))
                .checked("fluid pressure acceleration reduction")?;
        }
    }
    let mut boundary_value = Vec3f::ZERO;
    if multiplier[index].abs() > SOLVER_EPSILON {
        for neighbor in &reconstruction.solid[row.solid_start..row.solid_end] {
            let scale = checked_scalar(
                -(neighbor.volume * multiplier[index]),
                "boundary pressure acceleration scale",
            )?;
            boundary_value = boundary_value
                .add(neighbor.gradient.scale(scale))
                .checked("boundary pressure acceleration reduction")?;
        }
    }
    value = value
        .add(boundary_value)
        .checked("total pressure acceleration")?;
    Ok((value, boundary_value))
}

pub(super) fn matrix_action(
    reconstruction: &Reconstruction,
    acceleration: &[Vec3f],
) -> Result<Vec<f64>, WaterError> {
    matrix_action_with_workers(reconstruction, acceleration, None)
}

pub(super) fn matrix_action_with_workers(
    reconstruction: &Reconstruction,
    acceleration: &[Vec3f],
    workers: Option<&DeterministicWorkers>,
) -> Result<Vec<f64>, WaterError> {
    if let Some(workers) = workers {
        return workers.try_map(reconstruction.rows.len(), |index| {
            matrix_action_row(reconstruction, acceleration, index)
        });
    }
    let mut result = Vec::new();
    result
        .try_reserve_exact(acceleration.len())
        .map_err(heap_error)?;
    for index in 0..reconstruction.rows.len() {
        result.push(matrix_action_row(reconstruction, acceleration, index)?);
    }
    Ok(result)
}

fn matrix_action_row(
    reconstruction: &Reconstruction,
    acceleration: &[Vec3f],
    index: usize,
) -> Result<f64, WaterError> {
    let row = reconstruction.rows[index];
    let mut value = 0.0;
    for neighbor in &reconstruction.fluid[row.fluid_start..row.fluid_end] {
        let relative = acceleration[index]
            .sub(acceleration[neighbor.other])
            .checked("matrix relative acceleration")?;
        let term = checked_scalar(
            REST_VOLUME * relative.dot(neighbor.gradient),
            "fluid matrix term",
        )?;
        value = checked_scalar(value + term, "fluid matrix reduction")?;
    }
    for neighbor in &reconstruction.solid[row.solid_start..row.solid_end] {
        let term = checked_scalar(
            neighbor.volume * acceleration[index].dot(neighbor.gradient),
            "boundary matrix term",
        )?;
        value = checked_scalar(value + term, "boundary matrix reduction")?;
    }
    Ok(value)
}

#[cfg(test)]
#[derive(Clone, Copy)]
pub(super) enum WorkerFault {
    None,
    ErrorAt(usize),
    PanicAt(usize),
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn logical_partitions_are_fixed_and_contiguous() {
        let partitions = logical_partitions(1_003, LOGICAL_PARTITION_COUNT).unwrap();
        assert_eq!(partitions.len(), LOGICAL_PARTITION_COUNT);
        assert_eq!(partitions.first().unwrap().start, 0);
        assert_eq!(partitions.last().unwrap().end, 1_003);
        assert!(
            partitions
                .windows(2)
                .all(|pair| pair[0].end == pair[1].start)
        );
    }

    #[test]
    fn completion_order_cannot_change_index_order() {
        for worker_count in [1, 2, 4, 8] {
            let workers = DeterministicWorkers::new(worker_count).unwrap();
            let output = workers.try_map(1_003, Ok).unwrap();
            assert_eq!(output, (0..1_003).collect::<Vec<_>>());
        }
    }

    #[test]
    fn error_and_panic_are_rejected_before_merge() {
        let error_workers = DeterministicWorkers::with_fault(4, WorkerFault::ErrorAt(3)).unwrap();
        assert_eq!(
            error_workers.try_map(1_003, Ok).unwrap_err().code(),
            WORKER_FAILURE
        );
        let panic_workers = DeterministicWorkers::with_fault(4, WorkerFault::PanicAt(3)).unwrap();
        assert_eq!(
            panic_workers.try_map(1_003, Ok).unwrap_err().code(),
            WORKER_FAILURE
        );
    }

    #[test]
    fn operation_allocation_error_is_preserved_by_canonical_merge() {
        let workers = DeterministicWorkers::new(4).unwrap();
        let error = workers
            .try_map(1_003, |index| {
                if index == 700 {
                    Err(WaterError::new(
                        crate::error::DECODED_HEAP_CAPACITY_EXCEEDED,
                        "injected fragment allocation fault",
                    ))
                } else {
                    Ok(index)
                }
            })
            .unwrap_err();
        assert_eq!(error.code(), crate::error::DECODED_HEAP_CAPACITY_EXCEEDED);
    }
}
