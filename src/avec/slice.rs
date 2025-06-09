//! Slice-based decoder implementation.

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

/// Errors occurring while decoding from a slice.
#[derive(Debug, Error)]
pub enum Error {
    /// Unexpectedly reached the end of the slice.
    #[error("Unexpectedly reached the end of the slice.")]
    EndOfSlice,
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

/// Decode records from a slice of a document, publishing to a receiver.
///
/// This method is also re-exported as `derailleur::avec::decode_slice`.
pub fn decode(r: &[u8], o: &mut impl FromRecords) -> Result<(), Error> {
    let i = &mut 0; // Counter of bytes read, used to read bytes from the tip.

    let (size, successor) = Decoder::advance(take(r, i)?)?;

    let mut record_header = match successor {
        Left(state) => state.advance(take(r, i)?),
        Right(state) => state,
    };

    let end = *i + size as usize; // Offset to the end of the record section.

    // Apply the cyclic redundancy check before continuing.
    let found = u16::from_le_bytes(r.get(end..).ok_or(Error::EndOfSlice)?.try_into().unwrap());
    let calculated = compute_crc(0, r.get(..end).ok_or(Error::EndOfSlice)?);

    if found != calculated {
        Err(Error::CyclicRedundancyCheck { found, calculated })?;
    }

    // Store of previous definition record offsets, used to decode data records.
    let mut definition_table = [0; 16];

    while *i < end {
        let (local, successor) = record_header.advance(take(r, i)?)?;

        record_header = match successor {
            Left(state) => {
                definition_table[local as usize] = *i;
                decode_definition(state, r, i)?
            }
            Right((time, state)) => {
                let j = definition_table[local as usize];
                decode_data(state, time, r, i, j, o)?
            }
        };
    }

    Ok(())
}

fn decode_definition(state: Definition, r: &[u8], i: &mut usize) -> Result<RecordHeader, Error> {
    Ok(match state.advance(take(r, i)?) {
        Left(mut state) => loop {
            state = match state.advance(take(r, i)?) {
                Left(state) => state,
                Right(state) => break state,
            };
        },
        Right(state) => state,
    })
}

fn decode_data(
    state: DefinitionAlt,
    time: Option<u8>,
    r: &[u8],
    i: &mut usize,
    mut j: usize,
    o: &mut impl FromRecords,
) -> Result<RecordHeader, Error> {
    let (global, successor) = state.advance(take(r, &mut j)?);

    // Shadow the document receiver with that of a single record.
    let mut o = o.add_record(global);

    if let (Some(o), Some(time)) = (&mut o, time) {
        o.add_time_offset(time);
    }

    Ok(match successor {
        Left(mut state) => loop {
            let (f, inner_state) = state.advance(take(r, &mut j)?);

            let o = o.as_deref_mut();

            fn decode_field<
                T: FieldInner<From = [u8; N]>,
                O: FromRecord + ?Sized,
                const N: usize,
            >(
                mut state: Field<T>,
                r: &[u8],
                i: &mut usize,
                f: u8,

                mut o: Option<&mut O>,
                add: fn(&mut O, u8, T::Into),
            ) -> Result<Either<DefinitionFieldAlt, RecordHeader>, Error> {
                loop {
                    let (value, successor) = state.advance(take(r, i)?);

                    if let (Some(o), Some(value)) = (&mut o, value) {
                        add(o, f, value);
                    }

                    state = match successor {
                        Left(successor) => return Ok(successor),
                        Right(y) => y,
                    }
                }
            }

            let successor = match inner_state {
                AnyField::U8(s) => decode_field(s, r, i, f, o, FromRecord::add_u8),
                AnyField::U8Z(s) => decode_field(s, r, i, f, o, FromRecord::add_u8),
                AnyField::U16(s) => decode_field(s, r, i, f, o, FromRecord::add_u16),
                AnyField::U16Z(s) => decode_field(s, r, i, f, o, FromRecord::add_u16),
                AnyField::U32(s) => decode_field(s, r, i, f, o, FromRecord::add_u32),
                AnyField::U32Z(s) => decode_field(s, r, i, f, o, FromRecord::add_u32),
                AnyField::U64(s) => decode_field(s, r, i, f, o, FromRecord::add_u64),
                AnyField::U64Z(s) => decode_field(s, r, i, f, o, FromRecord::add_u64),

                AnyField::I8(s) => decode_field(s, r, i, f, o, FromRecord::add_i8),
                AnyField::I16(s) => decode_field(s, r, i, f, o, FromRecord::add_i16),
                AnyField::I32(s) => decode_field(s, r, i, f, o, FromRecord::add_i32),
                AnyField::I64(s) => decode_field(s, r, i, f, o, FromRecord::add_i64),

                AnyField::F32(s) => decode_field(s, r, i, f, o, FromRecord::add_f32),
                AnyField::F64(s) => decode_field(s, r, i, f, o, FromRecord::add_f64),
            }?;

            state = match successor {
                Left(state) => state,
                Right(state) => break state,
            };
        },
        Right(state) => state,
    })
}

/// Take an exact number of bytes from an offset in a slice, advancing the offset.
fn take<const N: usize>(r: &[u8], i: &mut usize) -> Result<[u8; N], Error> {
    let s = *i;
    *i += N;

    Ok(r.get(s..*i).ok_or(Error::EndOfSlice)?.try_into().unwrap())
}
