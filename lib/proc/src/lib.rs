use proc_macro::TokenStream;
use quote::{format_ident, quote};
use syn::{DeriveInput, Ident, parse_macro_input};

#[proc_macro_derive(Acceptor)]
pub fn derive_acceptor(input: TokenStream) -> TokenStream {
    // Parse the input tokens into a syntax tree
    let input = parse_macro_input!(input as DeriveInput);

    let name = input.ident;
    let func = format_ident!("visit_{}", transform_name(name.to_string()));

    quote! {
        impl Acceptor for #name {
            fn accept(&self, visitor: &mut dyn Visitor) {
                visitor.#func(self);
            }
        }
    }
    .into()
}

#[proc_macro]
pub fn acceptor_func(input: TokenStream) -> TokenStream {
    let name: Ident = parse_macro_input!(input);
    let func = format_ident!("visit_{}", transform_name(name.to_string()));

    quote! {
        fn #func (&mut self, node: &#name) {
            // Do nothing by default
        }
    }
    .into()
}

fn transform_name(name: String) -> String {
    name.to_lowercase().to_owned()
}
