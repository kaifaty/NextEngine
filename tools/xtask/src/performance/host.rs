use std::collections::BTreeSet;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

use serde::Deserialize;

use super::{
    PerformancePreflightV1, PerformanceResourceCountersV4, PerformanceTargetFingerprintV1,
    THOTH_TARGET_ID,
};

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct WindowsHostProbe {
    hostname: String,
    cpu_model: String,
    physical_cores: u32,
    logical_threads: u32,
    ram_bytes: u64,
    storage_model: String,
    storage_bytes: u64,
    os_name: String,
    os_build: String,
    bios_version: String,
    cpu_load_percent: Option<u32>,
    free_ram_bytes: Option<u64>,
    cpu_current_mhz: Option<u64>,
    cpu_max_mhz: Option<u64>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
struct NvidiaProbe {
    gpu_model: String,
    gpu_vram_mib: u64,
    gpu_driver: String,
    gpu_load_percent: Option<u32>,
    gpu_thermal_slowdown_active: Option<bool>,
}

pub fn inspect_current_host(
    target_id: &str,
) -> Result<(PerformanceTargetFingerprintV1, PerformancePreflightV1), String> {
    if cfg!(target_os = "windows") {
        inspect_windows_host(target_id)
    } else if cfg!(target_os = "linux") {
        inspect_linux_host(target_id)
    } else {
        Err("PERF_TARGET_FINGERPRINT_UNSUPPORTED_HOST".to_owned())
    }
}

fn inspect_windows_host(
    target_id: &str,
) -> Result<(PerformanceTargetFingerprintV1, PerformancePreflightV1), String> {
    let script = r#"$ErrorActionPreference='Stop'
$cpu = @(Get-CimInstance Win32_Processor)[0]
$computer = Get-CimInstance Win32_ComputerSystem
$os = Get-CimInstance Win32_OperatingSystem
$bios = Get-CimInstance Win32_BIOS
$disk = @(Get-CimInstance Win32_DiskDrive | Where-Object { $_.Model -like '*WDS100T1X0E*' })[0]
if ($null -eq $disk) { $disk = @(Get-CimInstance Win32_DiskDrive)[0] }
$load = @(Get-CimInstance Win32_Processor | Measure-Object -Property LoadPercentage -Average)[0].Average
[ordered]@{
  hostname = [string]$env:COMPUTERNAME
  cpu_model = ([string]$cpu.Name).Trim()
  physical_cores = [uint32]$cpu.NumberOfCores
  logical_threads = [uint32]$cpu.NumberOfLogicalProcessors
  ram_bytes = [uint64]$computer.TotalPhysicalMemory
  storage_model = ([string]$disk.Model).Trim()
  storage_bytes = [uint64]$disk.Size
  os_name = ([string]$os.Caption).Trim()
  os_build = [string]$os.BuildNumber
  bios_version = [string]$bios.SMBIOSBIOSVersion
  cpu_load_percent = if ($null -eq $load) { $null } else { [uint32][math]::Round($load) }
  free_ram_bytes = [uint64]$os.FreePhysicalMemory * 1024
  cpu_current_mhz = [uint64]$cpu.CurrentClockSpeed
  cpu_max_mhz = [uint64]$cpu.MaxClockSpeed
} | ConvertTo-Json -Compress"#;
    let output = run_command(
        "powershell",
        &["-NoProfile", "-NonInteractive", "-Command", script],
    )?;
    let host: WindowsHostProbe = serde_json::from_slice(&output)
        .map_err(|error| format!("PERF_HOST_PROBE_INVALID: {error}"))?;
    let nvidia = inspect_nvidia()?;
    let power_plan_output = run_command("powercfg", &["/getactivescheme"])?;
    let power_plan = parse_power_plan(&String::from_utf8_lossy(&power_plan_output));

    let cpu_clock_percent_of_maximum =
        host.cpu_current_mhz
            .zip(host.cpu_max_mhz)
            .and_then(|(current, maximum)| {
                (maximum != 0).then(|| {
                    u32::try_from(current.saturating_mul(100) / maximum).unwrap_or(u32::MAX)
                })
            });
    let fingerprint = PerformanceTargetFingerprintV1 {
        target_id: target_id.to_owned(),
        hostname: host.hostname,
        cpu_model: host.cpu_model,
        physical_cores: host.physical_cores,
        logical_threads: host.logical_threads,
        gpu_model: nvidia.gpu_model,
        gpu_vram_mib: nvidia.gpu_vram_mib,
        ram_bytes: host.ram_bytes,
        storage_model: host.storage_model,
        storage_bytes: host.storage_bytes,
        os_name: host.os_name,
        os_build: host.os_build,
        bios_version: host.bios_version,
        gpu_driver: nvidia.gpu_driver,
        power_plan,
    };
    let mut preflight = PerformancePreflightV1 {
        cpu_load_percent: host.cpu_load_percent,
        gpu_load_percent: nvidia.gpu_load_percent,
        free_ram_bytes: host.free_ram_bytes,
        cpu_clock_percent_of_maximum,
        gpu_thermal_slowdown_active: nvidia.gpu_thermal_slowdown_active,
        ready: true,
        diagnostics: Vec::new(),
    };
    preflight.recompute_readiness();
    Ok((fingerprint, preflight))
}

fn inspect_linux_host(
    target_id: &str,
) -> Result<(PerformanceTargetFingerprintV1, PerformancePreflightV1), String> {
    let cpuinfo = read_linux_host_file("/proc/cpuinfo")?;
    let (cpu_model, physical_cores, logical_threads) = parse_linux_cpuinfo(&cpuinfo)?;
    let meminfo = read_linux_host_file("/proc/meminfo")?;
    let ram_bytes = parse_linux_meminfo_bytes(&meminfo, "MemTotal")?;
    let free_ram_bytes = parse_linux_meminfo_bytes(&meminfo, "MemAvailable")?;
    let loadavg = read_linux_host_file("/proc/loadavg")?;
    let cpu_load_percent = parse_linux_load_percent(&loadavg, logical_threads)?;
    let cpu_clock_percent_of_maximum = inspect_linux_cpu_clock_percent()?;
    let hostname = read_linux_host_file("/proc/sys/kernel/hostname")?
        .trim()
        .to_owned();
    let kernel = read_linux_host_file("/proc/sys/kernel/osrelease")?
        .trim()
        .to_owned();
    let os_release = read_linux_host_file("/etc/os-release")?;
    let os_pretty_name = parse_linux_os_release_value(&os_release, "PRETTY_NAME")?;
    let os_version = parse_linux_os_release_value(&os_release, "VERSION_ID")?;
    let bios_version = read_linux_optional_host_file("/sys/class/dmi/id/bios_version")
        .filter(|value| !value.is_empty())
        .unwrap_or_else(|| "unavailable-on-linux-host".to_owned());
    let power_plan = inspect_linux_power_plan();
    let (storage_model, storage_bytes) = inspect_linux_root_storage()?;
    let nvidia = inspect_nvidia()?;

    if hostname.is_empty() || kernel.is_empty() {
        return Err("PERF_LINUX_HOST_PROBE_INVALID: hostname or kernel is empty".to_owned());
    }

    let fingerprint = PerformanceTargetFingerprintV1 {
        target_id: target_id.to_owned(),
        hostname,
        cpu_model,
        physical_cores,
        logical_threads,
        gpu_model: nvidia.gpu_model,
        gpu_vram_mib: nvidia.gpu_vram_mib,
        ram_bytes,
        storage_model,
        storage_bytes,
        os_name: format!("Linux ({os_pretty_name})"),
        os_build: format!("{os_version}; kernel {kernel}"),
        bios_version,
        gpu_driver: nvidia.gpu_driver,
        power_plan,
    };
    let mut preflight = PerformancePreflightV1 {
        cpu_load_percent: Some(cpu_load_percent),
        gpu_load_percent: nvidia.gpu_load_percent,
        free_ram_bytes: Some(free_ram_bytes),
        cpu_clock_percent_of_maximum: Some(cpu_clock_percent_of_maximum),
        gpu_thermal_slowdown_active: nvidia.gpu_thermal_slowdown_active,
        ready: true,
        diagnostics: Vec::new(),
    };
    preflight.recompute_readiness();
    Ok((fingerprint, preflight))
}

fn read_linux_host_file(path: impl AsRef<Path>) -> Result<String, String> {
    let path = path.as_ref();
    fs::read_to_string(path)
        .map_err(|error| format!("PERF_LINUX_HOST_PROBE_FAILED: {}: {error}", path.display()))
}

fn read_linux_optional_host_file(path: impl AsRef<Path>) -> Option<String> {
    fs::read_to_string(path)
        .ok()
        .map(|value| value.trim().to_owned())
}

fn parse_linux_cpuinfo(value: &str) -> Result<(String, u32, u32), String> {
    let mut cpu_model = None;
    let mut logical_threads = 0_u32;
    let mut physical_core_ids = BTreeSet::new();
    let mut declared_cores = None;

    for processor in value.split("\n\n") {
        let mut physical_id = None;
        let mut core_id = None;
        let mut is_processor = false;
        for line in processor.lines() {
            let Some((key, field)) = line.split_once(':') else {
                continue;
            };
            let key = key.trim();
            let field = field.trim();
            match key {
                "processor" => is_processor = true,
                "model name" if cpu_model.is_none() && !field.is_empty() => {
                    cpu_model = Some(field.to_owned());
                }
                "physical id" => physical_id = field.parse::<u32>().ok(),
                "core id" => core_id = field.parse::<u32>().ok(),
                "cpu cores" if declared_cores.is_none() => {
                    declared_cores = field.parse::<u32>().ok();
                }
                _ => {}
            }
        }
        if is_processor {
            logical_threads = logical_threads.saturating_add(1);
            if let (Some(physical_id), Some(core_id)) = (physical_id, core_id) {
                physical_core_ids.insert((physical_id, core_id));
            }
        }
    }

    let cpu_model = cpu_model
        .filter(|model| !model.is_empty())
        .ok_or_else(|| "PERF_LINUX_CPU_MODEL_UNAVAILABLE".to_owned())?;
    if logical_threads == 0 {
        return Err("PERF_LINUX_LOGICAL_THREAD_COUNT_INVALID".to_owned());
    }
    let physical_cores = if physical_core_ids.is_empty() {
        declared_cores.unwrap_or(logical_threads)
    } else {
        u32::try_from(physical_core_ids.len())
            .map_err(|error| format!("PERF_LINUX_PHYSICAL_CORE_COUNT_INVALID: {error}"))?
    };
    if physical_cores == 0 {
        return Err("PERF_LINUX_PHYSICAL_CORE_COUNT_INVALID".to_owned());
    }
    Ok((cpu_model, physical_cores, logical_threads))
}

fn parse_linux_meminfo_bytes(value: &str, name: &str) -> Result<u64, String> {
    let line = value
        .lines()
        .find(|line| line.split_once(':').is_some_and(|(key, _)| key == name))
        .ok_or_else(|| format!("PERF_LINUX_MEMINFO_FIELD_MISSING: {name}"))?;
    let (_, field) = line
        .split_once(':')
        .ok_or_else(|| format!("PERF_LINUX_MEMINFO_FIELD_INVALID: {name}"))?;
    let mut fields = field.split_ascii_whitespace();
    let kibibytes = fields
        .next()
        .ok_or_else(|| format!("PERF_LINUX_MEMINFO_FIELD_INVALID: {name}"))?
        .parse::<u64>()
        .map_err(|error| format!("PERF_LINUX_MEMINFO_FIELD_INVALID: {name}: {error}"))?;
    if fields.next() != Some("kB") || fields.next().is_some() {
        return Err(format!("PERF_LINUX_MEMINFO_FIELD_INVALID: {name}"));
    }
    kibibytes
        .checked_mul(1024)
        .ok_or_else(|| format!("PERF_LINUX_MEMINFO_FIELD_OVERFLOW: {name}"))
}

fn parse_linux_load_percent(value: &str, logical_threads: u32) -> Result<u32, String> {
    if logical_threads == 0 {
        return Err("PERF_LINUX_LOGICAL_THREAD_COUNT_INVALID".to_owned());
    }
    let one_minute_load = value
        .split_ascii_whitespace()
        .next()
        .ok_or_else(|| "PERF_LINUX_LOADAVG_INVALID".to_owned())?
        .parse::<f64>()
        .map_err(|error| format!("PERF_LINUX_LOADAVG_INVALID: {error}"))?;
    if !one_minute_load.is_finite() || one_minute_load.is_sign_negative() {
        return Err("PERF_LINUX_LOADAVG_INVALID".to_owned());
    }
    let percent = (one_minute_load * 100.0 / f64::from(logical_threads)).ceil();
    if percent > f64::from(u32::MAX) {
        Ok(u32::MAX)
    } else {
        Ok(percent as u32)
    }
}

fn parse_linux_os_release_value(value: &str, name: &str) -> Result<String, String> {
    let raw = value
        .lines()
        .find_map(|line| line.strip_prefix(&format!("{name}=")))
        .ok_or_else(|| format!("PERF_LINUX_OS_RELEASE_FIELD_MISSING: {name}"))?;
    let decoded = raw
        .strip_prefix('"')
        .and_then(|value| value.strip_suffix('"'))
        .unwrap_or(raw)
        .replace("\\\"", "\"")
        .replace("\\\\", "\\");
    if decoded.is_empty() {
        Err(format!("PERF_LINUX_OS_RELEASE_FIELD_INVALID: {name}"))
    } else {
        Ok(decoded)
    }
}

fn inspect_linux_cpu_clock_percent() -> Result<u32, String> {
    let cpu_root = Path::new("/sys/devices/system/cpu");
    let entries = fs::read_dir(cpu_root)
        .map_err(|error| format!("PERF_LINUX_CPU_CLOCK_PROBE_FAILED: {error}"))?;
    let mut maximum_current_khz = 0_u64;
    let mut maximum_supported_khz = 0_u64;
    for entry in entries {
        let entry = entry.map_err(|error| format!("PERF_LINUX_CPU_CLOCK_PROBE_FAILED: {error}"))?;
        let name = entry.file_name();
        let name = name.to_string_lossy();
        if !name.strip_prefix("cpu").is_some_and(|suffix| {
            !suffix.is_empty() && suffix.bytes().all(|byte| byte.is_ascii_digit())
        }) {
            continue;
        }
        let cpufreq = entry.path().join("cpufreq");
        if let Some(current) = read_linux_optional_u64(cpufreq.join("scaling_cur_freq")) {
            maximum_current_khz = maximum_current_khz.max(current);
        }
        if let Some(maximum) = read_linux_optional_u64(cpufreq.join("cpuinfo_max_freq"))
            .or_else(|| read_linux_optional_u64(cpufreq.join("scaling_max_freq")))
        {
            maximum_supported_khz = maximum_supported_khz.max(maximum);
        }
    }
    if maximum_current_khz == 0 || maximum_supported_khz == 0 {
        return Err("PERF_LINUX_CPU_CLOCK_PROBE_UNAVAILABLE".to_owned());
    }
    Ok(
        u32::try_from(maximum_current_khz.saturating_mul(100) / maximum_supported_khz)
            .unwrap_or(u32::MAX),
    )
}

fn read_linux_optional_u64(path: impl AsRef<Path>) -> Option<u64> {
    read_linux_optional_host_file(path)?.parse().ok()
}

fn inspect_linux_power_plan() -> String {
    read_linux_optional_host_file("/sys/devices/system/cpu/cpu0/cpufreq/scaling_governor")
        .filter(|value| !value.is_empty())
        .map_or_else(
            || "linux-governor-unavailable".to_owned(),
            |governor| format!("linux-governor:{governor}"),
        )
}

fn inspect_linux_root_storage() -> Result<(String, u64), String> {
    let mountinfo = read_linux_host_file("/proc/self/mountinfo")?;
    let device_id = parse_linux_root_device_id(&mountinfo)?;
    let mut device_path = fs::canonicalize(format!("/sys/dev/block/{device_id}"))
        .map_err(|error| format!("PERF_LINUX_STORAGE_PROBE_FAILED: {error}"))?;
    if device_path.join("partition").is_file() {
        device_path = device_path
            .parent()
            .map(Path::to_path_buf)
            .ok_or_else(|| "PERF_LINUX_STORAGE_PROBE_INVALID".to_owned())?;
    }
    device_path = linux_storage_backing_device(device_path)?;
    let device_name = device_path
        .file_name()
        .and_then(|name| name.to_str())
        .ok_or_else(|| "PERF_LINUX_STORAGE_PROBE_INVALID".to_owned())?;
    let model = read_linux_optional_host_file(device_path.join("device/model"))
        .filter(|value| !value.is_empty())
        .unwrap_or_else(|| device_name.to_owned());
    let sectors = read_linux_optional_u64(device_path.join("size"))
        .ok_or_else(|| "PERF_LINUX_STORAGE_SIZE_UNAVAILABLE".to_owned())?;
    let bytes = sectors
        .checked_mul(512)
        .ok_or_else(|| "PERF_LINUX_STORAGE_SIZE_OVERFLOW".to_owned())?;
    if bytes == 0 {
        return Err("PERF_LINUX_STORAGE_SIZE_INVALID".to_owned());
    }
    Ok((model, bytes))
}

fn linux_storage_backing_device(device_path: PathBuf) -> Result<PathBuf, String> {
    let slaves = device_path.join("slaves");
    let Ok(entries) = fs::read_dir(slaves) else {
        return Ok(device_path);
    };
    let mut paths = entries
        .filter_map(Result::ok)
        .map(|entry| entry.path())
        .collect::<Vec<_>>();
    paths.sort();
    let Some(first) = paths.first() else {
        return Ok(device_path);
    };
    fs::canonicalize(first).map_err(|error| format!("PERF_LINUX_STORAGE_PROBE_FAILED: {error}"))
}

fn parse_linux_root_device_id(value: &str) -> Result<&str, String> {
    value
        .lines()
        .find_map(|line| {
            let fields = line.split_ascii_whitespace().collect::<Vec<_>>();
            (fields.get(4) == Some(&"/"))
                .then(|| fields.get(2).copied())
                .flatten()
        })
        .filter(|device_id| {
            device_id.split_once(':').is_some_and(|(major, minor)| {
                !major.is_empty()
                    && !minor.is_empty()
                    && major.bytes().all(|byte| byte.is_ascii_digit())
                    && minor.bytes().all(|byte| byte.is_ascii_digit())
            })
        })
        .ok_or_else(|| "PERF_LINUX_ROOT_DEVICE_UNAVAILABLE".to_owned())
}

pub fn validate_thoth_fingerprint(fingerprint: &PerformanceTargetFingerprintV1) -> Vec<String> {
    let mut diagnostics = Vec::new();
    let normalized_cpu = normalize(&fingerprint.cpu_model);
    let normalized_gpu = normalize(&fingerprint.gpu_model);
    let normalized_os = normalize(&fingerprint.os_name);
    if fingerprint.target_id != THOTH_TARGET_ID {
        diagnostics.push("PERF_TARGET_ID_MISMATCH".to_owned());
    }
    if !fingerprint.hostname.eq_ignore_ascii_case("THOTH") {
        diagnostics.push("PERF_HOSTNAME_MISMATCH".to_owned());
    }
    if !normalized_cpu.contains("amdryzen93950x")
        || fingerprint.physical_cores != 16
        || fingerprint.logical_threads != 32
    {
        diagnostics.push("PERF_CPU_MISMATCH".to_owned());
    }
    if !normalized_gpu.contains("nvidiageforcertx3080") || fingerprint.gpu_vram_mib != 10_240 {
        diagnostics.push("PERF_GPU_MISMATCH".to_owned());
    }
    if fingerprint.ram_bytes < 31 * 1024 * 1024 * 1024 {
        diagnostics.push("PERF_RAM_MISMATCH".to_owned());
    }
    if !normalize(&fingerprint.storage_model).contains("wds100t1x0e00afy0")
        || fingerprint.storage_bytes < 1_000_000_000_000
    {
        diagnostics.push("PERF_STORAGE_MISMATCH".to_owned());
    }
    if !normalized_os.contains("windows11pro") || fingerprint.os_build != "26200" {
        diagnostics.push("PERF_OS_MISMATCH".to_owned());
    }
    if fingerprint.gpu_driver != "610.88" {
        diagnostics.push("PERF_GPU_DRIVER_MISMATCH".to_owned());
    }
    if normalize(&fingerprint.power_plan) != "amdryzenhighperformance" {
        diagnostics.push("PERF_POWER_PLAN_MISMATCH".to_owned());
    }
    diagnostics
}

pub fn inspect_process_counters() -> PerformanceResourceCountersV4 {
    if !cfg!(target_os = "windows") {
        return PerformanceResourceCountersV4 {
            unavailable: vec![
                "process residency counters are not implemented on this report-only host"
                    .to_owned(),
                "Vulkan timestamps require a representative render workload".to_owned(),
            ],
            ..PerformanceResourceCountersV4::default()
        };
    }
    let pid = std::process::id();
    let script = format!(
        r#"$ErrorActionPreference='Stop'
$source = @'
using System;
using System.Runtime.InteropServices;
public static class NextEngineProcessIo {{
  [StructLayout(LayoutKind.Sequential)]
  public struct IoCounters {{
    public ulong ReadOperationCount;
    public ulong WriteOperationCount;
    public ulong OtherOperationCount;
    public ulong ReadTransferCount;
    public ulong WriteTransferCount;
    public ulong OtherTransferCount;
  }}
  [DllImport("kernel32.dll", SetLastError=true)]
  public static extern bool GetProcessIoCounters(
    IntPtr processHandle,
    out IoCounters counters
  );
}}
'@
Add-Type -TypeDefinition $source
$process = Get-Process -Id {pid}
$counters = New-Object NextEngineProcessIo+IoCounters
if (-not [NextEngineProcessIo]::GetProcessIoCounters($process.Handle, [ref]$counters)) {{
  throw "GetProcessIoCounters failed: $([Runtime.InteropServices.Marshal]::GetLastWin32Error())"
}}
[ordered]@{{
  process_peak_working_set_bytes = [uint64]$process.PeakWorkingSet64
  io_read_bytes = [uint64]$counters.ReadTransferCount
  io_write_bytes = [uint64]$counters.WriteTransferCount
}} | ConvertTo-Json -Compress"#
    );
    let output = run_command(
        "powershell",
        &["-NoProfile", "-NonInteractive", "-Command", &script],
    );
    #[derive(Deserialize)]
    struct Probe {
        process_peak_working_set_bytes: Option<u64>,
        io_read_bytes: Option<u64>,
        io_write_bytes: Option<u64>,
    }
    match output.and_then(|bytes| {
        serde_json::from_slice::<Probe>(&bytes).map_err(|error| error.to_string())
    }) {
        Ok(probe) => PerformanceResourceCountersV4 {
            process_peak_working_set_bytes: probe.process_peak_working_set_bytes,
            io_read_bytes: probe.io_read_bytes,
            io_write_bytes: probe.io_write_bytes,
            unavailable: vec![
                "device residency requires a representative Vulkan workload".to_owned(),
                "Vulkan timestamps require a representative render workload".to_owned(),
            ],
            ..PerformanceResourceCountersV4::default()
        },
        Err(error) => PerformanceResourceCountersV4 {
            unavailable: vec![
                format!("process counters unavailable: {error}"),
                "device residency requires a representative Vulkan workload".to_owned(),
                "Vulkan timestamps require a representative render workload".to_owned(),
            ],
            ..PerformanceResourceCountersV4::default()
        },
    }
}

pub fn finish_process_counters(
    before: &PerformanceResourceCountersV4,
) -> PerformanceResourceCountersV4 {
    let after = inspect_process_counters();
    let io_read_bytes = counter_delta(before.io_read_bytes, after.io_read_bytes);
    let io_write_bytes = counter_delta(before.io_write_bytes, after.io_write_bytes);
    let mut unavailable = after.unavailable;
    for diagnostic in &before.unavailable {
        if !unavailable.contains(diagnostic) {
            unavailable.push(diagnostic.clone());
        }
    }
    if io_read_bytes.is_none() || io_write_bytes.is_none() {
        unavailable.push(
            "process I/O deltas require successful counters before and after the workload"
                .to_owned(),
        );
    }
    PerformanceResourceCountersV4 {
        process_peak_working_set_bytes: after.process_peak_working_set_bytes,
        device_resident_bytes: after.device_resident_bytes,
        io_read_bytes,
        io_write_bytes,
        logical_resource_charges: after.logical_resource_charges,
        vulkan_timestamp_queries: after.vulkan_timestamp_queries,
        unavailable,
    }
}

fn counter_delta(before: Option<u64>, after: Option<u64>) -> Option<u64> {
    after
        .zip(before)
        .and_then(|(after, before)| after.checked_sub(before))
}

fn inspect_nvidia() -> Result<NvidiaProbe, String> {
    let query = "--query-gpu=name,memory.total,driver_version,utilization.gpu,clocks_event_reasons.sw_thermal_slowdown,clocks_event_reasons.hw_thermal_slowdown";
    let output = run_command("nvidia-smi", &[query, "--format=csv,noheader,nounits"])?;
    let line = String::from_utf8(output)
        .map_err(|error| format!("PERF_GPU_PROBE_INVALID_UTF8: {error}"))?;
    let fields = line
        .lines()
        .next()
        .ok_or_else(|| "PERF_GPU_PROBE_EMPTY".to_owned())?
        .split(',')
        .map(str::trim)
        .collect::<Vec<_>>();
    if fields.len() != 6 {
        return Err(format!(
            "PERF_GPU_PROBE_INVALID: expected 6 fields, got {}",
            fields.len()
        ));
    }
    let software_thermal = parse_active(fields[4])?;
    let hardware_thermal = parse_active(fields[5])?;
    Ok(NvidiaProbe {
        gpu_model: fields[0].to_owned(),
        gpu_vram_mib: fields[1]
            .parse()
            .map_err(|error| format!("PERF_GPU_VRAM_INVALID: {error}"))?,
        gpu_driver: fields[2].to_owned(),
        gpu_load_percent: Some(
            fields[3]
                .parse()
                .map_err(|error| format!("PERF_GPU_LOAD_INVALID: {error}"))?,
        ),
        gpu_thermal_slowdown_active: Some(software_thermal || hardware_thermal),
    })
}

fn parse_active(value: &str) -> Result<bool, String> {
    match value {
        "Active" => Ok(true),
        "Not Active" => Ok(false),
        _ => Err(format!("PERF_GPU_THERMAL_STATUS_INVALID: {value}")),
    }
}

fn parse_power_plan(value: &str) -> String {
    value
        .rsplit_once('(')
        .and_then(|(_, suffix)| suffix.strip_suffix(')'))
        .map(str::trim)
        .unwrap_or(value.trim())
        .to_owned()
}

fn normalize(value: &str) -> String {
    value
        .chars()
        .filter(|character| character.is_ascii_alphanumeric())
        .flat_map(char::to_lowercase)
        .collect()
}

fn run_command(program: &str, arguments: &[&str]) -> Result<Vec<u8>, String> {
    let output = Command::new(program)
        .args(arguments)
        .output()
        .map_err(|error| format!("PERF_HOST_PROBE_FAILED: {program}: {error}"))?;
    if output.status.success() {
        Ok(output.stdout)
    } else {
        Err(format!(
            "PERF_HOST_PROBE_FAILED: {program}: {}",
            String::from_utf8_lossy(&output.stderr).trim()
        ))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn linux_cpuinfo_counts_physical_cores_and_logical_threads() {
        let fixture = "\
processor : 0\n\
model name : Example CPU\n\
physical id : 0\n\
core id : 0\n\
cpu cores : 2\n\n\
processor : 1\n\
model name : Example CPU\n\
physical id : 0\n\
core id : 0\n\
cpu cores : 2\n\n\
processor : 2\n\
model name : Example CPU\n\
physical id : 0\n\
core id : 1\n\
cpu cores : 2\n\n\
processor : 3\n\
model name : Example CPU\n\
physical id : 0\n\
core id : 1\n\
cpu cores : 2\n";
        assert_eq!(
            parse_linux_cpuinfo(fixture),
            Ok(("Example CPU".to_owned(), 2, 4))
        );
    }

    #[test]
    fn linux_meminfo_and_load_are_normalized_to_typed_evidence() {
        let meminfo = "MemTotal:       33554432 kB\nMemAvailable:   12582912 kB\n";
        assert_eq!(
            parse_linux_meminfo_bytes(meminfo, "MemTotal"),
            Ok(34_359_738_368)
        );
        assert_eq!(
            parse_linux_meminfo_bytes(meminfo, "MemAvailable"),
            Ok(12_884_901_888)
        );
        assert_eq!(parse_linux_load_percent("3.20 2.00 1.00 1/1 1", 32), Ok(10));
    }

    #[test]
    fn linux_release_and_root_device_parsers_reject_incomplete_input() {
        let release = "NAME=Example\nPRETTY_NAME=\"Example Linux 1\"\nVERSION_ID=\"1\"\n";
        assert_eq!(
            parse_linux_os_release_value(release, "PRETTY_NAME"),
            Ok("Example Linux 1".to_owned())
        );
        assert_eq!(
            parse_linux_os_release_value(release, "VERSION_ID"),
            Ok("1".to_owned())
        );
        assert!(parse_linux_os_release_value(release, "MISSING").is_err());

        let mountinfo = "36 26 259:6 / / rw,relatime - ext4 /dev/nvme1n1p2 rw\n";
        assert_eq!(parse_linux_root_device_id(mountinfo), Ok("259:6"));
        assert!(parse_linux_root_device_id("36 26 0:1 / /tmp rw - tmpfs tmpfs rw\n").is_err());
    }
}
