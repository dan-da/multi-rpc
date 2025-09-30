use proc_macro::TokenStream;
use quote::{format_ident, quote};
use syn::{parse_macro_input, ItemImpl, ItemTrait};

mod protocols;
use protocols::{JsonRpSee, Protocol, RestAxum, Tarpc};

const PROTOCOLS: &[&dyn Protocol] = &[&Tarpc, &RestAxum, &JsonRpSee];

#[proc_macro_attribute]
pub fn multi_rpc_trait(_attr: TokenStream, input: TokenStream) -> TokenStream {
    let mut item_trait = parse_macro_input!(input as ItemTrait);
    let trait_ident = item_trait.ident.clone();
    let generated_mod_ident = format_ident!("{}_generated", trait_ident.to_string().to_lowercase());

    let generated_trait_code: Vec<_> = PROTOCOLS.iter().map(|p| p.transform_trait(&mut item_trait)).collect();

    quote! {
        #item_trait
        pub mod #generated_mod_ident {
            use super::*;
            use std::sync::Arc;
            use multi_rpc::error::RpcError;
            #(#generated_trait_code)*
        }
    }.into()
}

#[proc_macro_attribute]
pub fn multi_rpc_impl(_attr: TokenStream, input: TokenStream) -> TokenStream {
    let item_impl = parse_macro_input!(input as ItemImpl);

    // ✅ Create the module name (e.g., `greeter_impls`) from the trait name.
    let trait_ident = &item_impl.trait_.as_ref().unwrap().1.segments.last().unwrap().ident;
    let impls_mod_ident = format_ident!("{}_impls", trait_ident.to_string().to_lowercase());

    let generated_impl_code: Vec<_> = PROTOCOLS.iter().map(|p| p.transform_impl(&item_impl)).collect();

    quote! {
        #item_impl

        // ✅ Wrap all generated code in the namespaced module.
        pub mod #impls_mod_ident {
            // `use super::*;` allows the generated code to find `MyGreeter`, `greeter_generated`, etc.
            use super::*;
            use std::sync::Arc;

            #(#generated_impl_code)*
        }
    }.into()
}

#[proc_macro_attribute]
pub fn rest(_attr: TokenStream, item: TokenStream) -> TokenStream {
    item
}
