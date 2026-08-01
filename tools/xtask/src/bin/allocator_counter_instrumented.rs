#![forbid(unsafe_code)]

#[global_allocator]
static PROCESS_ALLOCATOR: next_process_allocation_counter::ProcessAllocationCounter =
    next_process_allocation_counter::ProcessAllocationCounter::system();

fn main() {
    if let Err(error) =
        xtask::allocator_counter_kernel::run_instrumented_helper(std::env::args().skip(1))
    {
        eprintln!("{error}");
        std::process::exit(1);
    }
}
