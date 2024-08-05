use proc_macro::TokenStream;
use quote::{format_ident, quote};
use syn::{parse::Parser, parse_macro_input, AttributeArgs, FnArg, ItemFn, Pat, PatType, Result};
use syn::{Data, DeriveInput, Fields, Stmt};
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
    let fn_name_string = fn_name.to_string();
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
                let funct_name = #fn_name_string;
                $runtime.get_resource::<#type_param>().register(Box::new( ( funct_name, Box::new(#fn_name) as Box<dyn Fn(#macro_invocation)>)))

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
    let mut input_fn = parse_macro_input!(item as ItemFn);

    // Extract function name and parameters
    let fn_name = &input_fn.sig.ident;
    let fn_name_string = fn_name.to_string();

    // Generate the new statement to be added at the start of the function body
    let span_stmt: Stmt = syn::parse_quote! {
        crate::span!(span, #fn_name_string);
    };

    // Modify the function body to include the new statement at the start
    let block = &mut input_fn.block;
    block.stmts.insert(0, span_stmt);

    // Generate macro invocation code for parameters
    let fn_params = &input_fn.sig.inputs;
    let mut macro_invocation = quote! {};
    for _ in fn_params {
        macro_invocation = quote! {
            #macro_invocation _,
        };
    }

    let expanded = quote! {
        #[macro_export]
        #[allow(macro_expanded_macro_exports_accessed_by_absolute_paths)]
        macro_rules! #fn_name {
            ($runtime: expr) => {
                $crate::typed_addr::dupe($runtime).get_resource_mut::<#type_param>().register(Box::new((#fn_name_string, Box::new(#fn_name) as Box<dyn Fn(#macro_invocation)>, Vec::default())))
            };
        }

        #input_fn
    };

    // Return the generated code as a TokenStream
    TokenStream::from(expanded)
}

#[proc_macro_attribute]
pub fn system_fn_non_expand(attr: TokenStream, item: TokenStream) -> TokenStream {
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

#[proc_macro_derive(Comp)]
pub fn comp_derive(input: TokenStream) -> TokenStream {
    let ast = syn::parse(input).unwrap();

    impl_comp(&ast)
}

fn impl_comp(ast: &syn::DeriveInput) -> TokenStream {
    let name = &ast.ident;
    let gen = quote! {
        impl legion::internals::world::Comp for #name {}

    };
    gen.into()
}

#[proc_macro_derive(EngineResource)]
pub fn res_derive(input: TokenStream) -> TokenStream {
    let ast = syn::parse(input).unwrap();

    impl_res(&ast)
}
fn impl_res(ast: &syn::DeriveInput) -> TokenStream {
    let name = &ast.ident;
    let gen = quote! {
    use crate::FromEngine as _;

    /*use crate::EngineRuntime;
    use crate::TypedAddr;
    use std::any::TypeId;*/


    impl EngineResource for #name {
        fn get_mut(engine: &mut crate::EngineRuntime) -> &'static mut Self {
            crate::TypedAddr::new({
                let op = engine.static_resource_map.get_mut(&std::any::TypeId::of::<Self>());
                match op {
                    Some(val) => *val,
                    None => {
                        let new = Box::leak(Box::new(#name::from_engine(crate::dupe(engine))));
                        let addr = crate::TypedAddr::new_with_ref(new).addr;
                        engine.static_resource_map.insert(std::any::TypeId::of::<Self>(), addr);
                        addr
                    }
                }
            })
            .get()
        }
        fn get(engine: &mut crate::EngineRuntime) -> &'static Self {
            crate::TypedAddr::new({
                let op = engine.static_resource_map.get_mut(&std::any::TypeId::of::<Self>());
                match op {
                    Some(val) => *val,
                    None => {
                        let new = Box::leak(Box::new(#name::from_engine(crate::dupe(engine))));
                        let addr = crate::TypedAddr::new_with_ref(new).addr;
                        engine.static_resource_map.insert(std::any::TypeId::of::<Self>(), addr);
                        addr
                    }
                }
            })
            .get()
        }
    }

    };
    gen.into()
}

use syn::Ident;

#[proc_macro]
pub fn not_prod(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as Ident);
    let expanded = quote! {
        #[cfg(not(feature = "prod"))]
        {
            #input
        }
    };
    TokenStream::from(expanded)
}

#[proc_macro_derive(FromEngine)]
pub fn from_engine_macro(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);

    let name = &input.ident;

    let expanded = match input.data {
        Data::Struct(data_struct) => {
            let field_inits = match data_struct.fields {
                Fields::Named(fields_named) => fields_named
                    .named
                    .iter()
                    .map(|field| {
                        let field_name = &field.ident;
                        let field_type = &field.ty;

                        quote! {
                            #field_name: <#field_type as crate::FromEngine>::from_engine(dupe(runtime))
                        }
                    })
                    .collect::<Vec<_>>(),
                _ => panic!("FromEngine can only be derived for structs with named fields"),
            };

            quote! {
                impl #name {
                    pub fn from_engine(runtime: &'static mut EngineRuntime) -> Self {
                        Self {
                            #(#field_inits),*
                        }
                    }
                }
            }
        }
        _ => panic!("FromEngine can only be derived for structs"),
    };

    TokenStream::from(expanded)
}
