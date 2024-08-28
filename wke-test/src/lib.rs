use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, ItemFn};

#[proc_macro_attribute]
pub fn setup_teardown(attr: TokenStream, item: TokenStream) -> TokenStream {
    let input = parse_macro_input!(item as ItemFn);
    let name = &input.sig.ident;
    let block = &input.block;

    let begin = input
        .attrs
        .iter()
        .find(|attr| attr.path().is_ident("begin"));
    let end = input.attrs.iter().find(|attr| attr.path().is_ident("end"));

    if let Some(begin) = begin {}

    let gen = quote! {
        #[test]
        fn #name() {
            setup();
            let result = std::panic::catch_unwind(|| {
                #block
            });
            teardown();
            if let Err(err) = result {
                std::panic::resume_unwind(err);
            }
        }
    };

    gen.into()
}
