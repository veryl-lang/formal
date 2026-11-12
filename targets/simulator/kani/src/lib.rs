//! Production binary storage and write-log helpers through normal dependencies.
//!
//! Proofs assume a little-endian 64-bit target and valid allocations for the stated
//! unsafe API extents. Exact alias is admitted where stated; partial overlap is not.
//! Four-state source execution and generated backends have separate comparisons.
//! Safety and unwinding checks remain enabled; each module states its finite bounds.

#[cfg(kani)]
mod arithmetic;
#[cfg(kani)]
mod bitwise;
#[cfg(kani)]
mod commit;
#[cfg(kani)]
mod memory;
#[cfg(kani)]
mod shift;
#[cfg(kani)]
mod signed_shift;
#[cfg(kani)]
mod window;
#[cfg(kani)]
mod window_log;

#[cfg(kani)]
mod window_log_pool;

#[cfg(kani)]
mod window_log_chunks;

#[cfg(kani)]
mod window_log_variable;

#[cfg(kani)]
mod window_commit;

#[cfg(kani)]
mod window_history;

#[cfg(kani)]
mod window_from;

#[cfg(kani)]
mod dual_slot_window;

#[cfg(kani)]
mod dual_slot_wide;

#[cfg(kani)]
mod window_read;

#[cfg(kani)]
mod window_read_source;

#[cfg(kani)]
mod window_from_variable;

#[cfg(kani)]
mod window_from_chunks;
