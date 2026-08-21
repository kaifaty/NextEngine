//! The single reviewed `unsafe` boundary of ADR-093: apply one CPU mask to
//! the calling thread and verify it by reading the mask back.
//!
//! The mask uses the Linux kernel affinity ABI directly (little-endian one
//! bit per logical CPU, `CPU_SETSIZE` bits wide), which is exactly what
//! glibc's `cpu_set_t` wraps; no platform-specific helper macros are needed.

use super::AffinityError;

/// `CPU_SETSIZE` bits as sixty-four 64-bit words.
const MASK_WORDS: usize = libc::CPU_SETSIZE as usize / 64;

const _: () = assert!(
    std::mem::size_of::<libc::c_ulong>() == std::mem::size_of::<u64>(),
    "kernel affinity masks are arrays of 64-bit words on supported Linux targets"
);

/// Pin the calling thread to exactly one logical CPU and verify the mask.
pub fn pin_current_thread_to(cpu_id: u32) -> Result<(), AffinityError> {
    #[cfg(target_os = "linux")]
    {
        pin_linux(cpu_id)
    }
    #[cfg(not(target_os = "linux"))]
    {
        let _ = cpu_id;
        Err(AffinityError::UnsupportedPlatform)
    }
}

#[cfg(target_os = "linux")]
fn pin_linux(cpu_id: u32) -> Result<(), AffinityError> {
    let requested = mask_for(cpu_id)?;
    // SAFETY: `requested` is a valid affinity mask of `CPU_SETSIZE` bits and
    // the syscall only reads it. This is one half of the entire ADR-093
    // unsafe surface.
    let applied = unsafe {
        libc::sched_setaffinity(
            0,
            std::mem::size_of_val(&requested),
            requested.as_ptr().cast(),
        )
    };
    if applied != 0 {
        return Err(AffinityError::PinFailed(
            std::io::Error::last_os_error().raw_os_error().unwrap_or(0),
        ));
    }

    let mut verified = [0_u64; MASK_WORDS];
    // SAFETY: `verified` is a writable affinity mask the syscall fills; the
    // read-back proves the kernel accepted exactly the requested mask.
    let read_back = unsafe {
        libc::sched_getaffinity(
            0,
            std::mem::size_of_val(&verified),
            verified.as_mut_ptr().cast(),
        )
    };
    if read_back != 0 {
        return Err(AffinityError::PinFailed(
            std::io::Error::last_os_error().raw_os_error().unwrap_or(0),
        ));
    }
    if verified != requested {
        return Err(AffinityError::PinVerificationMismatch);
    }
    Ok(())
}

#[cfg(target_os = "linux")]
fn mask_for(cpu_id: u32) -> Result<[u64; MASK_WORDS], AffinityError> {
    let index = usize::try_from(cpu_id).map_err(|_| AffinityError::InvalidTopology("cpu id"))?;
    if index >= MASK_WORDS * 64 {
        return Err(AffinityError::InvalidTopology("cpu id exceeds CPU_SETSIZE"));
    }
    let mut mask = [0_u64; MASK_WORDS];
    mask[index / 64] |= 1_u64 << (index % 64);
    Ok(mask)
}
