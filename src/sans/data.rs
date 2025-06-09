//! States processing data records.

use core::marker::PhantomData;

use either::Either::{self, Left, Right};

use super::{definition::DefinitionFieldAlt, header::RecordHeader};

/// State token to decode a field of base type `T`.
#[derive(Debug)]
pub struct Field<T> {
    pub(super) fields_remaining: u8,
    pub(super) bytes_remaining: u8,
    pub(super) is_little_endian: bool,
    pub(super) _phantom: PhantomData<T>,
}

impl<T: FieldInner> Field<T> {
    /// Transition to another state by decoding a field of base type `T`.
    ///
    /// Returns the field value as a Rust primitive if the field did not contain
    /// its 'invalid' marker value, and a successor state.
    pub fn advance(
        self,
        r: T::From,
    ) -> (
        Option<T::Into>,
        Either<Either<DefinitionFieldAlt, RecordHeader>, Self>,
    ) {
        let value = T::from(r, self.is_little_endian);

        let size = size_of::<T::From>() as u8;

        let successor = if self.bytes_remaining == size {
            Left(if self.fields_remaining != 0 {
                Left(DefinitionFieldAlt {
                    fields_remaining: self.fields_remaining,
                    is_little_endian: self.is_little_endian,
                })
            } else {
                Right(RecordHeader(()))
            })
        } else {
            Right(Self {
                fields_remaining: self.fields_remaining,
                bytes_remaining: self.bytes_remaining - size,
                is_little_endian: self.is_little_endian,
                _phantom: PhantomData,
            })
        };

        (value, successor)
    }
}

pub trait FieldInner {
    /// The data storing this base type.
    type From;
    /// The primitive corresponding to this base type.
    type Into;

    /// Convert data of this base type to the corresponding primitive, if valid.
    fn from(r: Self::From, is_le: bool) -> Option<Self::Into>;
}

macro_rules! field_inner {
    ($t:ident, $into:ident, $invalid:ident, $(#[$attr:meta])*) => {
        $(#[$attr])*
        #[derive(Debug)]
        pub struct $t;

        impl FieldInner for $t {
            type From = [u8; size_of::<Self::Into>()];
            type Into = $into;

            fn from(r: Self::From, is_le: bool) -> Option<Self::Into> {
                let x = if is_le {
                    Self::Into::from_le_bytes(r)
                } else {
                    Self::Into::from_be_bytes(r)
                };

                if x != Self::Into::$invalid {
                    Some(x)
                } else {
                    None
                }
            }
        }
    };
}

field_inner!(U8, u8, MAX, /** `uint8`, `enum`, `byte` */);
field_inner!(U8Z, u8, MIN, /** `uint8z`, `string` */);
field_inner!(U16, u16, MAX,/** `uint16` */);
field_inner!(U16Z, u16, MIN, /** `uint16z` */);
field_inner!(U32, u32, MAX, /** `uint32` */);
field_inner!(U32Z, u32, MIN, /** `uint32z` */);
field_inner!(U64, u64, MAX, /** `uint64` */);
field_inner!(U64Z, u64, MIN, /** `uint64z` */);

field_inner!(I8, i8, MAX, /** `sint8` */);
field_inner!(I16, i16, MAX, /** `sint16` */);
field_inner!(I32, i32, MAX, /** `sint32` */);
field_inner!(I64, i64, MAX, /** `sint64` */);

field_inner!(F32, f32, MAX, /** `float32` */);
field_inner!(F64, f64, MAX, /** `float64` */);

/// A `Field` state token for a base type.
pub enum AnyField {
    U8(Field<U8>),
    U8Z(Field<U8Z>),
    U16(Field<U16>),
    U16Z(Field<U16Z>),
    U32(Field<U32>),
    U32Z(Field<U32Z>),
    U64(Field<U64>),
    U64Z(Field<U64Z>),

    I8(Field<I8>),
    I16(Field<I16>),
    I32(Field<I32>),
    I64(Field<I64>),

    F32(Field<F32>),
    F64(Field<F64>),
}
