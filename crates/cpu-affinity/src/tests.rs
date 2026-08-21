use super::*;
use std::fs;
use std::path::Path;

/// 3950X-shaped fixture: 16 physical cores in four L3 domains of four cores,
/// SMT siblings offset by 16.
fn write_zen2_topology(root: &Path) {
    for cpu in 0..32_u32 {
        let core = cpu % 16;
        let domain = core / 4;
        let dir = root.join(format!("cpu{cpu}"));
        fs::create_dir_all(dir.join("topology")).unwrap();
        fs::write(dir.join("online"), b"1").unwrap();
        fs::write(dir.join("topology/core_id"), core.to_string()).unwrap();
        fs::write(dir.join("topology/physical_package_id"), b"0").unwrap();
        let cache = dir.join("cache/index0");
        fs::create_dir_all(&cache).unwrap();
        fs::write(cache.join("level"), b"3").unwrap();
        fs::write(cache.join("id"), domain.to_string()).unwrap();
    }
}

fn detect_fixture(root: &Path) -> CpuTopology {
    CpuTopology::detect_from(root.to_path_buf()).expect("fixture topology")
}

#[test]
fn zen2_topology_interleaves_domains_round_robin() {
    let temp = tempfile::tempdir().unwrap();
    write_zen2_topology(temp.path());
    let topology = detect_fixture(temp.path());
    assert_eq!(topology.logical_cpus.len(), 32);
    assert_eq!(
        topology.interleaved_core_sequence().unwrap(),
        vec![0, 4, 8, 12, 1, 5, 9, 13, 2, 6, 10, 14, 3, 7, 11, 15]
    );
    assert_eq!(topology.deterministic_worker_placement(1).unwrap(), vec![0]);
    assert_eq!(
        topology.deterministic_worker_placement(4).unwrap(),
        vec![0, 4, 8, 12]
    );
    assert_eq!(
        topology.deterministic_worker_placement(8).unwrap(),
        vec![0, 4, 8, 12, 1, 5, 9, 13]
    );
}

#[test]
fn placement_rejects_zero_and_oversubscribed_workers() {
    let temp = tempfile::tempdir().unwrap();
    write_zen2_topology(temp.path());
    let topology = detect_fixture(temp.path());
    assert!(matches!(
        topology.deterministic_worker_placement(0),
        Err(AffinityError::InvalidTopology("worker count is zero"))
    ));
    assert!(matches!(
        topology.deterministic_worker_placement(17),
        Err(AffinityError::InvalidTopology(
            "worker count exceeds distinct physical cores"
        ))
    ));
}

#[test]
fn missing_cache_information_degrades_to_core_order() {
    let temp = tempfile::tempdir().unwrap();
    for cpu in 0..8_u32 {
        let dir = temp.path().join(format!("cpu{cpu}"));
        fs::create_dir_all(dir.join("topology")).unwrap();
        fs::write(dir.join("topology/core_id"), cpu.to_string()).unwrap();
        fs::write(dir.join("topology/physical_package_id"), b"0").unwrap();
    }
    let topology = detect_fixture(temp.path());
    assert!(
        topology
            .logical_cpus
            .iter()
            .all(|cpu| cpu.last_level_cache_id.is_none())
    );
    // One implicit domain: cores ascend in plain core-id order.
    assert_eq!(
        topology.deterministic_worker_placement(4).unwrap(),
        vec![0, 1, 2, 3]
    );
}

#[test]
fn offline_cpus_and_foreign_entries_are_excluded() {
    let temp = tempfile::tempdir().unwrap();
    write_zen2_topology(temp.path());
    fs::write(temp.path().join("cpu5/online"), b"0").unwrap();
    fs::write(temp.path().join("cpuidle"), b"ignored directory").unwrap();
    let topology = detect_fixture(temp.path());
    assert_eq!(topology.logical_cpus.len(), 31);
    assert!(!topology.logical_cpus.iter().any(|cpu| cpu.cpu_id == 5));
    // Core 5 survives through its online SMT sibling: the representative
    // moves from cpu5 to cpu21 and the placement stays domain-balanced.
    assert_eq!(
        topology.deterministic_worker_placement(8).unwrap(),
        vec![0, 4, 8, 12, 1, 21, 9, 13]
    );
}

#[test]
fn multi_package_hosts_are_rejected() {
    let temp = tempfile::tempdir().unwrap();
    for (cpu, package) in [(0u32, 0u32), (32, 1)] {
        let dir = temp.path().join(format!("cpu{cpu}"));
        fs::create_dir_all(dir.join("topology")).unwrap();
        fs::write(dir.join("topology/core_id"), b"0").unwrap();
        fs::write(
            dir.join("topology/physical_package_id"),
            package.to_string(),
        )
        .unwrap();
    }
    assert!(matches!(
        CpuTopology::detect_from(temp.path().to_path_buf()),
        Err(AffinityError::InvalidTopology("multi-package host"))
    ));
}
