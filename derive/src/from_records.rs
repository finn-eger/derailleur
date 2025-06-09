use proc_macro::TokenStream;
use quote::quote;
use syn::{
    Data, DeriveInput, Error, Field, Fields, Ident, LitInt, Result, Type,
    parse::{Parse, ParseStream},
};

pub(crate) fn expand_from_records(input: &DeriveInput) -> Result<TokenStream> {
    let Data::Struct(data) = &input.data else {
        Err(Error::new_spanned(
            input,
            "`FromRecords` may only be derived on structs.",
        ))?
    };

    let Fields::Named(fields) = &data.fields else {
        Err(Error::new_spanned(
            input,
            "`FromRecords` may only be derived on structs with named fields.",
        ))?
    };

    let fields = fields
        .named
        .iter()
        .map(FieldMetadata::parse)
        .map(Result::transpose)
        .flatten() // Skip fields without an attribute.
        .collect::<Result<Vec<_>>>()?;

    let cases = fields.iter().map(|field| {
        let FieldMetadata {
            name,
            number,
            is_vec,
        } = field;

        let assignment = if *is_vec {
            quote! {
                self.#name.push(Default::default());
                self.#name.last_mut().map(|r| r as _)
            }
        } else {
            quote! {
                Some(self.#name.insert(Default::default()))
            }
        };

        quote! { #number => { #assignment } }
    });

    let name = &input.ident;

    let expanded = quote! {
        impl FromRecords for #name {
            fn add_record(&mut self, id: u16) -> Option<&mut dyn FromRecord> {
                match id {
                    #(#cases)*
                    _ => None,
                }
            }
        }
    };

    Ok(expanded.into())
}

#[derive(Debug)]
struct FieldMetadata {
    name: Ident,
    number: LitInt,
    is_vec: bool,
}

impl FieldMetadata {
    fn parse(field: &Field) -> Result<Option<Self>> {
        let name = field.ident.clone().unwrap();

        let Some(attr) = field.attrs.iter().find(|a| a.path().is_ident("record")) else {
            return Ok(None);
        };

        let RecordAttribute { number } = attr.meta.require_list()?.parse_args()?;

        let Type::Path(path) = &field.ty else {
            Err(Error::new_spanned(
                &field.ty,
                "Field must have a type annotation.",
            ))?
        };

        let Some(segment) = path.path.segments.first() else {
            Err(Error::new_spanned(
                &path.path.segments,
                "Field must have an `Option<T>` or `Vec<T>` type.",
            ))?
        };

        let is_vec = if segment.ident == "Option" {
            false
        } else if segment.ident == "Vec" {
            true
        } else {
            Err(Error::new_spanned(
                &segment.ident,
                "Field must have an `Option<T>` or `Vec<T>` type.",
            ))?
        };

        Ok(Some(Self {
            name,
            number,
            is_vec,
        }))
    }
}

#[derive(Debug)]
struct RecordAttribute {
    number: LitInt,
}

impl Parse for RecordAttribute {
    fn parse(input: ParseStream) -> Result<Self> {
        let number = input.parse::<LitInt>()?;
        Ok(Self { number })
    }
}
