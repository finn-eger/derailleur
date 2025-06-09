//! States processing definition records.

use core::marker::PhantomData;

use either::Either::{self, Left, Right};
use zerocopy::FromBytes;

use crate::sans::data::Field;

use super::{data::AnyField, header::RecordHeader};

/// State token to perform a first-pass decoding of a definition message.
#[derive(Debug)]
pub struct Definition(pub(super) ());

impl Definition {
    /// Transition to another state by performing a first-pass decoding of a
    /// definition message.
    ///
    /// Returns a successor state token.
    pub fn advance(self, r: [u8; 5]) -> Either<DefinitionField, RecordHeader> {
        let DefinitionMessage {
            fields_remaining, ..
        } = zerocopy::transmute!(r);

        if fields_remaining != 0 {
            Left(DefinitionField { fields_remaining })
        } else {
            Right(RecordHeader(()))
        }
    }
}

/// State token to perform a first-pass decoding of a definition field.
#[derive(Debug)]
pub struct DefinitionField {
    pub(super) fields_remaining: u8,
}

impl DefinitionField {
    /// Transition to another state by performing a first-pass decoding of a
    /// definition field.
    ///
    /// Returns a successor state token.
    pub fn advance(self, _r: [u8; 3]) -> Either<DefinitionField, RecordHeader> {
        let fields_remaining = self.fields_remaining - 1;

        if fields_remaining != 0 {
            Left(DefinitionField { fields_remaining })
        } else {
            Right(RecordHeader(()))
        }
    }
}

/// State token to decode a definition message.
pub struct DefinitionAlt(pub(super) ());

#[repr(C, packed)]
#[derive(Debug, FromBytes)]
struct DefinitionMessage {
    _reserved: u8,
    architecture: u8,
    global_message: [u8; 2],
    fields_remaining: u8,
}

impl DefinitionAlt {
    /// Transition to another state by decoding a definition message.
    ///
    /// **This method expects bytes not read from the tip of the cursor.** See
    /// the architecture description in the [`crate::sans`] module documentation
    /// for clarification.
    ///
    /// Returns the global message number, and a successor state token.
    pub fn advance(self, r: [u8; 5]) -> (u16, Either<DefinitionFieldAlt, RecordHeader>) {
        let DefinitionMessage {
            architecture,
            global_message,
            fields_remaining,
            ..
        } = zerocopy::transmute!(r);

        let is_little_endian = architecture == 0;
        let global_message = if is_little_endian {
            u16::from_le_bytes(global_message)
        } else {
            u16::from_be_bytes(global_message)
        };

        let successor = if fields_remaining != 0 {
            Left(DefinitionFieldAlt {
                fields_remaining,
                is_little_endian,
            })
        } else {
            Right(RecordHeader(()))
        };

        (global_message, successor)
    }
}

/// State token to decode a definition field.
#[derive(Debug)]
pub struct DefinitionFieldAlt {
    pub(super) fields_remaining: u8,
    pub(super) is_little_endian: bool,
}

impl DefinitionFieldAlt {
    /// Transition to another state by decoding a definition field.
    ///
    /// **This method expects bytes not read from the tip of the cursor.** See
    /// the architecture description in the [`crate::sans`] module documentation
    /// for clarification.
    ///
    /// Returns the field number, and the successor state.
    pub fn advance(self, r: [u8; 3]) -> (u8, AnyField) {
        #[repr(C, packed)]
        #[derive(FromBytes)]
        struct FieldHeader {
            field: u8,
            size: u8,
            base_type: u8,
        }

        let FieldHeader {
            field,
            size,
            base_type,
        } = zerocopy::transmute!(r);

        fn new_any_field<T>(
            (fields_remaining, is_little_endian, bytes_remaining): (u8, bool, u8),
        ) -> Field<T> {
            Field {
                fields_remaining,
                bytes_remaining,
                is_little_endian,
                _phantom: PhantomData,
            }
        }

        let parameters = (self.fields_remaining - 1, self.is_little_endian, size);

        let successor = match base_type {
            0x00 => AnyField::U8(new_any_field(parameters)),
            0x01 => AnyField::I8(new_any_field(parameters)),
            0x02 => AnyField::U8(new_any_field(parameters)),
            0x83 => AnyField::I16(new_any_field(parameters)),
            0x84 => AnyField::U16(new_any_field(parameters)),
            0x85 => AnyField::I32(new_any_field(parameters)),
            0x86 => AnyField::U32(new_any_field(parameters)),
            0x07 => AnyField::U8Z(new_any_field(parameters)),
            0x88 => AnyField::F32(new_any_field(parameters)),
            0x89 => AnyField::F64(new_any_field(parameters)),
            0x0A => AnyField::U8Z(new_any_field(parameters)),
            0x8B => AnyField::U16Z(new_any_field(parameters)),
            0x8C => AnyField::U32Z(new_any_field(parameters)),
            0x0D => AnyField::U8(new_any_field(parameters)),
            0x8E => AnyField::I64(new_any_field(parameters)),
            0x8F => AnyField::U64(new_any_field(parameters)),
            0x90 => AnyField::U64Z(new_any_field(parameters)),
            _ => unreachable!(),
        };

        (field, successor)
    }
}
