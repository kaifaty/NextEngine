//! Deterministic CPU topology detection, worker placement and thread pinning.
//!
//! ADR-093 confines the entire `unsafe` surface of this crate to
//! [`raw::pin_current_thread_to`]: one syscall applying one CPU mask to the
//! calling thread plus a read-back verification. Gameplay simulation never
//! depends on this crate; only the R5 release-performance workload consumes
//! it.

#![allow(
    unsafe_code,
    reason = "ADR-093 confines CPU affinity to this reviewed crate boundary"
)]

mod raw;

#[cfg(test)]
mod tests;

/// Pin the calling thread to exactly one logical CPU.
///
/// This is the only mutation this crate performs; see [`AffinityError`] for
/// the typed failure modes. Failures must be propagated, never absorbed.
pub fn pin_current_thread(cpu_id: u32) -> Result<(), AffinityError> {
    raw::pin_current_thread_to(cpu_id)
}

use std::collections::{BTreeMap, BTreeSet};
use std::fmt::{Display, Formatter};
use std::fs;
use std::path::{Path, PathBuf};

/// One online logical CPU with its physical-core identity and cache domain.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct LogicalCpu {
    pub cpu_id: u32,
    pub core_id: u32,
    /// Identifier of the highest-level cache domain (`L3` on current hosts).
    pub last_level_cache_id: Option<u32>,
}

/// Online CPU topology in canonical ascending CPU order.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CpuTopology {
    pub logical_cpus: Vec<LogicalCpu>,
}

#[derive(Debug)]
pub enum AffinityError {
    UnsupportedPlatform,
    TopologyUnreadable(&'static str),
    InvalidTopology(&'static str),
    PinFailed(i32),
    PinVerificationMismatch,
}

impl Display for AffinityError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::UnsupportedPlatform => write!(f, "CPU_AFFINITY_UNSUPPORTED_PLATFORM"),
            Self::TopologyUnreadable(what) => {
                write!(f, "CPU_AFFINITY_TOPOLOGY_UNREADABLE: {what}")
            }
            Self::InvalidTopology(what) => write!(f, "CPU_AFFINITY_INVALID_TOPOLOGY: {what}"),
            Self::PinFailed(errno) => write!(f, "CPU_AFFINITY_PIN_FAILED: errno {errno}"),
            Self::PinVerificationMismatch => write!(f, "CPU_AFFINITY_PIN_VERIFICATION_MISMATCH"),
        }
    }
}

impl std::error::Error for AffinityError {}

impl CpuTopology {
    /// Read the online topology from Linux sysfs.
    pub fn detect() -> Result<Self, AffinityError> {
        detect_sysfs_topology(PathBuf::from("/sys/devices/system/cpu"))
    }

    /// Detect from an explicit sysfs root; exposed for focused tests.
    pub fn detect_from(sysfs_cpu_root: PathBuf) -> Result<Self, AffinityError> {
        detect_sysfs_topology(sysfs_cpu_root)
    }

    /// Physical cores ordered by `(last-level-cache id, core id)` and
    /// represented by their lowest logical CPU id.
    pub fn ordered_physical_cores(&self) -> Vec<(Option<u32>, u32, u32)> {
        let mut seen = BTreeSet::new();
        let mut cores = Vec::new();
        for cpu in &self.logical_cpus {
            if seen.insert((cpu.core_id, cpu.last_level_cache_id)) {
                cores.push((cpu.last_level_cache_id, cpu.core_id, cpu.cpu_id));
            }
        }
        cores.sort();
        cores
    }

    /// Deterministic placement sequence: a round-robin interleave of cache
    /// domains, each contributing its physical cores in ascending core-id
    /// order.
    pub fn interleaved_core_sequence(&self) -> Result<Vec<u32>, AffinityError> {
        let mut by_domain: BTreeMap<Option<u32>, Vec<u32>> = BTreeMap::new();
        for (domain, _core_id, representative) in self.ordered_physical_cores() {
            by_domain.entry(domain).or_default().push(representative);
        }
        if by_domain.values().all(Vec::is_empty) || by_domain.is_empty() {
            return Err(AffinityError::InvalidTopology("no online physical cores"));
        }
        let domains: Vec<&Vec<u32>> = by_domain.values().collect();
        let max_depth = domains
            .iter()
            .map(|domain| domain.len())
            .max()
            .expect("non-empty");
        let mut sequence = Vec::with_capacity(domains.iter().map(|domain| domain.len()).sum());
        for depth in 0..max_depth {
            for domain in &domains {
                if let Some(representative) = domain.get(depth) {
                    sequence.push(*representative);
                }
            }
        }
        Ok(sequence)
    }

