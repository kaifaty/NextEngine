//! Optional Tracy stage zones for developer profiling.
//!
//! Telemetry only (SPEC-23: measured data never selects an authoritative
//! result). With the `profile-tracy` feature off the Tracy client is not
//! linked and `stage_zone!` expands to nothing, so the default build carries
//! zero zone overhead and byte-identical code paths.
//!
//! Each invocation creates one guard that ends at the end of its enclosing
//! block scope. Place at most one zone per block scope: a second `let`
//! binding in the same scope would shadow the first guard without ending it
//! early, silently producing wrong nesting.

#[cfg(feature = "profile-tracy")]
macro_rules! stage_zone {
    ($name:literal) => {
        let _stage_zone = ::tracy_client::span!($name);
    };
}

#[cfg(not(feature = "profile-tracy"))]
macro_rules! stage_zone {
    ($name:literal) => {};
}

pub(crate) use stage_zone;
