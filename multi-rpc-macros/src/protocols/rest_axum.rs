use std::collections::HashMap;

use proc_macro2::TokenStream;
use quote::format_ident;
use quote::quote;
use syn::parse::Parse;
use syn::parse::ParseStream;
use syn::punctuated::Punctuated;
use syn::FnArg;
use syn::Ident;
use syn::ImplItem;
use syn::ItemImpl;
use syn::ItemTrait;
use syn::LitStr;
use syn::Pat;
use syn::Result;
use syn::ReturnType;
use syn::Token;
use syn::Type;

use super::Protocol;

// Represents a mapping from a public API name to a private Rust variable name.
// Can be either a simple identifier `limit` (shorthand for `limit = limit`)
// or an explicit rename `q = search_query`.
struct ParamMapping {
    public_name: Ident,
    private_name: Ident,
}

impl Parse for ParamMapping {
    fn parse(input: ParseStream) -> Result<Self> {
        let public_name: Ident = input.parse()?;
        if input.peek(Token![=]) {
            let _eq_token: Token![=] = input.parse()?;
            let private_name: Ident = input.parse()?;
            Ok(ParamMapping {
                public_name,
                private_name,
            })
        } else {
            Ok(ParamMapping {
                public_name: public_name.clone(),
                private_name: public_name,
            })
        }
    }
}

// Main struct to parse the entire `#[rest(...)]` attribute.
struct RestAttribute {
    method: Ident,
    path: LitStr,
    query_params: Punctuated<ParamMapping, Token![,]>,
    body_params: Punctuated<ParamMapping, Token![,]>,
}

impl Parse for RestAttribute {
    fn parse(input: ParseStream) -> Result<Self> {
        let mut method = None;
        let mut path = None;
        let mut query_params = Punctuated::new();
        let mut body_params = Punctuated::new();

        let top_level_vars = Punctuated::<syn::Meta, Token![,]>::parse_terminated(input)?;

        for meta in top_level_vars {
            if meta.path().is_ident("method") {
                if let syn::Meta::NameValue(nv) = meta {
                    if let syn::Expr::Path(expr_path) = nv.value {
                        method = expr_path.path.get_ident().cloned();
                    }
                }
            } else if meta.path().is_ident("path") {
                if let syn::Meta::NameValue(nv) = meta {
                    if let syn::Expr::Lit(expr_lit) = nv.value {
                        if let syn::Lit::Str(lit_str) = expr_lit.lit {
                            path = Some(lit_str);
                        }
                    }
                }
            } else if meta.path().is_ident("query") {
                if let syn::Meta::List(list) = meta {
                    query_params = list.parse_args_with(Punctuated::parse_terminated)?;
                }
            } else if meta.path().is_ident("body") {
                if let syn::Meta::List(list) = meta {
                    body_params = list.parse_args_with(Punctuated::parse_terminated)?;
                }
            }
        }

        Ok(RestAttribute {
            method: method
                .ok_or_else(|| syn::Error::new(input.span(), "Missing `method` argument"))?,
            path: path.ok_or_else(|| syn::Error::new(input.span(), "Missing `path` argument"))?,
            query_params,
            body_params,
        })
    }
}

pub struct RestAxum;

impl Protocol for RestAxum {
    fn transform_trait(&self, _item_trait: &ItemTrait) -> TokenStream {
        quote! {}
    }

