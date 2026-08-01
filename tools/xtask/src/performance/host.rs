use std::process::Command;

use serde::Deserialize;

use super::{
    MINIMUM_FREE_RAM_BYTES, PerformancePreflightV1, PerformanceResourceCountersV2,
    PerformanceTargetFingerprintV1, THOTH_TARGET_ID,
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
    if !cfg!(target_os = "windows") {
        return Err("PERF_TARGET_FINGERPRINT_UNSUPPORTED_HOST".to_owned());
    }

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
    validate_preflight(&mut preflight);
    Ok((fingerprint, preflight))
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
    if fingerprint.gpu_driver != "591.86" {
        diagnostics.push("PERF_GPU_DRIVER_MISMATCH".to_owned());
    }
    if normalize(&fingerprint.power_plan) != "amdryzenhighperformance" {
        diagnostics.push("PERF_POWER_PLAN_MISMATCH".to_owned());
    }
    diagnostics
}

pub fn inspect_process_counters() -> PerformanceResourceCountersV2 {
    if !cfg!(target_os = "windows") {
        return PerformanceResourceCountersV2 {
            unavailable: vec![
                "process residency counters are not implemented on this report-only host"
                    .to_owned(),
                "Vulkan timestamps require a representative render workload".to_owned(),
            ],
            ..PerformanceResourceCountersV2::default()
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
  host_resident_bytes = [uint64]$process.WorkingSet64
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
        host_resident_bytes: Option<u64>,
        io_read_bytes: Option<u64>,
        io_write_bytes: Option<u64>,
    }
    match output.and_then(|bytes| {
        serde_json::from_slice::<Probe>(&bytes).map_err(|error| error.to_string())
    }) {
        Ok(probe) => PerformanceResourceCountersV2 {
            host_resident_bytes: probe.host_resident_bytes,
            io_read_bytes: probe.io_read_bytes,
            io_write_bytes: probe.io_write_bytes,
            unavailable: vec![
                "device residency requires a representative Vulkan workload".to_owned(),
                "Vulkan timestamps require a representative render workload".to_owned(),
            ],
            ..PerformanceResourceCountersV2::default()
        },
        Err(error) => PerformanceResourceCountersV2 {
            unavailable: vec![
                format!("process counters unavailable: {error}"),
                "device residency requires a representative Vulkan workload".to_owned(),
                "Vulkan timestamps require a representative render workload".to_owned(),
            ],
            ..PerformanceResourceCountersV2::default()
        },
    }
}

pub fn finish_process_counters(
    before: &PerformanceResourceCountersV2,
) -> PerformanceResourceCountersV2 {
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
    PerformanceResourceCountersV2 {
        host_resident_bytes: after.host_resident_bytes,
        device_resident_bytes: after.device_resident_bytes,
        io_read_bytes,
        io_write_bytes,
        allocator_allocated_bytes: after.allocator_allocated_bytes,
        allocator_allocation_count: after.allocator_allocation_count,
        allocator_counter: after.allocator_counter,
        vulkan_timestamp_queries: after.vulkan_timestamp_queries,
        unavailable,
    }
}

fn counter_delta(before: Option<u64>, after: Option<u64>) -> Option<u64> {
    after
        .zip(before)
        .and_then(|(after, before)| after.checked_sub(before))
}

fn validate_preflight(preflight: &mut PerformancePreflightV1) {
    if preflight
        .cpu_load_percent
        .is_none_or(|percent| percent >= 5)
    {
        preflight
            .diagnostics
            .push("PERF_CPU_NOT_IDLE_BELOW_FIVE_PERCENT".to_owned());
    }
    if preflight
        .gpu_load_percent
        .is_none_or(|percent| percent >= 5)
    {
        preflight
            .diagnostics
            .push("PERF_GPU_NOT_IDLE_BELOW_FIVE_PERCENT".to_owned());
    }
    if preflight
        .free_ram_bytes
        .is_none_or(|bytes| bytes < MINIMUM_FREE_RAM_BYTES)
    {
        preflight
            .diagnostics
            .push("PERF_FREE_RAM_BELOW_TWENTY_GIB".to_owned());
    }
    if preflight
        .cpu_clock_percent_of_maximum
        .is_none_or(|percent| percent < 80)
    {
        preflight
            .diagnostics
            .push("PERF_CPU_THROTTLING_CHECK_FAILED".to_owned());
    }
    if preflight.gpu_thermal_slowdown_active != Some(false) {
        preflight
            .diagnostics
            .push("PERF_GPU_THERMAL_SLOWDOWN_CHECK_FAILED".to_owned());
    }
    preflight.ready = preflight.diagnostics.is_empty();
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
