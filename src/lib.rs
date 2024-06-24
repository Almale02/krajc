use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, AttributeArgs, ItemFn};

#[proc_macro_attribute]
pub fn system_fn(attr: TokenStream, item: TokenStream) -> TokenStream {
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
