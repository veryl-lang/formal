//! Executable reference semantics independent of production and verifier code.
//!
//! Inherited language rules cite IEEE Std 1800-2023 in their defining modules.
//! DOI: 10.1109/IEEESTD.2024.10458102. Storage, layout, and API rules are explicitly
//! internal contracts. Specification interpretation requires independent review;
//! production/reference equivalence does not prove that interpretation or whole
//! compiler conformance. Simulator agreement does not override normative rules.
//!
//! Keep the rule, its edition-specific basis, caller assumptions, and unresolved
//! interpretation beside the definition. Share models by semantic domain, rather
//! than reproducing production algorithms.

pub mod arithmetic;
pub mod comparison;
pub mod conditional;
pub mod gates;
pub mod logic4;
pub mod partial_assignment;
pub mod resize;
pub mod selection;
pub mod shift;

pub mod context;

pub mod nonlinear;

pub mod layout;

pub mod coordinates;

pub mod aggregate;

pub mod words;
pub mod write_log;

pub mod scalar;

pub mod bitvector;

pub mod expressions;
