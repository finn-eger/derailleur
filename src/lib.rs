#![no_std]

//! An efficient deserializer for Garmin's Flexible and Interoperable Data
//! Transfer protocol.
//!
//! Derailleur provides a set of ergonomic interfaces for common decoding
//! patterns, and exposes its underlying finite-state machine for applications
//! needing finer control over internals (such as those running on embedded
//! systems).
//!
//! Most users should begin with the functions and derive macros in the [`avec`]
//! module. <!-- These are suited to extracting records, especially of a known
//! shape, from files and data slices. --> If these prove insufficient, consider
//! implementing a decoder as described in the [`sans`] module.
//!
//! ## Cargo Features
//!
//! The following crate feature flags are available:
//!
//! - `derive`: enable derive macros (default).
//! - `std`: enable reader-based decoder (default).

pub mod avec;
pub mod sans;
