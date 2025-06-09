# Derailleur

An efficient deserializer for Garmin's Flexible and Interoperable Data Transfer
protocol.

Derailleur provides a set of ergonomic interfaces for common decoding patterns,
and exposes its underlying finite-state machine for applications needing finer
control over internals (such as those running on embedded systems).

Derailleur is compatible with `#![no_std]` and `#![no_alloc]`, and has been
architected to use very little memory.

For usage details and explanatory notes, refer to the [documentation][Docs.rs].

## Limitations

- Documents with 'developer data' are not yet supported, and will yield an error.

[Docs.rs]: https://docs.rs/derailleur/latest