    /// Choose the pinned logical CPU for each of `worker_count` workers.
    pub fn deterministic_worker_placement(
        &self,
        worker_count: usize,
    ) -> Result<Vec<u32>, AffinityError> {
        if worker_count == 0 {
            return Err(AffinityError::InvalidTopology("worker count is zero"));
        }
        let sequence = self.interleaved_core_sequence()?;
        if worker_count > sequence.len() {
            return Err(AffinityError::InvalidTopology(
                "worker count exceeds distinct physical cores",
            ));
        }
        Ok(sequence[..worker_count].to_vec())
    }
}

fn read_trimmed(path: &Path, what: &'static str) -> Result<String, AffinityError> {
    fs::read_to_string(path)
        .map(|value| value.trim().to_owned())
        .map_err(|_| AffinityError::TopologyUnreadable(what))
}

fn parse_u32(value: &str, what: &'static str) -> Result<u32, AffinityError> {
    value
        .parse::<u32>()
        .map_err(|_| AffinityError::InvalidTopology(what))
}

fn detect_sysfs_topology(base: PathBuf) -> Result<CpuTopology, AffinityError> {
    let entries =
        fs::read_dir(&base).map_err(|_| AffinityError::TopologyUnreadable("cpu directory"))?;
    let mut logical_cpus = Vec::new();
    for entry in entries.flatten() {
        let name = entry.file_name();
        let Some(cpu_id_text) = extract_online_cpu_name(&name.to_string_lossy()) else {
            continue;
        };
        // A present-and-zero `online` flag marks an offline CPU; the flag is
        // absent on always-online entries.
        if matches!(
            read_trimmed(&entry.path().join("online"), "online flag").as_deref(),
            Ok("0")
        ) {
            continue;
        }
        let cpu_id = parse_u32(&cpu_id_text, "cpu id")?;
        let topology = entry.path().join("topology");
        let core_id = parse_u32(
            &read_trimmed(&topology.join("core_id"), "core_id")?,
            "core_id",
        )?;
        let package = parse_u32(
            &read_trimmed(&topology.join("physical_package_id"), "package")?,
            "package",
        )?;
        if package != 0 {
            return Err(AffinityError::InvalidTopology("multi-package host"));
        }
        let last_level_cache_id = last_level_cache_id(&entry.path())?;
        logical_cpus.push(LogicalCpu {
            cpu_id,
            core_id,
            last_level_cache_id,
        });
    }
    if logical_cpus.is_empty() {
        return Err(AffinityError::InvalidTopology("no online logical CPUs"));
    }
    logical_cpus.sort_by_key(|cpu| cpu.cpu_id);
    Ok(CpuTopology { logical_cpus })
}

fn extract_online_cpu_name(name: &str) -> Option<String> {
    let cpu_id_text = name.strip_prefix("cpu")?;
    if cpu_id_text.is_empty() || !cpu_id_text.bytes().all(|byte| byte.is_ascii_digit()) {
        return None;
    }
    Some(cpu_id_text.to_owned())
}

fn last_level_cache_id(cpu_dir: &Path) -> Result<Option<u32>, AffinityError> {
    let mut best_level = 0;
    let mut best_id = None;
    for index in 0..=u8::MAX {
        let index_dir = cpu_dir.join(format!("cache/index{index}"));
        let Ok(level_text) = read_trimmed(&index_dir.join("level"), "cache level") else {
            break;
        };
        let level = parse_u32(&level_text, "cache level")?;
        let id = parse_u32(
            &read_trimmed(&index_dir.join("id"), "cache id")?,
            "cache id",
        )?;
        if level > best_level {
            best_level = level;
            best_id = Some(id);
        } else if level == best_level {
            debug_assert_eq!(best_id, Some(id), "inconsistent same-level cache ids");
        }
    }
    Ok(best_id)
}
