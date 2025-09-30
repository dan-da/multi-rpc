use proc_macro2::TokenStream;
use syn::{ItemImpl, ItemTrait};

#[cfg(feature = "tarpc")] mod tarpc;
#[cfg(feature = "tarpc")] pub use tarpc::Tarpc;
#[cfg(feature = "rest-axum")] mod rest_axum;
#[cfg(feature = "rest-axum")] pub use rest_axum::RestAxum;
#[cfg(feature = "jsonrpsee")] mod jsonrpsee;
#[cfg(feature = "jsonrpsee")] pub use jsonrpsee::JsonRpSee;

#[cfg(not(feature = "tarpc"))] pub struct Tarpc;
#[cfg(not(feature = "rest-axum"))] pub struct RestAxum;
#[cfg(not(feature = "jsonrpsee"))] pub struct JsonRpSee;

/// A trait defining a consistent interface for all RPC protocol generators.
pub trait Protocol: Sync {
    /// Transforms the user's trait definition.
    fn transform_trait(&self, item_trait: &ItemTrait) -> TokenStream;
    /// Transforms the user's `impl` block to generate adapter implementations.
    fn transform_impl(&self, item_impl: &ItemImpl) -> TokenStream;
}

// --- Dummy Trait Impls for Disabled Features ---
#[cfg(not(feature = "tarpc"))]
impl Protocol for Tarpc {
    fn transform_trait(&self, _: &ItemTrait) -> TokenStream { quote::quote! {} }
    fn transform_impl(&self, _: &ItemImpl) -> TokenStream { quote::quote! {} }
}
#[cfg(not(feature = "rest-axum"))]
impl Protocol for RestAxum {
    fn transform_trait(&self, _: &ItemTrait) -> TokenStream { quote::quote! {} }
    fn transform_impl(&self, _: &ItemImpl) -> TokenStream { quote::quote! {} }
}
#[cfg(not(feature = "jsonrpsee"))]
impl Protocol for JsonRpSee {
    fn transform_trait(&self, _: &ItemTrait) -> TokenStream { quote::quote! {} }
    fn transform_impl(&self, _: &ItemImpl) -> TokenStream { quote::quote! {} }
}
