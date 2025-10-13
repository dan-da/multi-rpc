use proc_macro::TokenStream;
use quote::quote;
use syn::parse_macro_input;
use syn::ItemImpl;
use syn::ItemTrait;

mod protocols;
use protocols::JsonRpSee;
use protocols::Protocol;
use protocols::RestAxum;
use protocols::Tarpc;

const PROTOCOLS: &[&dyn Protocol] = &[&Tarpc, &RestAxum, &JsonRpSee];

#[proc_macro_attribute]
pub fn multi_rpc_trait(_attr: TokenStream, input: TokenStream) -> TokenStream {
    let item_trait = parse_macro_input!(input as ItemTrait);

    let generated_trait_code: Vec<_> = PROTOCOLS
        .iter()
        .map(|p| p.transform_trait(&item_trait))
        .collect();

    quote! {
        #item_trait
        #(#generated_trait_code)*
    }
    .into()
}

#[proc_macro_attribute]
pub fn multi_rpc_impl(_attr: TokenStream, input: TokenStream) -> TokenStream {
    let item_impl = parse_macro_input!(input as ItemImpl);

    let generated_impl_code: Vec<_> = PROTOCOLS
        .iter()
        .map(|p| p.transform_impl(&item_impl))
        .collect();

    quote! {
        #item_impl

        #(#generated_impl_code)*
    }
    .into()
}

#[proc_macro_attribute]
pub fn rest(_attr: TokenStream, item: TokenStream) -> TokenStream {
    item
}
