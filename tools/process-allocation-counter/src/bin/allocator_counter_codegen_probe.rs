#![forbid(unsafe_code)]

//! Minimal release-codegen probe for the tooling-only process allocator.
//!
//! Keeping this probe in the std-only allocator package makes the evidence
//! build independent of the full `xtask` dependency graph. The production
//! helper and this probe install the same `ProcessAllocationCounter` type;
//! source-boundary checks separately pin that production hook.

#[global_allocator]
static PROCESS_ALLOCATOR: next_process_allocation_counter::ProcessAllocationCounter =
    next_process_allocation_counter::ProcessAllocationCounter::system();

fn main() {
    let allocation = std::hint::black_box(Box::new(41_u64));
    std::hint::black_box(allocation);
}
