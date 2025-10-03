use proc_macro2::TokenStream;
use quote::format_ident;
use quote::quote;
use syn::FnArg;
use syn::ImplItem;
use syn::ItemImpl;
use syn::ItemTrait;
use syn::Pat;
use syn::ReturnType;
use syn::TraitItem;
use syn::Type;

use super::Protocol;

pub struct JsonRpSee;

impl Protocol for JsonRpSee {
    fn transform_trait(&self, item_trait: &ItemTrait) -> TokenStream {
        let rpc_trait_ident = format_ident!("{}Rpc", item_trait.ident);

        let methods = item_trait.items.iter().filter_map(|item| {
            if let TraitItem::Fn(method) = item {
                let mut sig = method.sig.clone();
                let method_name = sig.ident.to_string();

                // **Strategy: All methods in the adapted trait will return the same universal type.**
                sig.output = syn::parse_quote! {
                    -> Result<serde_json::Value, jsonrpsee::types::error::ErrorObject<'static>>
                };

                Some(quote! {
                    #[method(name = #method_name)]
                    #sig;
                })
            } else {
                None
            }
        });

        quote! {
            use jsonrpsee::proc_macros::rpc;

            #[rpc(server)]
            pub trait #rpc_trait_ident { #(#methods)* }

            #[derive(Clone)]
            pub struct RpcAdapter<S>(pub std::sync::Arc<S>);
        }
    }

    fn transform_impl(&self, item_impl: &ItemImpl) -> TokenStream {
        let self_ty = &item_impl.self_ty;
        let trait_ident = &item_impl
            .trait_
            .as_ref()
            .unwrap()
            .1
            .segments
            .last()
            .unwrap()
            .ident;
        let generated_mod_ident =
            format_ident!("{}_generated", trait_ident.to_string().to_lowercase());
        let rpc_trait_ident = format_ident!("{}RpcServer", trait_ident);

        let method_impls = item_impl.items.iter().filter_map(|item| {
            if let ImplItem::Fn(method) = item {
                let sig = &method.sig;
                let method_ident = &sig.ident;
                let arg_names: Vec<Pat> = method
                    .sig
                    .inputs
                    .iter()
                    .skip(1)
                    .filter_map(|arg| {
                        if let FnArg::Typed(pt) = arg {
                            Some((*pt.pat).clone())
                        } else {
                            None
                        }
                    })
                    .collect();

                let (adapted_sig, body) = {
                    let mut is_result = false;
                    let mut adapted_sig = sig.clone();
                    adapted_sig.output = syn::parse_quote! {
                        -> Result<serde_json::Value, jsonrpsee::types::error::ErrorObject<'static>>
                    };

                    // Check if the original return type was a Result.
                    if let ReturnType::Type(_, ty) = &sig.output {
                        if let Type::Path(type_path) = &**ty {
                            if let Some(segment) = type_path.path.segments.last() {
                                if segment.ident == "Result" {
                                    is_result = true;
                                }
                            }
                        }
                    }

                    let body_logic = if is_result {
                        // CASE 1: The original returns a Result, so we match and adapt it.
                        quote! {
                            match self.0.#method_ident(#(#arg_names),*).await {
                                Ok(ok_value) => {
                                    match serde_json::to_value(ok_value) {
                                        Ok(json_value) => Ok(json_value),
                                        Err(e) => Err(jsonrpsee::types::error::ErrorObject::owned(
                                            jsonrpsee::types::error::ErrorCode::InternalError.code(),
                                            e.to_string(),
                                            None::<()>,
                                        )),
                                    }
                                }
                                Err(err) => Err(jsonrpsee::types::error::ErrorObject::owned(
                                    jsonrpsee::types::error::ErrorCode::InternalError.code(),
                                    err.to_string(),
                                    None::<()>,
                                )),
                            }
                        }
                    } else {
                        // CASE 2: The original returns a direct serializable type.
                        let result_expr = quote! { self.0.#method_ident(#(#arg_names),*).await };
                        quote! {
                             match serde_json::to_value(#result_expr) {
                                Ok(json_value) => Ok(json_value),
                                Err(e) => Err(jsonrpsee::types::error::ErrorObject::owned(
                                    jsonrpsee::types::error::ErrorCode::InternalError.code(),
                                    format!("Failed to serialize RPC response: {}", e),
                                    None::<()>,
                                )),
                            }
                        }
                    };
                    (adapted_sig, body_logic)
                };

                Some(quote! {
                    #adapted_sig {
                        #body
                    }
                })
            } else {
                None
            }
        });

        quote! {
            #[jsonrpsee::core::async_trait]
            impl #generated_mod_ident::#rpc_trait_ident for #generated_mod_ident::RpcAdapter<#self_ty>
            {
                #(#method_impls)*
            }

            pub fn jsonrpsee(addr: std::net::SocketAddr)
                -> impl FnOnce(std::sync::Arc<#self_ty>) -> std::pin::Pin<Box<dyn std::future::Future<Output = ()> + Send>>
            {
                use #generated_mod_ident::#rpc_trait_ident;
                move |service| {
                    Box::pin(async move {
                        let module = #generated_mod_ident::RpcAdapter(service).into_rpc();
                        println!("🌐 JSON-RPC (jsonrpsee) server listening on http://{}", addr);
                        let server = jsonrpsee::server::Server::builder().build(addr).await.unwrap();
                        server.start(module).stopped().await;
                    })
                }
            }
        }
    }
}
