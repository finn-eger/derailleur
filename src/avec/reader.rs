//! Reader-based decoder implementation.
//!
//! _Requires Cargo feature `std`._

use std::{io::Read, vec::Vec};

use either::Either::{self, Left, Right};
use thiserror::Error;

use crate::sans::{
    Decoder,
    check::compute_crc,
    data::{AnyField, Field, FieldInner},
    definition::{Definition, DefinitionAlt, DefinitionFieldAlt},
    header::{DocumentHeaderError, RecordHeader, RecordHeaderError},
};

use super::{FromRecord, FromRecords};

extern crate std;

/// Errors occurring while decoding from a reader.
#[derive(Debug, Error)]
pub enum Error {
    /// An error from the supplied reader.
    #[error(transparent)]
    Io(#[from] std::io::Error),
    /// Calculated and found CRC values do not match.
    #[error("Calculated ({calculated}) and found ({found}) CRC values do not match.")]
    CyclicRedundancyCheck { found: u16, calculated: u16 },
    /// Incorrect file header.
    #[error("Incorrect file header: {0}.")]
    Header(#[from] DocumentHeaderError),
    /// Found unsupported developer data.
    #[error("Found unsupported developer data.")]
    Developer,
}

impl From<RecordHeaderError> for Error {
    fn from(err: RecordHeaderError) -> Self {
        match err {
            RecordHeaderError::DeveloperData => Self::Developer,
        }
    }
}

/// Decode records from a reader of a document, publishing to a receiver.
///
/// This method is also re-exported as `derailleur::avec::decode_reader`.
///
/// _Requires Cargo feature `std`._
pub fn decode(r: &mut impl Read, o: &mut impl FromRecords) -> Result<(), Error> {
    let i = &mut 0; // Counter of bytes read, used to end decoding.
    let c = &mut 0; // Cyclic redundancy check accumulator value.

    let (size, successor) = Decoder::advance(take(r, Some((i, c)))?)?;

    let mut record_header = match successor {
        Left(state) => state.advance(take(r, Some((i, c)))?),
        Right(state) => state,
    };

    let end = *i + size as usize; // Offset to the end of the record section.

    // Store of previous definition records, used to decode data records.
    let mut definitions: [_; 16] = Default::default();

    while *i < end {
        let (local, successor) = record_header.advance(take(r, Some((i, c)))?)?;

        record_header = match successor {
            Left(state) => {
                let d = &mut definitions[local as usize];
                decode_definition(state, r, i, c, d)?
            }
            Right((time, state)) => {
                let d = &mut definitions[local as usize].as_slice();
                decode_data(state, time, r, i, c, d, o)?
            }
        };
    }

    let calculated = *c;
    let found = u16::from_le_bytes(take(r, None)?);

    if found != calculated {
        Err(Error::CyclicRedundancyCheck { found, calculated })?;
    }

    Ok(())
}

fn decode_definition(
    state: Definition,
    r: &mut impl Read,
    i: &mut usize,
    c: &mut u16,
    d: &mut Vec<u8>,
) -> Result<RecordHeader, Error> {
    d.clear();

    let bytes = take(r, Some((i, c)))?;
    d.extend_from_slice(&bytes);

    let record_header = match state.advance(bytes) {
        Left(mut state) => loop {
            let bytes = take(r, Some((i, c)))?;
            d.extend_from_slice(&bytes);

            state = match state.advance(bytes) {
                Left(state) => state,
                Right(state) => break state,
            };
        },
        Right(state) => state,
    };

    Ok(record_header)
}

fn decode_data(
    state: DefinitionAlt,
    time: Option<u8>,
    r: &mut impl Read,
    i: &mut usize,
    c: &mut u16,
    d: &mut &[u8],
    o: &mut impl FromRecords,
) -> Result<RecordHeader, Error> {
    let (global, successor) = state.advance(take(d, None)?);

    // Shadow the document receiver with that of a single record.
    let mut o = o.add_record(global);

    if let (Some(o), Some(time)) = (&mut o, time) {
        o.add_time_offset(time);
    }

    let record_header = match successor {
        Left(mut state) => loop {
            let (f, inner_state) = state.advance(take(d, None)?);

            let o = o.as_deref_mut();

            fn decode_field<
                T: FieldInner<From = [u8; N]>,
                O: FromRecord + ?Sized,
                const N: usize,
            >(
                mut state: Field<T>,
                r: &mut impl Read,
                i: &mut usize,
                c: &mut u16,
                f: u8,

                mut o: Option<&mut O>,
                add: fn(&mut O, u8, T::Into),
            ) -> Result<Either<DefinitionFieldAlt, RecordHeader>, Error> {
                loop {
                    let (value, successor) = state.advance(take(r, Some((i, c)))?);

                    if let (Some(o), Some(value)) = (&mut o, value) {
                        add(o, f, value);
                    }

                    state = match successor {
                        Left(successor) => return Ok(successor),
                        Right(state) => state,
                    }
                }
            }

            let successor = match inner_state {
                AnyField::U8(s) => decode_field(s, r, i, c, f, o, FromRecord::add_u8),
                AnyField::U8Z(s) => decode_field(s, r, i, c, f, o, FromRecord::add_u8),
                AnyField::U16(s) => decode_field(s, r, i, c, f, o, FromRecord::add_u16),
                AnyField::U16Z(s) => decode_field(s, r, i, c, f, o, FromRecord::add_u16),
                AnyField::U32(s) => decode_field(s, r, i, c, f, o, FromRecord::add_u32),
                AnyField::U32Z(s) => decode_field(s, r, i, c, f, o, FromRecord::add_u32),
                AnyField::U64(s) => decode_field(s, r, i, c, f, o, FromRecord::add_u64),
                AnyField::U64Z(s) => decode_field(s, r, i, c, f, o, FromRecord::add_u64),

                AnyField::I8(s) => decode_field(s, r, i, c, f, o, FromRecord::add_i8),
                AnyField::I16(s) => decode_field(s, r, i, c, f, o, FromRecord::add_i16),
                AnyField::I32(s) => decode_field(s, r, i, c, f, o, FromRecord::add_i32),
                AnyField::I64(s) => decode_field(s, r, i, c, f, o, FromRecord::add_i64),

                AnyField::F32(s) => decode_field(s, r, i, c, f, o, FromRecord::add_f32),
                AnyField::F64(s) => decode_field(s, r, i, c, f, o, FromRecord::add_f64),
            }?;

            state = match successor {
                Left(state) => state,
                Right(state) => break state,
            };
        },
        Right(state) => state,
    };

    Ok(record_header)
}

/// Take an exact number of bytes from a reader, optionally advancing a counter
/// and accumulating a CRC value.
fn take<const N: usize>(
    r: &mut impl Read,
    ic: Option<(&mut usize, &mut u16)>,
) -> Result<[u8; N], Error> {
    let mut buf = [0; N];
    r.read_exact(&mut buf)?;

    if let Some((i, c)) = ic {
        *i += N;
        *c = compute_crc(*c, &buf);
    }

    Ok(buf)
}