    fn transform_impl(&self, item_impl: &ItemImpl) -> TokenStream {
        let self_ty = &item_impl.self_ty;

        let mut routes = Vec::new();
        let mut wrapper_structs = Vec::new();

        for item in &item_impl.items {
            if let ImplItem::Fn(method) = item {
                if let Some(attr) = method.attrs.iter().find(|a| a.path().is_ident("rest")) {
                    let rest_attr: RestAttribute = match attr.parse_args() {
                        Ok(attr) => attr,
                        Err(_) => continue,
                    };

                    let http_method =
                        format_ident!("{}", rest_attr.method.to_string().to_lowercase());
                    let path = &rest_attr.path;
                    let method_ident = &method.sig.ident;

                    let mut handler_args = vec![];
                    let mut call_args = vec![];

                    let all_fn_args: HashMap<_, _> = method
                        .sig
                        .inputs
                        .iter()
                        .skip(1)
                        .filter_map(|arg| {
                            if let FnArg::Typed(pt) = arg {
                                if let Pat::Ident(pi) = &*pt.pat {
                                    return Some((pi.ident.clone(), &pt.ty));
                                }
                            }
                            None
                        })
                        .collect();

                    let path_str = path.value();
                    let path_params: Vec<_> = path_str
                        .split('/')
                        .filter(|s| s.starts_with(':'))
                        .map(|s| format_ident!("{}", &s[1..]))
                        .collect();

                    for p_param in &path_params {
                        handler_args.push(quote! { axum::extract::Path(#p_param) });
                        call_args.push(quote! { #p_param });
                    }

                    if !rest_attr.query_params.is_empty() {
                        let query_wrapper_ident =
                            format_ident!("{}Query", method_ident.to_string());
                        let mut query_wrapper_fields = vec![];
                        for q_param in &rest_attr.query_params {
                            let pub_name_str = q_param.public_name.to_string();
                            let priv_name = &q_param.private_name;
                            let arg_ty = all_fn_args.get(priv_name).unwrap();
                            query_wrapper_fields.push(
                                quote! { #[serde(rename = #pub_name_str)] pub #priv_name: #arg_ty },
                            );
                            call_args.push(quote! { query_params.#priv_name });
                        }
                        handler_args.push(quote! { axum::extract::Query(query_params): axum::extract::Query<#query_wrapper_ident> });
                        wrapper_structs.push(quote! {
                            #[derive(serde::Deserialize)]
                            pub struct #query_wrapper_ident {
                                #(#query_wrapper_fields),*
                            }
                        });
                    }

                    if !rest_attr.body_params.is_empty() {
                        let body_wrapper_ident = format_ident!("{}Body", method_ident.to_string());
                        let mut body_wrapper_fields = vec![];
                        for b_param in &rest_attr.body_params {
                            let pub_name_str = b_param.public_name.to_string();
                            let priv_name = &b_param.private_name;
                            let arg_ty = all_fn_args.get(priv_name).unwrap();
                            body_wrapper_fields.push(
                                quote! { #[serde(rename = #pub_name_str)] pub #priv_name: #arg_ty },
                            );
                            call_args.push(quote! { body_params.#priv_name });
                        }
                        handler_args.push(quote! { axum::extract::Json(body_params): axum::extract::Json<#body_wrapper_ident> });
                        wrapper_structs.push(quote! {
                            #[derive(serde::Deserialize)]
                            pub struct #body_wrapper_ident {
                                #(#body_wrapper_fields),*
                            }
                        });
                    }

                    // The Mutex is needed to get exclusive access to the service
                    // and correctly call methods that take &mut self.
                    let method_call =
                        quote! { service.lock().await.#method_ident(#(#call_args),*).await };

                    let mut is_result = false;
                    if let ReturnType::Type(_, ty) = &method.sig.output {
                        if let Type::Path(type_path) = &**ty {
                            if let Some(segment) = type_path.path.segments.last() {
                                if segment.ident == "Result" {
                                    is_result = true;
                                }
                            }
                        }
                    }

                    let handler_body = if is_result {
                        quote! {
                            match #method_call {
                                Ok(result) => axum::response::Json(result).into_response(),
                                Err(e) => (axum::http::StatusCode::INTERNAL_SERVER_ERROR, e.to_string()).into_response(),
                            }
                        }
                    } else {
                        quote! {
                            let result = #method_call;
                            axum::response::Json(result).into_response()
                        }
                    };

                    let handler_args_punctuated =
                        Punctuated::<_, Token![,]>::from_iter(handler_args);

                    routes.push(quote! {
                        .route(#path, axum::routing::#http_method(|
                            axum::extract::State(service): axum::extract::State<std::sync::Arc<tokio::sync::Mutex<#self_ty>>>,
                            #handler_args_punctuated
                        | async move {
                            use axum::response::IntoResponse;
                            #handler_body
                        }))
                    });
                }
            }
        }

        quote! {
            #(#wrapper_structs)*

            pub fn rest_axum(addr: std::net::SocketAddr)
                -> impl FnOnce(std::sync::Arc<tokio::sync::Mutex<#self_ty>>) -> std::pin::Pin<Box<dyn std::future::Future<Output = ()> + Send>>
            {
                use std::sync::Arc;
                use tokio::sync::Mutex;

                move |service| {
                    Box::pin(async move {
                        let app = axum::Router::new()
                            #(#routes)*
                            .with_state(service);

                        println!("🌐 REST (Axum) server listening on http://{}", addr);
                        let listener = tokio::net::TcpListener::bind(addr).await.unwrap();
                        axum::serve(listener, app.into_make_service()).await.unwrap();
                    })
                }
            }
        }
    }
}
