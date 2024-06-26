use std::{borrow::Borrow, ops::Deref};

use proc_macro::TokenStream;
use quote::{format_ident, quote, quote_spanned, ToTokens};
use syn::{
    parse::{Parse, ParseStream, Parser},
    parse_macro_input, AttrStyle, AttributeArgs, Expr, FnArg, ItemFn, Lit, Meta, NestedMeta, Pat,
    PatType, Result,
};
extern crate proc_macro;

type TokenStream2 = proc_macro2::TokenStream;

#[proc_macro_attribute]
pub fn system_fn2(attr: TokenStream, item: TokenStream) -> TokenStream {
    let args = parse_macro_input!(attr as AttributeArgs);

    let type_param = match args.first() {
        Some(syn::NestedMeta::Meta(syn::Meta::Path(path))) => path,
        _ => panic!("Expected a single type parameter"),
    }; // Parse the attribute arguments

    // Parse the function definition
    let input_fn = parse_macro_input!(item as ItemFn);

    // Extract function name and parameters
    let fn_name = &input_fn.sig.ident;
    let _fn_name_string = fn_name.to_string();
    let fn_params = &input_fn.sig.inputs;

    // Generate macro invocation code
    let mut macro_invocation = quote! {};
    for _ in fn_params {
        macro_invocation = quote! {
            #macro_invocation _,
        };
    }

    // Generate macro_rules! macro
    let expanded = quote! {
        #[macro_export]
        macro_rules! #fn_name{
            ($runtime: expr) => {
                $runtime.get_resource::<#type_param>().register(Box::new( ("#fn_name", Box::new(#fn_name) as Box<dyn Fn(#macro_invocation)>)))

            };
        }

        #input_fn
    };

    // Return the generated code as a TokenStream
    TokenStream::from(expanded)
}

#[proc_macro_attribute]
pub fn system_fn(attr: TokenStream, item: TokenStream) -> TokenStream {
    let args = parse_macro_input!(attr as AttributeArgs);

    let type_param = match args.first() {
        Some(syn::NestedMeta::Meta(syn::Meta::Path(path))) => path,
        _ => panic!("Expected a single type parameter"),
    };

    // Parse the function definition
    let input_fn = parse_macro_input!(item as ItemFn);

    // Extract function name and parameters
    let fn_name = &input_fn.sig.ident;
    let fn_params = &input_fn.sig.inputs;

    // Generate macro invocation code
    let mut macro_invocation = quote! {};
    for _ in fn_params {
        macro_invocation = quote! {
            #macro_invocation _,
        };
    }

    // Collect all query parameters and their filter attributes
    let mut query_filters = Vec::new();

    for arg in fn_params.iter() {
        if let FnArg::Typed(PatType { pat, attrs, .. }) = arg {
            if let Pat::Ident(ident) = &**pat {
                for attr in attrs {
                    if attr.path.is_ident("filter") {
                        let parser = MyParsrer;
                        let content = attr
                            .parse_args_with(parser)
                            .expect("paniced because parser failed");
                        query_filters.push(content);
                    }
                }
            }
        }
    }

    // Generate unique struct names and implementations for each query parameter
    let filter_structs: Vec<_> = query_filters
        .iter()
        .map(|filter_stream| {
            let struct_name = format_ident!("SystemQueryFilter__{}__{}", fn_name, "test");
            let stream = filter_stream.to_string();
            quote! {
                // Generated struct
                struct #struct_name;

                impl SystemFilter for #struct_name {
                    fn get_query<T: IntoView>() -> impl legion::query::Query {
                        T::query().filter(#stream)
                    }
                }
            }
        })
        .collect();

    // Generate macro_rules! macro
    let expanded = quote! {
        #[macro_export]
        macro_rules! #fn_name {
            ($runtime: expr) => {
                $runtime.get_resource::<#type_param>().register(Box::new( ("#fn_name", Box::new(#fn_name) as Box<dyn Fn(#macro_invocation)>)))
            };
        }

        #input_fn

        #(#filter_structs)*
    };

    // Return the generated code as a TokenStream
    TokenStream::from(expanded)
}

struct MyParsrer;

impl Parser for MyParsrer {
    type Output = TokenStream;
    fn parse(self, tokens: proc_macro::TokenStream) -> Result<Self::Output> {
        Ok(tokens)
    }
    fn parse2(self, tokens: TokenStream2) -> Result<Self::Output> {
        Ok(std::convert::Into::into(tokens))
    }
}
