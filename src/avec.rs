//! Convenience interfaces for common decoding patterns.
//!
//! The functions in this module are suited to decoding records from files and
//! data slices, publishing to the [`FromRecords`] and [`FromRecord`] traits.
//!
//! In many cases (when records are of a known shape), these traits can be
//! derived. See the [`FromRecords`](macro@FromRecords) and
//! [`FromRecord`](macro@FromRecord) macros for details.

#[cfg(feature = "std")]
pub mod reader;
pub mod slice;

#[cfg(feature = "std")]
pub use reader::decode as decode_reader;
pub use slice::decode as decode_slice;

/// Derive [`FromRecords`] for a struct holding a collection of records.
///
/// _Requires Cargo feature `derive`._
///
/// # Example
///
/// To collect a single record, add the `record(N)` attribute to an `Option<T>`
/// struct field, where `N` is the global message number and `T` is a type
/// implementing [`Record`] and [`Default`]. Additional records received for the
/// same message number will overwrite earlier ones. To collect multiple
/// occurrences of a record, apply the attribute to a `Vec<T>` instead.
///
/// ```
/// #[derive(Debug, Default, FromRecords)]
/// struct ActivityRecordSet {
///     #[record(0)]
///     file_id: Option<FileId>,
///     #[record(20)]
///     records: Vec<Record>,
/// }
/// ```
#[cfg(feature = "derive")]
pub use derailleur_derive::FromRecords;

/// Produce record receivers for a document.
///
/// See the [`FromRecords`](macro@FromRecords) derive macro for an automatic
/// implementation of this trait.
pub trait FromRecords {
    /// Retrieve a receiver for a record, if one exists.
    fn add_record(&mut self, id: u16) -> Option<&mut dyn FromRecord>;
}

/// Derive [`FromRecord`] for a struct representing a single record.
///
/// _Requires Cargo feature `derive`._
///
/// # Examples
///
/// To receive a single value for a record field, add the `field(N)` attribute
/// to an `Option<T>` struct field, where `N` is the field number and `T` is the
/// corresponding Rust primitive. Additional values received for the same field
/// will replace earlier ones.
///
/// To receive the time offset stored in compressed timestamp headers, supply
/// `time` in place of a field number.
///
/// ```
/// #[derive(Debug, Default, FromRecord)]
/// struct Record {
///     #[field(time)]
///     time_offset: Option<u8>,
///     #[field(0)]
///     position_lat: Option<i32>,
///     #[field(1)]
///     position_long: Option<i32>,
///     #[field(2)]
///     altitude: Option<u16>,
/// }
/// ```
///
/// Rather than decoding directly into domain types, it's recommended to store
/// the received primitives and process them afterward in an accessor.
///
/// ```
/// impl Record {
///     fn position(&self) -> Option<(f32, f32)> {
///         if let (Some(lat), Some(long)) = (self.position_lat, self.position_long) {
///             // Convert from the stored integers to floating point degrees.
///             let lat = (lat as f32 * 180.0) / (i32::MAX as f32);
///             let long = (long as f32 * 180.0) / (i32::MAX as f32);
///
///             Some(Coordinate {
///                 latitude_deg: lat,
///                 longitude_deg: long,
///             })
///         } else {
///             None
///         }
///     }
/// }
/// ```
///
/// To receive arrays or arbitrary types (for example, decoding directly into an
/// enumeration), supply an accumulator closure. Since the element type cannot
/// be inferred, the second argument must be typed.
///
/// ```
/// #[derive(Debug, Default, FromRecord)]
/// struct Course {
///     #[field(5, |v, c: u8| v.push(c))]
///     name: Option<Vec<u8>>,
/// }
/// ```
///
/// Keep in mind that a UTF-8 string cannot be built byte-by-byte, as a single
/// Unicode code point can span multiple bytes. Instead, collect into a buffer,
/// and convert this buffer later.
///
/// ```
/// impl Course {
///     fn name(&self) -> Option<&str> {
///         self.name
///             .as_ref()
///             .and_then(|v| std::str::from_utf8(v).ok())
///     }
/// }
/// ```
#[cfg(feature = "derive")]
pub use derailleur_derive::FromRecord;

/// Receive field values for a record.
///
/// Before publishing, fields are converted to their corresponding Rust
/// primitive, and those holding the 'invalid' marker value are skipped. Array
/// types (including strings) are published item-by-item, calling the receiver
/// repeatedly.
///
/// The default implementation of each method ignores received values.
///
/// See the [`FromRecord`](macro@FromRecord) derive macro for an automatic
/// implementation of this trait.
#[allow(unused_variables)]
pub trait FromRecord {
    /// Add the compressed time offset to the record.
    fn add_time_offset(&mut self, _: u8) {}
    /// Add a `u8` for a field to the record.
    ///
    /// This method receives values for fields represented by a, or an array of,
    /// unsigned bytes. This includes the base types `enum`, `string`, and
    /// `byte`.
    fn add_u8(&mut self, field: u8, _: u8) {}
    /// Add a `u16` for a field to the record.
    fn add_u16(&mut self, field: u8, _: u16) {}
    /// Add a `u32` for a field to the record.
    fn add_u32(&mut self, field: u8, _: u32) {}
    /// Add a `u64` for a field to the record.
    fn add_u64(&mut self, field: u8, _: u64) {}

    /// Add a `i8` for a field to the record.
    fn add_i8(&mut self, field: u8, _: i8) {}
    /// Add a `i16` for a field to the record.
    fn add_i16(&mut self, field: u8, _: i16) {}
    /// Add a `i32` for a field to the record.
    fn add_i32(&mut self, field: u8, _: i32) {}
    /// Add a `i64` for a field to the record.
    fn add_i64(&mut self, field: u8, _: i64) {}

    /// Add a `f32` for a field to the record.
    fn add_f32(&mut self, field: u8, _: f32) {}
    /// Add a `f64` for a field to the record.
    fn add_f64(&mut self, field: u8, _: f64) {}
}
