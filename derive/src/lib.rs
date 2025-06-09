use proc_macro::TokenStream;
use syn::{DeriveInput, parse_macro_input};

mod from_record;
mod from_records;

#[proc_macro_derive(FromRecord, attributes(field))]
pub fn derive_from_record(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    match from_record::expand_from_record(&input) {
        Ok(tokens) => tokens,
        Err(err) => err.to_compile_error().into(),
    }
}

#[proc_macro_derive(FromRecords, attributes(record))]
pub fn derive_from_records(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    match from_records::expand_from_records(&input) {
        Ok(tokens) => tokens,
        Err(err) => err.to_compile_error().into(),
    }
}
