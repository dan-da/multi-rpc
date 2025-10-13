use proc_macro2::TokenStream;
use quote::format_ident;
use quote::quote;
use syn::punctuated::Punctuated;
use syn::FnArg;
use syn::ImplItem;
use syn::ItemImpl;
use syn::ItemTrait;
use syn::Pat;
use syn::Token;
use syn::TraitItem;

use super::Protocol;
pub struct Tarpc;

impl Protocol for Tarpc {
    fn transform_trait(&self, item_trait: &ItemTrait) -> TokenStream {
        let original_trait_ident = &item_trait.ident;
        let tarpc_trait_ident = format_ident!("{}Tarpc", original_trait_ident);
        let generated_client_ident = format_ident!("{}Client", tarpc_trait_ident);
        let desired_client_ident = format_ident!("{}Client", original_trait_ident);

        let methods = item_trait.items.iter().filter_map(|item| {
            if let TraitItem::Fn(method) = item {
                let mut sig = method.sig.clone();
                sig.inputs = sig.inputs.into_iter().skip(1).collect();
                Some(quote! { #sig; })
            } else {
                None
            }
        });

        quote! {
            #[tarpc::service]
            pub trait #tarpc_trait_ident { #(#methods)* }

            // Alias the generated client `RPCTarpcClient` to the more ergonomic `RPCClient`.
            // This makes the change non-breaking for existing clients.
            pub use self::#generated_client_ident as #desired_client_ident;

            #[derive(Clone)]
            pub struct TarpcAdapter<S>(
                // An Arc reference to the Mutex in ServerBuilder
                pub std::sync::Arc<tokio::sync::Mutex<S>>
            );
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
        let tarpc_trait_ident = format_ident!("{}Tarpc", trait_ident);

        let request_ident = format_ident!("{}Request", tarpc_trait_ident);
        let response_ident = format_ident!("{}Response", tarpc_trait_ident);

        let adapter_methods = item_impl.items.iter().filter_map(|item| {
            if let ImplItem::Fn(method) = item {
                let sig = &method.sig;
                let method_name = &sig.ident;
                let return_ty = &sig.output;
                let user_args_and_tys: Punctuated<_, Token![,]> = sig.inputs.iter().skip(1).cloned().collect();
                let original_arg_names: Vec<Pat> = user_args_and_tys.iter().filter_map(|arg| if let FnArg::Typed(pt) = arg { Some((*pt.pat).clone()) } else { None }).collect();

                let method_call = quote! { self.0.lock().await.#method_name(#(#original_arg_names),*).await };

                Some(quote! {
                    async fn #method_name(self, _: tarpc::context::Context, #user_args_and_tys) #return_ty {
                        #method_call
                    }
                })
            } else { None }
        });

        quote! {
            impl #tarpc_trait_ident for TarpcAdapter<#self_ty> {
                #(#adapter_methods)*
            }

            async fn run_tarpc_server<L, T>(service: std::sync::Arc<tokio::sync::Mutex<#self_ty>>, mut listener: L)
            where
                L: futures::Stream<Item = std::io::Result<T>> + Unpin,
                T: tarpc::Transport<
                    tarpc::Response<#response_ident>,
                    tarpc::ClientMessage<#request_ident>
                > + Send + 'static,
            {
                use futures::StreamExt;
                use tarpc::server::{BaseChannel, Channel};

                println!("📡 Tarpc server starting...");
                while let Some(Ok(transport)) = listener.next().await {
                    let server = TarpcAdapter(service.clone());
                    let channel = BaseChannel::with_defaults(transport).execute(server.serve());
                    tokio::spawn(channel.for_each_concurrent(None, |f| f));
                }
            }

            pub fn tarpc_tcp(addr: std::net::SocketAddr)
                -> impl FnOnce(std::sync::Arc<tokio::sync::Mutex<#self_ty>>) -> std::pin::Pin<Box<dyn std::future::Future<Output = ()> + Send>>
            {
                move |service|
                {
                    Box::pin(async move {
                        let listener = tarpc::serde_transport::tcp::listen(addr, tarpc::tokio_serde::formats::Json::default).await.unwrap();
                        run_tarpc_server(service, listener).await;
                    })
                }
            }
        }
    }
}
