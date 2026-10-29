//! Production analyzer operations through ordinary crate dependencies.
//!
//! Each module states its quantified domain and caller boundary. Native value inputs
//! are normalized unless a harness explicitly admits otherwise. Scalar metadata
//! proofs do not imply recursive expression or source-to-result correctness.
//! Reference definitions and their specification basis live in veryl_formal_semantics.

#[cfg(kani)]
mod arithmetic;

#[cfg(kani)]
mod comparison;
#[cfg(kani)]
mod conditional;
#[cfg(kani)]
mod gates;

#[cfg(kani)]
mod partial_assignment;
#[cfg(kani)]
mod resize;
#[cfg(kani)]
mod selection;
#[cfg(kani)]
mod shift;

#[cfg(kani)]
mod context;

#[cfg(kani)]
mod nonlinear;

#[cfg(kani)]
mod layout;

#[cfg(kani)]
mod coordinates;

#[cfg(kani)]
mod extension;

#[cfg(kani)]
mod public_comparison;

#[cfg(kani)]
mod public_gates;

#[cfg(kani)]
mod public_shift;

#[cfg(kani)]
mod public_wide_resize;

#[cfg(kani)]
mod public_big_getters;

#[cfg(kani)]
mod public_big_amount;
