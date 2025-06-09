//! Internal finite-state machine for implementing decoders.
//!
//! This module is intended for advanced applications that need fine control
//! over decoder internals. See [`crate::avec`] for implementations covering
//! common decoding patterns.
//!
//! **Documentation for this module is incomplete, and the API is likely to
//! undergo significant changes before a full release. Implementing a custom
//! decoder requires a basic understanding of the structure of FIT protocol
//! data.**
//!
//! # Architecture
//!
//! All states are represented by a zero-size, non-copy token. Once enough bytes
//! are ready, transition to another state by calling the token's `advance`
//! method. This will return a successor state token, along with any extracted
//! data.
//!
//! When decoding a data record, the finite-state machine performs a second,
//! interwoven pass over the definition record. This frees implementations to
//! choose how they manage memory constraints. The bytes used to advance a
//! sequence of these `Alt`-suffixed definition states must match those used to
//! advance through their first-pass counterparts with the same field number.
//!
//! Only the initial state, re-exported for convenience as [`Decoder`], can be
//! constructed.
//!
//! This architecture enables the compiler and type system to guide applications
//! toward a correct implementation. However, some areas of the decoding process
//! are not represented in the finite-state machine and must be carefully
//! written:
//!
//! - Reading bytes from the correct place in the document, including buffering
//! or seeking as necessary.
//!
//! - Ending decoding once the specified number of document bytes have been
//! read.
//!
//! - Applying cyclic redundancy checks. A helper function is provided in the
//! [`check`] module.
//!
//! Implementers are recommended to begin by studying and modifying a decoder
//! from the [`crate::avec`] module.

pub mod check;
pub mod data;
pub mod definition;
pub mod header;

/// Entrypoint to the finite-state machine.
pub type Decoder = header::DocumentHeader;
