use proc_macro::TokenStream;
use quote::quote;
use syn::{ItemFn, parse_macro_input};

#[proc_macro]
pub fn rsx(_input: TokenStream) -> TokenStream {
    TokenStream::from(quote! {
        // This is where the generated code will go
        // For now, we'll just return an empty token stream
    })
}

#[proc_macro_attribute]
pub fn component(_attr: TokenStream, item: TokenStream) -> TokenStream {
    let _input_fn = parse_macro_input!(item as ItemFn);
    TokenStream::from(quote! {
        // This is where the generated code will go
        // For now, we'll just return an empty token stream
    })
}
