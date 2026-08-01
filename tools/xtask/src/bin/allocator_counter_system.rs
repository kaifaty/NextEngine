#![forbid(unsafe_code)]

fn main() {
    if let Err(error) = xtask::allocator_counter_kernel::run_system_helper(std::env::args().skip(1))
    {
        eprintln!("{error}");
        std::process::exit(1);
    }
}
