use std::{collections::HashMap, fmt::Debug};

use proc_macro::{Span, TokenStream};
use quote::{ToTokens, format_ident, quote};
use syn::{
    Data, DeriveInput, Error, ExprClosure, Field, Fields, GenericArgument, Ident, LitInt, Pat,
    PathArguments, Result, Token, Type,
    parse::{Parse, ParseStream},
    spanned::Spanned,
};

pub(crate) fn expand_from_record(input: &DeriveInput) -> Result<TokenStream> {
    let Data::Struct(data) = &input.data else {
        Err(Error::new(
            input.span(),
            "`FromRecord` may only be derived on structs.",
        ))?
    };

    let Fields::Named(fields) = &data.fields else {
        Err(Error::new(
            input.span(),
            "`FromRecord` may only be derived on structs with named fields.",
        ))?
    };

    let fields = fields
        .named
        .iter()
        .map(FieldMetadata::parse)
        .map(Result::transpose)
        .flatten() // Skip fields without an attribute.
        .collect::<Result<Vec<_>>>()?;

    type Case = (Ident, Option<(Type, ExprClosure)>);
    let mut field_methods: HashMap<Type, HashMap<LitInt, Case>> = HashMap::new();
    let mut time_method: Option<Case> = None;

    for field in fields {
        let assignment = (field.name, field.handler);

        match field.identifier {
            FieldIdentifier::Number(number) => {
                let existing = field_methods
                    .entry(field.primitive)
                    .or_default()
                    .insert(number.clone(), assignment);

                if !existing.is_none() {
                    Err(Error::new(
                        number.span(),
                        "Field identifiers must be unique.",
                    ))?
                }
            }
            FieldIdentifier::Time => {
                let existing = time_method.replace(assignment);

                if !existing.is_none() {
                    Err(Error::new(
                        field.span.into(),
                        "Field identifiers must be unique.",
                    ))?;
                }
            }
        }
    }

    let field_methods = field_methods.into_iter().map(|(primitive, fields)| {
        let cases = fields.into_iter().map(|(number, (name, handler))| {
            let assignment = if let Some((field_type, handler)) = handler {
                let body = handler.body;
                let acc = handler.inputs.iter().nth(0).unwrap();
                let val = handler.inputs.iter().nth(1).unwrap();

                quote! {
                    (|#acc: &mut #field_type, #val| {#body})(&mut self.#name, value)
                }
            } else {
                quote! {
                    self.#name = Some(value)
                }
            };

            quote! { #number => { #assignment } }
        });

        let primitive = format_ident!("{}", primitive.to_token_stream().to_string());
        let method = format_ident!("add_{}", primitive);

        quote! {
            fn #method(&mut self, field: u8, value: #primitive) {
                match field {
                    #(#cases)*
                    _ => {}
                };
            }
        }
    });

    let time_method = time_method.map(|(name, handler)| {
        let assignment = if let Some((field_type, handler)) = handler {
            let body = handler.body;
            let acc = handler.inputs.iter().nth(0).unwrap();
            let val = handler.inputs.iter().nth(1).unwrap();

            quote! {
                (|#acc: &mut #field_type, #val| #body)(&mut self.#name, offset)
            }
        } else {
            quote! { self.#name = Some(offset) }
        };

        quote! {
            fn add_time_offset(&mut self, offset: u8) {
                #assignment;
            }
        }
    });

    let name = &input.ident;

    let expanded = quote! {
        impl FromRecord for #name {
            #(#field_methods)*
            #time_method
        }
    };

    Ok(expanded.into())
}

#[derive(Debug)]
struct FieldMetadata {
    name: Ident,
    primitive: Type,
    identifier: FieldIdentifier,
    handler: Option<(Type, ExprClosure)>,
    span: Span,
}

#[derive(Debug)]
enum FieldIdentifier {
    Number(LitInt),
    Time,
}

impl FieldMetadata {
    fn parse(field: &Field) -> Result<Option<Self>> {
        let name = field.ident.clone().unwrap();

        let Some(attr) = field.attrs.iter().find(|a| a.path().is_ident("field")) else {
            return Ok(None);
        };

        let span = attr.span().unwrap();

        let FieldAttribute {
            identifier,
            handler,
        } = attr.meta.require_list()?.parse_args()?;

        let primitive = if let Some(handler) = &handler {
            let Some(parameter) = handler.inputs.iter().nth(1) else {
                Err(Error::new_spanned(
                    handler,
                    "Handler closure must have two parameters.",
                ))?
            };

            let Pat::Type(pat_type) = parameter else {
                Err(Error::new_spanned(
                    parameter,
                    "Handler closure's second parameter must be annotated with the expected primitive type.",
                ))?
            };

            (*pat_type.ty).clone()
        } else {
            let Type::Path(path) = &field.ty else {
                Err(Error::new_spanned(
                    &field.ty,
                    "Field must have a type annotation.",
                ))?
            };

            let Some(segment) = path.path.segments.first() else {
                Err(Error::new_spanned(
                    &path.path.segments,
                    "Field must have a type annotation.",
                ))?
            };

            if segment.ident != "Option" {
                Err(Error::new_spanned(
                    &segment.ident,
                    "Field without a handler must have type `Option<T>`.",
                ))?
            }

            let PathArguments::AngleBracketed(arguments) = &segment.arguments else {
                Err(Error::new_spanned(
                    &segment.arguments,
                    "Field of type `Option<T>` must have a generic parameter.",
                ))?
            };

            let Some(argument) = arguments.args.first() else {
                Err(Error::new_spanned(
                    &arguments.args,
                    "Field of type `Option<T>` must have a generic parameter.",
                ))?
            };

            let GenericArgument::Type(inner_type) = argument else {
                Err(Error::new_spanned(
                    argument,
                    "Generic argument of a field of type `Option<T>` must be a type.",
                ))?
            };

            inner_type.clone()
        };

        let handler = handler.map(|h| (field.ty.clone(), h));

        Ok(Some(Self {
            name,
            primitive,
            identifier,
            handler,
            span,
        }))
    }
}

#[derive(Debug)]
struct FieldAttribute {
    identifier: FieldIdentifier,
    handler: Option<ExprClosure>,
}

impl Parse for FieldAttribute {
    fn parse(input: ParseStream) -> Result<Self> {
        let identifier = if let Ok(ident) = input.parse::<Ident>() {
            if ident == "time" {
                FieldIdentifier::Time
            } else {
                Err(Error::new_spanned(
                    ident,
                    "Field identifier must be an integer literal or `time`.",
                ))?
            }
        } else {
            FieldIdentifier::Number(input.parse::<LitInt>()?)
        };

        let handler = if !input.is_empty() {
            input.parse::<Token![,]>()?;
            Some(input.parse::<ExprClosure>()?)
        } else {
            None
        };

        Ok(Self {
            identifier,
            handler,
        })
    }
}
